//! RoboTrade - Desktop Trading Robot
//!
//! Ponto de entrada da aplicação.

#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

fn main() {
    robotrade_app_lib::run();
}
