use std::{
    future::Future,
    sync::Arc,
    time::{SystemTime, SystemTimeError},
};

#[cfg(target_family = "unix")]
use nix::{
    sys::signal::{kill, Signal},
    unistd::Pid,
};

#[cfg(target_family = "windows")]
use async_process::windows::CommandExt;

use serde::Serialize;
use specta::Type;
use tauri::AppHandle;
use tokio::time::{sleep, Duration};
use tokio::{select, sync::Mutex};

use async_process::{Child, Command, Stdio};
use futures_lite::{io::BufReader, AsyncBufReadExt, StreamExt};
use tokio::{spawn, task::JoinHandle, try_join};

use log::{debug, error, trace};

use crate::{
    errors::AppCommandError,
    events::{send_command_log_update_event, send_command_update_event},
    prisma::{_prisma::PrismaClient, command},
};


enum CommandLogLineSource {
    STDOUT = 1,
    STDERR = 2,
    INFO = 3,
}

enum LastRunResultType {
    Exit,
    Killed,
    Error,
}

impl LastRunResultType {
    pub fn as_str(&self) -> &'static str {
        match self {
            LastRunResultType::Exit => "exit",
            LastRunResultType::Killed => "killed",
            LastRunResultType::Error => "error",
        }
    }
}

fn timestamp() -> Result<f64, SystemTimeError> {
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_nanos();
    Ok(nanos as f64 / 1000000.0)
}

const CREATE_NO_WINDOW: u32 = 0x08000000;

struct OngoingProcess {
    command_id: i32,
    child: Arc<Mutex<Child>>,

    // The join handle of the task that waits for the process to finish
    status_join_handle: JoinHandle<Result<(), AppCommandError>>,

    // The join handle of the task to store the output of the process
    // It can be consumed either when the process stops normally or when it's killed, so we need to use an Option here
    output_join_handle: Arc<Mutex<Option<JoinHandle<Result<(), AppCommandError>>>>>,
}

pub struct ProcessManager {
    ongoing_processes: Arc<Mutex<Vec<OngoingProcess>>>,
    stopping_commands: Arc<Mutex<Vec<i32>>>,
    app_handle: Arc<AppHandle>,
    db_client: Arc<PrismaClient>,
}

#[derive(Debug, Serialize, Type, Clone, Copy)]
pub enum ProcessStatus {
    Running,
    Stopping,
    Stopped,
}

impl ProcessManager {
    pub fn new(app_handle: Arc<AppHandle>, db_client: Arc<PrismaClient>) -> Self {
        Self {
            ongoing_processes: Arc::new(Mutex::new(vec![])),
            stopping_commands: Arc::new(Mutex::new(vec![])),
            app_handle,
            db_client,
        }
    }

    pub async fn check_process_status(
        &self,
        command_id: i32,
    ) -> Result<ProcessStatus, AppCommandError> {
        let existing_process = self
            .ongoing_processes
            .lock()
            .await
            .iter()
            .find(|p| p.command_id == command_id)
            .is_some();

        if existing_process {
            Ok(ProcessStatus::Running)
        } else if self.stopping_commands.lock().await.contains(&command_id) {
            Ok(ProcessStatus::Stopping)
        } else {
            Ok(ProcessStatus::Stopped)
        }
    }

    pub async fn kill_process(&self, command_id: i32) -> Result<(), AppCommandError> {
        let mut ongoing_processes = self.ongoing_processes.lock().await;

        let index = ongoing_processes
            .iter()
            .position(|p| p.command_id == command_id);

        if let Some(index) = index {
            let process = ongoing_processes.swap_remove(index);

            // Drop the lock first to avoid deadlock
            drop(ongoing_processes);
            self.stopping_commands.lock().await.push(command_id);
            send_command_update_event(&self.app_handle, command_id)?;

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

            #[cfg(target_family = "windows")]
            {
                // Use taskkill to kill the process tree
                let status = Command::new("taskkill")
                    .args(&["/pid", &child.id().to_string(), "/t", "/f"])
                    .creation_flags(CREATE_NO_WINDOW)
                    .spawn()?
                    .status()
                    .await?;

                debug!("Taskkill status: {}", status);

                if child.try_status()?.is_none() {
                    debug!("Child process is still running, waiting a bit");

                    sleep(Duration::from_secs(3)).await;
                }
            }

            // Check if the child process is still running
            if child.try_status()?.is_none() {
                debug!("Child process is still running, killing it");

                // Send SIGKILL to the child process
                child.kill()?;
            }

            debug!("Child process is stopped");

            if let Some(join_handle) = process.output_join_handle.lock().await.take() {
                debug!("Got output join handle, waiting for output...");

                // Only wait for output for 1 sec, if we don't kill it cleanly, the output might get stuck
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
                    CommandLogLineSource::INFO as i32,
                    command_killed_log,
                    timestamp()?,
                    vec![],
                )
                .exec()
                .await?;

            debug!("Created kill command log line");

            self.db_client
                .command()
                .update(
                    command::id::equals(command_id),
                    vec![
                        command::last_run_result_type::set(Some(LastRunResultType::Killed.as_str().into())),
                        command::last_run_code::set(None),
                    ],
                )
                .exec()
                .await?;

            debug!("Updated last run result");

            self.stopping_commands
                .lock()
                .await
                .retain(|c| *c != command_id);

            debug!("Removed command from stopping commands");

            send_command_log_update_event(&self.app_handle, command_id)?;
            send_command_update_event(&self.app_handle, command_id)?;

            debug!("Sent update events");
        }

        Ok(())
    }

    pub async fn run_process(&self, command: command::Data) -> Result<(), AppCommandError> {
        #[cfg(target_family = "windows")]
        let mut cmd = 
            // Powershell is significantly slower, but you have to use it to run commands on
            // these kinds of paths
            if command.cwd.starts_with("\\\\") {
                let mut cmd = Command::new("powershell");
                cmd.arg("-Command");
                cmd.arg(command.command.clone());

                cmd.creation_flags(CREATE_NO_WINDOW);

                cmd
            } else {
                let mut cmd = Command::new("cmd");
                cmd.arg("/s");
                cmd.arg("/c");

                cmd.raw_arg(command.command.clone());

                cmd.creation_flags(CREATE_NO_WINDOW);

                cmd
            };
        

        #[cfg(target_family = "unix")]
        let mut cmd = {
            let mut cmd = Command::new("bash");
            cmd.arg("-c").arg(&command.command);
            cmd
        };

        cmd.current_dir(command.cwd.clone())
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let child = cmd.spawn();

        if let Err(spawn_error) = child {
            let error_message = format!("Command failed to start: {}", spawn_error);

            self.db_client
                .command_log_line()
                .create(
                    command::id::equals(command.id),
                    CommandLogLineSource::INFO as i32,
                    error_message,
                    timestamp()?,
                    vec![],
                )
                .exec()
                .await?;

            self.db_client
                .command()
                .update(
                    command::id::equals(command.id),
                    vec![
                        command::last_run_result_type::set(Some(LastRunResultType::Error.as_str().into())),
                        command::last_run_code::set(None),
                    ],
                )
                .exec()
                .await?;

            send_command_log_update_event(&self.app_handle, command.id)?;
            send_command_update_event(&self.app_handle, command.id)?;

            return Ok(());
        };

        let mut child = child.expect("Spawn errors to already be handled");

        let out_process = {
            let db = Arc::clone(&self.db_client);
            let app_handle = Arc::clone(&self.app_handle);
            let stdout = child.stdout.take().unwrap();
            async move {
                let mut lines = BufReader::new(stdout).lines();

                while let Some(line) = lines.try_next().await? {
                    trace!("{} stdout: {}", command.id, line);
                    db.command_log_line()
                        .create(
                            command::id::equals(command.id),
                            CommandLogLineSource::STDOUT as i32,
                            line,
                            timestamp()?,
                            vec![],
                        )
                        .exec()
                        .await?;
                    trace!("Written to db");
                    send_command_log_update_event(&app_handle, command.id)?;
                }

                debug!("Stdout finished");

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
                    trace!("{} stderr: {}", command.id, line);
                    db.command_log_line()
                        .create(
                            command::id::equals(command.id),
                            CommandLogLineSource::STDERR as i32,
                            line,
                            timestamp()?,
                            vec![],
                        )
                        .exec()
                        .await?;
                    trace!("Written to db");
                    send_command_log_update_event(&app_handle, command.id)?;
                }

                debug!("Stderr finished");

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

                let command_exit_log = format!("Command finished with {}", status);
                db.command_log_line()
                    .create(
                        command::id::equals(command.id),
                        CommandLogLineSource::INFO as i32,
                        command_exit_log,
                        timestamp()?,
                        vec![],
                    )
                    .exec()
                    .await?;

                debug!("Created exit command log line");

                db.command()
                    .update(
                        command::id::equals(command.id),
                        vec![
                            command::last_run_result_type::set(Some(LastRunResultType::Exit.as_str().into())),
                            command::last_run_code::set(status.code().map(|c| c.to_string())),
                        ],
                    )
                    .exec()
                    .await?;

                debug!("Updated last run result");

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
                CommandLogLineSource::INFO as i32,
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
