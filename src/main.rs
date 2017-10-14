
extern crate env_logger;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate toml;

extern crate gtk;

mod config;
mod errors;
mod logging;
mod model;
mod ui;

use gtk::prelude::*;
use gtk::{Window, WindowType};

fn main() {
    logging::init_logger();
    debug!("hello, world");

    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }
    let conf = config::load_config().expect("Error loading config");
    let window = Window::new(WindowType::Toplevel);
    window.set_title("mpkdb");
    window.set_default_size(350, 70);
    ui::open::init_open_file_ui(&window);
    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });
    gtk::main();
}