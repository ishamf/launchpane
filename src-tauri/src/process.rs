use std::{
    sync::{Arc, Weak},
    time::{SystemTime, SystemTimeError},
};

use nix::{
    sys::signal::{kill, Signal},
    unistd::Pid,
};
use tauri::AppHandle;
use tokio::select;
use tokio::time::{sleep, Duration};

use async_process::{Child, ChildStdout, Command, Stdio};
use futures_lite::{io::BufReader, AsyncBufReadExt, Future, StreamExt};
use tokio::{join, spawn, task::JoinHandle, try_join};

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

pub struct OngoingProcess {
    command_id: i32,
    child: Child,
    join_handle: JoinHandle<Result<(), AppCommandError>>,
}

pub struct ProcessManager {
    ongoing_processes: Vec<OngoingProcess>,
    app_handle: Arc<AppHandle>,
    db_client: Arc<PrismaClient>,
}

impl ProcessManager {
    pub fn new(app_handle: Arc<AppHandle>, db_client: Arc<PrismaClient>) -> Self {
        Self {
            ongoing_processes: vec![],
            app_handle,
            db_client,
        }
    }

    pub fn check_process_running(&mut self, command_id: i32) -> Result<bool, AppCommandError> {
        let existing_process = self
            .ongoing_processes
            .iter_mut()
            .find(|p| p.command_id == command_id);

        match existing_process {
            None => Ok(false),
            Some(ongoing) => {
                let is_stopped = ongoing.child.try_status()?.is_some();

                if (is_stopped) {
                    self.ongoing_processes
                        .retain(|p| p.command_id != command_id);
                }

                Ok(!is_stopped)
            }
        }
    }

    pub async fn kill_process(&mut self, command_id: i32) -> Result<(), AppCommandError> {
        let index = self
            .ongoing_processes
            .iter()
            .position(|p| p.command_id == command_id);

        if let Some(index) = index {
            let mut process = self.ongoing_processes.swap_remove(index);

            // Send SIGINT to the child process
            kill(Pid::from_raw(process.child.id() as i32), Signal::SIGINT)?;

            // Wait for 10 seconds or until the child process exits
            select! {
                _ = sleep(Duration::from_secs(10)) => {}
                _ = process.child.status() => {}
            }

            // Check if the child process is still running
            if process.child.try_status()?.is_none() {
                // Send SIGKILL to the child process
                process.child.kill()?;
            }

            process.join_handle.await??;
        }

        Ok(())
    }

    pub fn run_process(&mut self, command: command::Data) -> Result<(), AppCommandError> {
        let mut child = Command::new("bash")
            .args(&["-c", &command.command])
            .current_dir(command.cwd)
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

        let join_handle: tokio::task::JoinHandle<Result<(), AppCommandError>> = spawn(async move {
            let res = try_join!(out_process, err_process);
            res.map(|_| ())
        });

        self.ongoing_processes.push(OngoingProcess {
            command_id: command.id,
            child,
            join_handle,
        });

        send_command_update_event(&self.app_handle, command.id)?;

        Ok(())
    }
}
