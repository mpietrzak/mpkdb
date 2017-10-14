
//! Part of the UI to locate and open files.

use gtk::prelude::*;
use gtk;

pub fn init_open_file_ui<F: Fn(String) + 'static>(window: &gtk::Window, callback: F) {
    let file_box = {
        let current_file_label = gtk::Label::new("");
        let button_choose_file = gtk::Button::new_with_label("Choose...");
        let b = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        b.add(&current_file_label);
        b.add(&button_choose_file);
        b.set_child_packing(&current_file_label, true, true, 0, gtk::PackType::Start);
        b
    };
    let main_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let entry = {
        let e = gtk::Entry::new();
        e.set_visibility(false);
        e
    };
    let open_btn = gtk::Button::new_with_label("Open");
    open_btn.connect_clicked(move |_| {
        callback(String::new())
    });
    main_box.add(&file_box);
    main_box.add(&entry);
    main_box.add(&open_btn);
    window.add(&main_box);
    window.show_all();
    debug!("init_open_file_ui: Done");
}

