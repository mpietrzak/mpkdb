
extern crate crypto;
extern crate env_logger;
extern crate gtk;
#[macro_use]
extern crate log;
#[macro_use]
extern crate nom;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate time;
extern crate toml;
extern crate uuid;

mod config;
mod db;
mod errors;
mod logging;
mod model;
mod ui;

use std::sync::Arc;
use std::sync::RwLock;

use gtk::{Window, WindowType};
use gtk::prelude::*;

/// Main entry.
/// TODO: Split in smaller fns, also split app into binary and lib.
fn main() {
    logging::env_logger_init();
    debug!("hello, world");
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }
    let _conf = config::load_config().expect("Error loading config");
    let state = Arc::new(RwLock::new(model::State { db: None }));
    let state_clone = Arc::clone(&state);
    let window = Window::new(WindowType::Toplevel);
    let window_clone = window.clone(); // moved to closure
    window.set_title("mpkdb");
    ui::open::init_open_file_ui(&window, None, move |s| {
        let password = "test";
        let db = match db::kdb::db::open(&s, password) {
            Ok(db) => Arc::new(RwLock::new(db)),
            Err(e) => {
                error!("Failed to open DB: {}", e);
                // TODO: UI
                return;
            }
        };
        // TODO: Is failure to lock something we should expect? Maybe I should use antidote?
        let mut state_guard = match state_clone.write() {
            Ok(g) => g,
            Err(e) => {
                // TODO: UI
                error!("Failed to lock state: {}", e);
                return;
            }
        };
        state_guard.db = Some(db);
        // DB opened successfully, show main UI.
        ui::main::init_main_ui(&window_clone);
    });
    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });
    gtk::main();
}
