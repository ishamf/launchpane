// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[allow(warnings, unused)]
mod prisma;

mod errors;
mod events;

use std::{path::MAIN_SEPARATOR, sync::Arc, vec};

use errors::AppCommandError;
use events::{send_command_update_event, AppEventPayload};
use prisma::*;

use prisma_client_rust::{Direction, QueryError};
use serde::Serialize;
use specta::{collect_types, Type};
use tauri::{api::path::home_dir, generate_handler, AppHandle, LogicalSize, Manager, Size, Window};
use tauri_specta::ts;

type AppState<'a> = tauri::State<'a, Arc<AppStateData>>;

struct AppStateData {
    client: PrismaClient,
}

#[tauri::command]
#[specta::specta]
async fn get_commands(state: AppState<'_>) -> Result<Vec<command::Data>, QueryError> {
    state.client.command().find_many(vec![]).exec().await
}

#[tauri::command]
#[specta::specta]
async fn get_command(
    state: AppState<'_>,
    command_id: i32,
) -> Result<Option<command::Data>, QueryError> {
    state
        .client
        .command()
        .find_unique(command::id::equals(command_id))
        .exec()
        .await
}

#[tauri::command]
#[specta::specta]
async fn get_command_log_lines(
    state: AppState<'_>,
    command_id: i32,
) -> Result<Vec<command_log_line::Data>, QueryError> {
    let mut log_lines = state
        .client
        .command_log_line()
        .find_many(vec![command_log_line::command_id::equals(command_id)])
        .order_by(command_log_line::timestamp::order(Direction::Desc))
        .take(1000)
        .exec()
        .await?;

    log_lines.reverse();

    Ok(log_lines)
}

#[tauri::command]
#[specta::specta]
async fn get_newer_command_log_lines(
    state: AppState<'_>,
    command_id: i32,
    last_id: i32,
) -> Result<Vec<command_log_line::Data>, QueryError> {
    state
        .client
        .command_log_line()
        .find_many(vec![command_log_line::command_id::equals(command_id)])
        .order_by(command_log_line::timestamp::order(Direction::Asc))
        .cursor(command_log_line::id::equals(last_id))
        .skip(1)
        .take(1000)
        .exec()
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

    send_command_update_event(app, command_id)?;

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

    send_command_update_event(app, command_id)?;

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
        .command()
        .create(
            "".into(),
            home_dir().unwrap_or("".into()).to_string_lossy().into(),
            "".into(),
            vec![],
        )
        .exec()
        .await?;

    send_command_update_event(app, result.id)?;

    Ok(result)
}

#[tauri::command]
#[specta::specta]
fn set_window_size(window: Window, width: f64, height: f64) -> Result<(), tauri::Error> {
    window.set_size(Size::Logical(LogicalSize { width, height }))
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

#[tokio::main]
async fn main() {
    tauri::async_runtime::set(tokio::runtime::Handle::current());

    #[cfg(debug_assertions)]
    ts::export(
        collect_types![
            get_commands,
            create_command,
            get_command,
            set_window_size,
            update_command,
            delete_command,
            get_platform_details,
            get_command_log_lines,
            get_newer_command_log_lines,
        ],
        "../src/lib/bindings.ts",
    )
    .unwrap();

    #[cfg(debug_assertions)]
    {
        let app_event_type = specta::ts::export::<AppEventPayload>(&Default::default()).unwrap();

        std::fs::write("../src/lib/events.ts", app_event_type).unwrap();
    };

    let db_client: PrismaClient = PrismaClient::_builder()
        .with_url("file:./app.db".into())
        .build()
        .await
        .expect("Error when creating database client");

    db_client
        ._migrate_deploy()
        .await
        .expect("Error when deploying database changes");

    let state = AppStateData { client: db_client };

    tauri::Builder::default()
        .manage(Arc::new(state))
        .setup(|app| {
            #[cfg(debug_assertions)]
            app.get_window("main").unwrap().open_devtools();

            Ok(())
        })
        .invoke_handler(generate_handler![
            get_commands,
            create_command,
            get_command,
            set_window_size,
            update_command,
            delete_command,
            get_platform_details,
            get_command_log_lines,
            get_newer_command_log_lines
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
