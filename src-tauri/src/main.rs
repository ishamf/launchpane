// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[allow(warnings, unused)]
mod prisma;

mod errors;
mod events;
mod process;
mod utils;

use std::{env::set_current_dir, fs::create_dir_all, path::MAIN_SEPARATOR, sync::Arc, vec};

use errors::{AppCommandError, ClientError};
use events::{send_command_update_event, AppEventPayload};
use log::info;
use prisma::*;
use tokio::join;
use utils::{get_midpoint_string, trace_elapsed_time};

use directories::ProjectDirs;
use prisma_client_rust::{Direction, QueryError};
use process::{ProcessManager, ProcessStatus};
use serde::Serialize;
use specta::{collect_types, Type};
use tauri::{api::path::home_dir, generate_handler, AppHandle, LogicalSize, Manager, Size, Window};
use tauri_specta::ts;

type AppState<'a> = tauri::State<'a, AppStateData>;

struct AppStateData {
    client: Arc<PrismaClient>,
    process_manager: Arc<ProcessManager>,
}

#[tauri::command]
#[specta::specta]
async fn get_commands(state: AppState<'_>) -> Result<Vec<command::Data>, QueryError> {
    state
        .client
        .command()
        .find_many(vec![])
        .order_by(command::order::order(Direction::Asc))
        .exec()
        .await
}

#[tauri::command]
#[specta::specta]
async fn get_command(
    state: AppState<'_>,
    command_id: i32,
) -> Result<Option<command::Data>, QueryError> {
    trace_elapsed_time("get_command", || {
        state
            .client
            .command()
            .find_unique(command::id::equals(command_id))
            .exec()
    })
    .await
}

#[tauri::command]
#[specta::specta]
async fn get_command_log_lines(
    state: AppState<'_>,
    command_id: i32,
) -> Result<Vec<command_log_line::Data>, QueryError> {
    trace_elapsed_time("get_command_log_lines", || async {
        let mut log_lines = state
            .client
            .command_log_line()
            .find_many(vec![command_log_line::command_id::equals(command_id)])
            .order_by(command_log_line::timestamp::order(Direction::Desc))
            .take(100)
            .exec()
            .await?;

        log_lines.reverse();

        Ok(log_lines)
    })
    .await
}

#[tauri::command]
#[specta::specta]
async fn get_older_command_log_lines(
    state: AppState<'_>,
    command_id: i32,
    first_id: i32,
) -> Result<Vec<command_log_line::Data>, AppCommandError> {
    trace_elapsed_time("get_older_command_log_lines", || async {
        if first_id == 0 {
            return Err(AppCommandError::ClientError(ClientError::InvalidCommandId));
        }

        let mut log_lines = state
            .client
            .command_log_line()
            .find_many(vec![command_log_line::command_id::equals(command_id)])
            .order_by(command_log_line::timestamp::order(Direction::Desc))
            .cursor(command_log_line::id::equals(first_id))
            .skip(1)
            .take(100)
            .exec()
            .await?;

        log_lines.reverse();

        Ok(log_lines)
    })
    .await
}

#[tauri::command]
#[specta::specta]
async fn get_newer_command_log_lines(
    state: AppState<'_>,
    command_id: i32,
    last_id: i32,
) -> Result<Vec<command_log_line::Data>, QueryError> {
    trace_elapsed_time("get_newer_command_log_lines", || async {
        if last_id == 0 {
            return get_command_log_lines(state, command_id).await;
        }

        state
            .client
            .command_log_line()
            .find_many(vec![command_log_line::command_id::equals(command_id)])
            .order_by(command_log_line::timestamp::order(Direction::Asc))
            .cursor(command_log_line::id::equals(last_id))
            .skip(1)
            .take(10000)
            .exec()
            .await
    })
    .await
}

command::partial_unchecked!(CommandUpdateData {
    name
    command
    cwd
});

#[tauri::command]
#[specta::specta]
async fn update_command(
    state: AppState<'_>,
    app: AppHandle,
    command_id: i32,
    data: CommandUpdateData,
) -> Result<command::Data, AppCommandError> {
    let result = state
        .client
        .command()
        .update_unchecked(command::id::equals(command_id), data.to_params())
        .exec()
        .await?;

    send_command_update_event(&app, command_id)?;

    Ok(result)
}

#[tauri::command]
#[specta::specta]
async fn delete_command(
    state: AppState<'_>,
    app: AppHandle,
    command_id: i32,
) -> Result<command::Data, AppCommandError> {
    let result = state
        .client
        .command()
        .delete(command::id::equals(command_id))
        .exec()
        .await?;

    send_command_update_event(&app, command_id)?;

    Ok(result)
}

#[tauri::command]
#[specta::specta]
async fn create_command(
    state: AppState<'_>,
    app: AppHandle,
) -> Result<command::Data, AppCommandError> {
    let result = state
        .client
        ._transaction()
        .run(|client| async move {
            let last_command = client
                .command()
                .find_first(vec![])
                .order_by(command::order::order(Direction::Desc))
                .exec()
                .await?;

            let last_order = last_command.map(|c| c.order).unwrap_or_else(String::new);

            let result = client
                .command()
                .create(
                    "".into(),
                    home_dir().unwrap_or("".into()).to_string_lossy().into(),
                    "".into(),
                    get_midpoint_string(last_order.as_str(), ""),
                    vec![],
                )
                .exec()
                .await?;

            Ok::<command::Data, AppCommandError>(result)
        })
        .await?;

    send_command_update_event(&app, result.id)?;

    Ok(result)
}

#[tauri::command]
#[specta::specta]
async fn move_command_between(
    state: AppState<'_>,
    app: AppHandle,
    command_id: i32,
    prev_command_id: Option<i32>,
    next_command_id: Option<i32>,
) -> Result<command::Data, AppCommandError> {
    let result = state
        .client
        ._transaction()
        .run(|client| async move {
            let command = client
                .command()
                .find_unique(command::id::equals(command_id))
                .exec();
            let prev_command = async {
                match prev_command_id {
                    Some(id) => {
                        client
                            .command()
                            .find_unique(command::id::equals(id))
                            .exec()
                            .await
                    }
                    None => Ok(None),
                }
            };
            let next_command = async {
                match next_command_id {
                    Some(id) => {
                        client
                            .command()
                            .find_unique(command::id::equals(id))
                            .exec()
                            .await
                    }
                    None => Ok(None),
                }
            };

            let (command, prev_command, next_command) = join!(command, prev_command, next_command);

            let new_order = match (
                command?,
                prev_command_id,
                prev_command?,
                next_command_id,
                next_command?,
            ) {
                // It's only ok to have no prev or next command if the ID is also not specified
                (Some(_), _, Some(prev_command), None, None) => {
                    Ok(get_midpoint_string(prev_command.order.as_str(), ""))
                }
                (Some(_), None, None, _, Some(next_command)) => {
                    Ok(get_midpoint_string("", next_command.order.as_str()))
                }
                (Some(_), _, Some(prev_command), _, Some(next_command)) => Ok(get_midpoint_string(
                    prev_command.order.as_str(),
                    next_command.order.as_str(),
                )),
                _ => Err(AppCommandError::ClientError(ClientError::CommandNotFound)),
            }?;

            let result = client
                .command()
                .update(
                    command::id::equals(command_id),
                    vec![command::order::set(new_order)],
                )
                .exec()
                .await?;

            Ok::<command::Data, AppCommandError>(result)
        })
        .await?;

    send_command_update_event(&app, result.id)?;

    Ok(result)
}

#[tauri::command]
#[specta::specta]
fn set_window_size(window: Window, width: f64, height: f64) -> Result<(), tauri::Error> {
    window.set_size(Size::Logical(LogicalSize { width, height }))
}

#[tauri::command]
#[specta::specta]
async fn get_process_status(
    state: AppState<'_>,
    command_id: i32,
) -> Result<ProcessStatus, AppCommandError> {
    state.process_manager.check_process_status(command_id).await
}

#[tauri::command]
#[specta::specta]
async fn run_process(state: AppState<'_>, command_id: i32) -> Result<(), AppCommandError> {
    let command = state
        .client
        .command()
        .find_unique(command::id::equals(command_id))
        .exec()
        .await?;

    match command {
        Some(command) => {
            state.process_manager.run_process(command).await?;
            Ok(())
        }
        None => Err(AppCommandError::ClientError(ClientError::CommandNotFound)),
    }
}

#[tauri::command]
#[specta::specta]
async fn kill_process(state: AppState<'_>, command_id: i32) -> Result<(), AppCommandError> {
    state.process_manager.kill_process(command_id).await
}

#[derive(Type, Serialize)]
struct PlatformDetails {
    path_separator: char,
}

#[tauri::command]
#[specta::specta]
fn get_platform_details() -> PlatformDetails {
    PlatformDetails {
        path_separator: MAIN_SEPARATOR,
    }
}

fn export_types() {
    // Export tauri-specta
    ts::export(
        collect_types![
            get_commands,
            create_command,
            get_command,
            set_window_size,
            update_command,
            move_command_between,
            delete_command,
            get_platform_details,
            get_command_log_lines,
            get_newer_command_log_lines,
            get_older_command_log_lines,
            get_process_status,
            run_process,
            kill_process,
        ],
        "../src/lib/generated/bindings.ts",
    )
    .unwrap();

    // Export custom events

    let app_event_type = specta::ts::export::<AppEventPayload>(&Default::default()).unwrap();

    std::fs::write("../src/lib/generated/events.ts", app_event_type).unwrap();
}

#[test]
fn export_types_runner() {
    export_types();
}

#[tokio::main]
async fn main() {
    tauri::async_runtime::set(tokio::runtime::Handle::current());

    env_logger::init();

    #[cfg(debug_assertions)]
    export_types();

    let project_dirs =
        ProjectDirs::from("com", "Adimaja", "launchpane").expect("Project dirs should be available");

    let data_dir = project_dirs.data_local_dir();

    info!("Using {} as data dir", data_dir.to_string_lossy());

    create_dir_all(data_dir).expect("Should be able to create data dir");

    set_current_dir(data_dir).expect("Should be able to access data dir");

    let db_client: PrismaClient = PrismaClient::_builder()
        .with_url("file:./app.db?connection_limit=1".into())
        .build()
        .await
        .expect("Database should be accessible");

    db_client
        ._migrate_deploy()
        .await
        .expect("Database migration should succeed");

    tauri::Builder::default()
        .setup(|app| {
            #[cfg(debug_assertions)]
            app.get_window("main").unwrap().open_devtools();

            let client_arc = Arc::new(db_client);

            let process_manager =
                ProcessManager::new(Arc::new(app.app_handle()), Arc::clone(&client_arc));

            let state = AppStateData {
                client: client_arc,
                process_manager: Arc::new(process_manager),
            };

            app.manage(state);

            Ok(())
        })
        .invoke_handler(generate_handler![
            get_commands,
            create_command,
            get_command,
            set_window_size,
            update_command,
            move_command_between,
            delete_command,
            get_platform_details,
            get_command_log_lines,
            get_newer_command_log_lines,
            get_older_command_log_lines,
            get_process_status,
            run_process,
            kill_process,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
