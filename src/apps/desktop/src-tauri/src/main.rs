#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    agent_hud_host::run_with_options(
        tauri::generate_context!(),
        None,
        agent_hud_host::HostOptions::installed(),
    );
}
