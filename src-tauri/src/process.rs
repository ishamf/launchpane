use std::{
    future::Future,
    sync::Arc,
    time::{SystemTime, SystemTimeError},
};

use nix::{
    sys::signal::{kill, Signal},
    unistd::Pid,
};
use tauri::AppHandle;
use tokio::time::{sleep, Duration};
use tokio::{select, sync::Mutex};

use async_process::{Child, Command, Stdio};
use futures_lite::{io::BufReader, AsyncBufReadExt, StreamExt};
use tokio::{spawn, task::JoinHandle, try_join};

use log::{debug, error};

use crate::{
    errors::AppCommandError,
    events::{send_command_log_update_event, send_command_update_event},
    prisma::{_prisma::PrismaClient, command},
};

enum CommandLineSource {
    STDOUT = 1,
    STDERR = 2,

    INFO = 3,
}

fn timestamp() -> Result<f64, SystemTimeError> {
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_nanos();
    Ok(nanos as f64 / 1000000.0)
}

struct OngoingProcess {
    command_id: i32,
    child: Arc<Mutex<Child>>,
    output_join_handle: Arc<Mutex<Option<JoinHandle<Result<(), AppCommandError>>>>>,
    status_join_handle: JoinHandle<Result<(), AppCommandError>>,
}

pub struct ProcessManager {
    ongoing_processes: Arc<Mutex<Vec<OngoingProcess>>>,
    app_handle: Arc<AppHandle>,
    db_client: Arc<PrismaClient>,
}

impl ProcessManager {
    pub fn new(app_handle: Arc<AppHandle>, db_client: Arc<PrismaClient>) -> Self {
        Self {
            ongoing_processes: Arc::new(Mutex::new(vec![])),
            app_handle,
            db_client,
        }
    }

    pub async fn check_process_running(
        &mut self,
        command_id: i32,
    ) -> Result<bool, AppCommandError> {
        let ongoing_processes = self.ongoing_processes.lock().await;
        let existing_process = ongoing_processes
            .iter()
            .find(|p| p.command_id == command_id);

        match existing_process {
            None => Ok(false),
            Some(_) => Ok(true),
        }
    }

    pub async fn kill_process(&mut self, command_id: i32) -> Result<(), AppCommandError> {
        let mut ongoing_processes = self.ongoing_processes.lock().await;

        let index = ongoing_processes
            .iter()
            .position(|p| p.command_id == command_id);

        if let Some(index) = index {
            let process = ongoing_processes.swap_remove(index);

            // Stop waiting for the process to finish first, otherwise we will deadlock!
            process.status_join_handle.abort();

            debug!("Waiting for child lock");

            let mut child = process.child.lock().await;

            debug!("Got child lock");

            // We can only send signals in unix
            #[cfg(target_family = "unix")]
            {
                // Send SIGINT to the child process
                kill(Pid::from_raw(child.id() as i32), Signal::SIGTERM)?;

                debug!("Sent SIGTERM to child process with pid {}", child.id());

                // Wait for 5 seconds or until the child process exits
                select! {
                    _ = sleep(Duration::from_secs(5)) => {
                        debug!("Timed out when waiting for child process to exit");
                    }
                    _ = child.status() => {}
                }
            }

            // Check if the child process is still running
            if child.try_status()?.is_none() {
                #[cfg(target_family = "unix")]
                debug!("Child process is still running, killing it");

                // Send SIGKILL to the child process
                child.kill()?;
            }

            debug!("Child process is stopped");

            if let Some(join_handle) = process.output_join_handle.lock().await.take() {
                debug!("Got output join handle, waiting for output...");

                // Only wait for output for 1 sec, if we kill it not cleanly, the output might get stuck
                select! {
                    _ = sleep(Duration::from_secs(1)) => {}
                    _ = join_handle => {}
                }
            }

            debug!("Finished waiting for output");

            let command_killed_log = "Command killed.".to_string();

            self.db_client
                .command_log_line()
                .create(
                    command::id::equals(command_id),
                    CommandLineSource::INFO as i32,
                    command_killed_log,
                    timestamp()?,
                    vec![],
                )
                .exec()
                .await?;

            debug!("Created kill command log line");

            send_command_log_update_event(&self.app_handle, command_id)?;
            send_command_update_event(&self.app_handle, command_id)?;

            debug!("Sent update events");
        }

        Ok(())
    }

    pub async fn run_process(&mut self, command: command::Data) -> Result<(), AppCommandError> {
        let mut child = Command::new("bash")
            .args(&["-c", &command.command])
            .current_dir(command.cwd.clone())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let out_process = {
            let db = Arc::clone(&self.db_client);
            let app_handle = Arc::clone(&self.app_handle);
            let stdout = child.stdout.take().unwrap();
            async move {
                let mut lines = BufReader::new(stdout).lines();

                while let Some(line) = lines.try_next().await? {
                    db.command_log_line()
                        .create(
                            command::id::equals(command.id),
                            CommandLineSource::STDOUT as i32,
                            line,
                            timestamp()?,
                            vec![],
                        )
                        .exec()
                        .await?;
                    send_command_log_update_event(&app_handle, command.id)?;
                }

                Ok::<(), AppCommandError>(())
            }
        };

        let err_process = {
            let db = Arc::clone(&self.db_client);
            let app_handle = Arc::clone(&self.app_handle);
            let stderr = child.stderr.take().unwrap();
            async move {
                let mut lines = BufReader::new(stderr).lines();

                while let Some(line) = lines.try_next().await? {
                    db.command_log_line()
                        .create(
                            command::id::equals(command.id),
                            CommandLineSource::STDERR as i32,
                            line,
                            timestamp()?,
                            vec![],
                        )
                        .exec()
                        .await?;
                    send_command_log_update_event(&app_handle, command.id)?;
                }

                Ok::<(), AppCommandError>(())
            }
        };

        let output_join_handle: JoinHandle<Result<(), AppCommandError>> =
            spawn(wrap_with_error_printer("output handler", async move {
                let res = try_join!(out_process, err_process);
                res.map(|_| ())
            }));

        let output_join_mutex = Arc::new(Mutex::new(Some(output_join_handle)));

        let child_mutex = Arc::new(Mutex::new(child));

        let status_join_handle = {
            let db = Arc::clone(&self.db_client);
            let ongoing_processes = Arc::clone(&self.ongoing_processes);
            let output_mutex = Arc::clone(&output_join_mutex);
            let spawned_child_mutex = Arc::clone(&child_mutex);
            let app_handle = Arc::clone(&self.app_handle);
            spawn(wrap_with_error_printer("status handler", async move {
                let mut child = spawned_child_mutex.lock().await;
                let status = child.status().await?;

                debug!("Child process exited with status {}", status);

                let output = output_mutex.lock().await.take();

                if let Some(output) = output {
                    debug!("Got output handle, waiting for output...");
                    output.await??;
                }

                ongoing_processes
                    .lock()
                    .await
                    .retain(|p| p.command_id != command.id);

                debug!("Removed process from ongoing processes");

                let command_exit_log = format!("Command exited with status {}", status);
                db.command_log_line()
                    .create(
                        command::id::equals(command.id),
                        CommandLineSource::INFO as i32,
                        command_exit_log,
                        timestamp()?,
                        vec![],
                    )
                    .exec()
                    .await?;

                debug!("Created exit command log line");

                send_command_log_update_event(&app_handle, command.id)?;
                send_command_update_event(&app_handle, command.id)?;

                debug!("Sent command log update event");

                Ok::<(), AppCommandError>(())
            }))
        };

        self.ongoing_processes.lock().await.push(OngoingProcess {
            command_id: command.id,
            child: child_mutex,
            output_join_handle: output_join_mutex,
            status_join_handle,
        });

        let start_command_log =
            format!("Running command `{}` at `{}`", command.command, command.cwd);

        self.db_client
            .command_log_line()
            .create(
                command::id::equals(command.id),
                CommandLineSource::INFO as i32,
                start_command_log,
                timestamp()?,
                vec![],
            )
            .exec()
            .await?;

        send_command_log_update_event(&self.app_handle, command.id)?;
        send_command_update_event(&self.app_handle, command.id)?;

        Ok(())
    }
}

async fn wrap_with_error_printer<R, T: Future<Output = Result<R, AppCommandError>>>(
    name: &str,
    future: T,
) -> Result<R, AppCommandError> {
    let res = future.await;
    match &res {
        Ok(_) => {}
        Err(err) => {
            error!(
                "Error in {}: {}",
                name,
                serde_json::to_string(&err).unwrap_or_default()
            );
        }
    }

    res
}
