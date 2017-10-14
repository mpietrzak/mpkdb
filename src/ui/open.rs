
//! Part of the UI to locate and open files.

use gtk::prelude::*;
use gtk;

pub fn init_open_file_ui(window: &gtk::Window) {
    let main_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let label = gtk::Label::new("hi");
    let entry = gtk::Entry::new();
    let open_btn = gtk::Button::new_with_label("Open");
    main_box.add(&label);
    main_box.add(&entry);
    main_box.add(&open_btn);
    window.add(&main_box);
    window.show_all();
    debug!("init_open_file_ui: Done");
}

