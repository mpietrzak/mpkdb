
extern crate env_logger;
#[macro_use]
extern crate log;
#[macro_use]
extern crate nom;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate toml;
extern crate time;
extern crate gtk;

mod config;
mod db;
mod errors;
mod logging;
mod model;
mod ui;

use std::sync::Arc;
use std::sync::RwLock;

use gtk::prelude::*;
use gtk::{Window, WindowType};

fn main() {
    logging::env_logger_init();
    debug!("hello, world");
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }
    let _conf = config::load_config().expect("Error loading config");
    let state = Arc::new(RwLock::new(model::State{db: None}));
    let state_clone = Arc::clone(&state);
    let window = Window::new(WindowType::Toplevel);
    window.set_title("mpkdb");
    // window.set_default_size(350, 70);
    ui::open::init_open_file_ui(&window, None, move |s| {
        debug!("About to open \"{}\"...", s);
        let db = match db::kdb::db::open(&s) {
            Ok(db) => Arc::new(RwLock::new(db)),
            Err(e) => {
                error!("Failed to open DB: {}", e);
                return;
            }
        };
        let mut state_guard = match state_clone.write() {
            Ok(g) => g,
            Err(e) => {
                error!("Failed to lock state: {}", e);
                return;
            }
        };
        state_guard.db = Some(db);
    });
    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });
    gtk::main();
}
