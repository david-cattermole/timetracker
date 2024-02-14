use gtk::prelude::*;
use gtk::{ApplicationWindow, Builder};
use gtk::{Box, ComboBoxText, Label, SpinButton, Statusbar, TextView};

pub fn get_window(builder: &Builder) -> ApplicationWindow {
    builder
        .object("window")
        .expect("Couldn't get 'window' widget.")
}

pub fn get_preset_buttons_layout(builder: &Builder) -> Box {
    builder
        .object("preset_buttons_layout")
        .expect("Couldn't get 'preset_button_layout' widget.")
}

pub fn get_week_number_spin_button(builder: &Builder) -> SpinButton {
    builder
        .object("week_number_spin_button")
        .expect("Couldn't get 'week_number_spin_button' widget.")
}

pub fn get_format_date_time_combo_box(builder: &Builder) -> ComboBoxText {
    builder
        .object("format_date_time_combo_box")
        .expect("Couldn't get 'format_date_time_combo_box'.")
}

pub fn get_format_duration_combo_box(builder: &Builder) -> ComboBoxText {
    builder
        .object("format_duration_combo_box")
        .expect("Couldn't get 'format_duration_combo_box'.")
}

pub fn get_date_range_label(builder: &Builder) -> Label {
    builder
        .object("date_range_label")
        .expect("Couldn't get 'date_range_label'.")
}

pub fn get_text_view(builder: &Builder) -> TextView {
    builder
        .object("text_view")
        .expect("Couldn't get 'text_view'.")
}

pub fn get_status_bar(builder: &Builder) -> Statusbar {
    builder
        .object("status_bar")
        .expect("Couldn't get 'status_bar'.")
}
