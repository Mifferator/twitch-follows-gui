#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(dead_code)]

mod api;
mod models;

use models::Channel;

#[tauri::command]
async fn fetch_follows(username: String, app: tauri::AppHandle) -> Result<Vec<Channel>, String> {
    let client = reqwest::Client::new();
    api::fetch_follows(&client, &username, &app)
        .await
        .map_err(|e| e.to_string())
}

#[derive(serde::Serialize)]
struct ModStatusResult {
    user_is_mod: bool,
    channel_is_mod: bool,
}

#[tauri::command]
async fn check_mod_status(channel_id: String, user_id: String) -> Result<ModStatusResult, String> {
    let client = reqwest::Client::new();
    let (user_is_mod, channel_is_mod) = api::check_mod_status(&client, &channel_id, &user_id)
        .await
        .map_err(|e| e.to_string())?;
    Ok(ModStatusResult { user_is_mod, channel_is_mod })
}

#[tauri::command]
async fn fetch_user_avatar(login: String) -> Result<Option<String>, String> {
    let client = reqwest::Client::new();
    api::fetch_user_avatar(&client, &login)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn open_channel(login: String) {
    open::that(format!("https://twitch.tv/{login}")).ok();
}

#[tauri::command]
fn quit(app: tauri::AppHandle) {
    app.exit(0);
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![fetch_follows, fetch_user_avatar, check_mod_status, open_channel, quit])
        .run(tauri::generate_context!())
        .expect("error running tauri app");
}
