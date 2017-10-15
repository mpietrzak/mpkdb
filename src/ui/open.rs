
//! Part of the UI to locate and open files.

use gtk::prelude::*;
use gtk;

pub fn init_open_file_ui<F: Fn(String) + 'static>(window: &gtk::Window, old_filename: Option<&str>, callback: F) {
    let current_file_label = gtk::Label::new("");
    let open_btn = gtk::Button::new_with_label("Open");
    let file_box = {
        // This clones will be moved to closure...
        let current_file_label_clone = current_file_label.clone();
        let open_btn_clone = open_btn.clone();
        let button_choose_file = gtk::Button::new_with_label("Choose...");
        let parent = window.clone();
        button_choose_file.connect_clicked(move |x| {
            debug!("init_open_file_ui: Choose file button clicked: {:?}", x);
            let dialog = gtk::FileChooserDialog::new(None, Some(&parent), gtk::FileChooserAction::Open);
            dialog.add_buttons(&[
                               ("Cancel", gtk::ResponseType::Cancel.into()),
                               ("Open", gtk::ResponseType::Ok.into()),
            ]);
            let result = dialog.run();
            debug!("init_open_file_ui: Result of running file chooser dialog: {:?}", result);
            let filename = dialog.get_filename();
            dialog.destroy();
            debug!("init_open_file_ui: File: {:?}", filename);
            match filename {
                Some(filename) => {
                    current_file_label_clone.set_text(&filename.to_string_lossy());
                    open_btn_clone.set_sensitive(true);
                }
                None => {
                }
            }
        });
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
    if let Some(ref old_filename) = old_filename {
        current_file_label.set_text(old_filename);
    } else {
        open_btn.set_sensitive(false);
    }
    {
        let current_file_label_clone = current_file_label.clone();
        open_btn.connect_clicked(move |_| {
            match current_file_label_clone.get_text() {
                Some(p) => callback(p),
                None => {
                    warn!("No text in label, can't open file");
                }
            }
        });
    }
    main_box.add(&file_box);
    main_box.add(&entry);
    main_box.add(&open_btn);
    window.add(&main_box);
    window.show_all();
    debug!("init_open_file_ui: Done");
}

