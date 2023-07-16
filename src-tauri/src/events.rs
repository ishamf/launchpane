use serde::Serialize;
use specta::Type;
use tauri::{AppHandle, Manager};

#[derive(Debug, Serialize, Type, Clone, Copy)]
pub enum AppEventPayload {
    CommandUpdateEvent(i32),
    CommandLogUpdateEvent(i32),
}

const EVENT_CHANNEL: &str = "change_event";

pub fn send_command_update_event(app: AppHandle, command_id: i32) -> Result<(), tauri::Error> {
    app.emit_all(EVENT_CHANNEL, AppEventPayload::CommandUpdateEvent(command_id))
}

pub fn send_command_log_update_event(app: AppHandle, command_id: i32) -> Result<(), tauri::Error> {
    app.emit_all(EVENT_CHANNEL, AppEventPayload::CommandLogUpdateEvent(command_id))
}
