
use gtk::BinExt;
use gtk::BoxExt;
use gtk::ContainerExt;
use gtk::StatusbarExt;
use gtk::WidgetExt;
use gtk;

fn create_results_box() -> gtk::Box {
    let results_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let tmp = gtk::Label::new("hello");
    results_box.add(&tmp);
    return results_box;
}

/// Init main app view - with Password Database visible, searchable etc.
/// Param window is the top level main app window.
/// We do our best to clean the window and get it into usable state, laying
/// out widgets and controls, binding handlers etc.
pub fn init_main_ui(window: &gtk::Window) {
    {
        if let Some(ref c) = window.get_child() {
            window.remove(c);
        }
    }
    let main_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let search_field = gtk::Entry::new();
    main_box.add(&search_field);
    let results_box = create_results_box();
    main_box.add(&results_box);
    let status_bar = gtk::Statusbar::new();
    main_box.add(&status_bar);
    main_box.set_child_packing(&results_box, true, true, 0, gtk::PackType::Start);
    window.add(&main_box);
    window.show_all();
    let status_bar_context_id = status_bar.get_context_id("main");
    status_bar.push(status_bar_context_id, "hello, world");
}
