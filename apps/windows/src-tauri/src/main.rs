#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    vpnbridge_windows_lib::run();
}
