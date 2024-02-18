use crate::constants;
use crate::constants::DATETIME_FORMAT_ISO_ID;
use crate::constants::DATETIME_FORMAT_ISO_LABEL;
use crate::constants::DATETIME_FORMAT_LOCALE_ID;
use crate::constants::DATETIME_FORMAT_LOCALE_LABEL;
use crate::constants::DATETIME_FORMAT_USA_MONTH_DAY_YEAR_ID;
use crate::constants::DATETIME_FORMAT_USA_MONTH_DAY_YEAR_LABEL;
use crate::constants::DURATION_FORMAT_DECIMAL_HOURS_ID;
use crate::constants::DURATION_FORMAT_DECIMAL_HOURS_LABEL;
use crate::constants::DURATION_FORMAT_HOURS_MINUTES_ID;
use crate::constants::DURATION_FORMAT_HOURS_MINUTES_LABEL;
use crate::constants::DURATION_FORMAT_HOURS_MINUTES_SECONDS_ID;
use crate::constants::DURATION_FORMAT_HOURS_MINUTES_SECONDS_LABEL;
use crate::settings::DisplayAppSettings;
use crate::utils::datetime_format_as_id;
use crate::utils::duration_format_as_id;
use crate::utils::get_absolute_week_start_end;
use crate::utils::id_as_datetime_format;
use crate::utils::id_as_duration_format;

use anyhow::Result;
use chrono::Datelike;
use gtk::glib::clone;
use gtk::prelude::*;
use gtk::{
    Application, ApplicationWindow, Box, Builder, ComboBoxText, Label, SpinButton, Statusbar,
    TextBuffer, TextView, ToggleButton,
};
use log::warn;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::SystemTime;

use timetracker_core::filesystem::get_database_file_path;
use timetracker_core::format::format_date;
use timetracker_core::settings::RECORD_INTERVAL_SECONDS;
use timetracker_core::storage::Storage;
use timetracker_print_lib::aggregate::get_map_keys_sorted_strings;
use timetracker_print_lib::datetime::DateTimeLocalPair;
use timetracker_print_lib::preset::create_presets;
use timetracker_print_lib::preset::generate_presets;

/// What state is a Preset in? A user can toggle the Preset on/off.
#[derive(Debug, Copy, Clone)]
pub enum PresetState {
    /// The Preset is enabled and able to be used.
    Enable,
    /// The Preset is disabled and cannot or should not be used.
    Disable,
}

pub struct GlobalState {
    settings: DisplayAppSettings,
    all_preset_names: Vec<String>,
    preset_states: HashMap<String, PresetState>,
    window: Option<ApplicationWindow>,
    status_bar: Option<Statusbar>,
    week_number_spin_button: Option<SpinButton>,
    format_date_time_combo_box: Option<ComboBoxText>,
    format_duration_combo_box: Option<ComboBoxText>,
    date_range_label: Option<Label>,
    preset_buttons_layout: Option<Box>,
    text_view: Option<TextView>,
    week_number: u32,
    text_buffer: TextBuffer,
}

pub type GlobalStateRcRefCell = Rc<RefCell<GlobalState>>;

impl GlobalState {
    pub fn new_with_settings(settings: DisplayAppSettings) -> GlobalState {
        let text_buffer = TextBuffer::builder().build();

        let mut preset_states = HashMap::new();
        for preset_name in &settings.print.display_presets {
            preset_states.insert(preset_name.clone(), PresetState::Enable);
        }

        // Add the additional preset names (not in the
        // 'display_presets') to the end of the displayed list,
        // sorted.
        let mut other_preset_names = Vec::new();
        for preset_name in settings.print.presets.keys() {
            let is_display_preset = settings
                .print
                .display_presets
                .iter()
                .any(|x| x.eq(preset_name));
            if !is_display_preset {
                other_preset_names.push(preset_name);
            }
        }
        other_preset_names.sort_unstable();

        let mut all_preset_names = settings.print.display_presets.clone();
        for preset_name in other_preset_names {
            all_preset_names.push(preset_name.clone());
            preset_states.insert(preset_name.clone(), PresetState::Disable);
        }

        GlobalState {
            settings: settings,
            all_preset_names: all_preset_names,
            preset_states: preset_states,
            window: None,
            status_bar: None,
            week_number_spin_button: None,
            format_date_time_combo_box: None,
            format_duration_combo_box: None,
            date_range_label: None,
            preset_buttons_layout: None,
            text_view: None,
            week_number: 1,
            text_buffer: text_buffer,
        }
    }
}

fn generate_text(
    week_datetime_pair: DateTimeLocalPair,
    settings: &DisplayAppSettings,
) -> Result<String> {
    let database_file_path = get_database_file_path(
        &settings.core.database_dir,
        &settings.core.database_file_name,
    );
    if !database_file_path.is_some() {
        warn!(
            "Database file {:?} not found in {:?}",
            &settings.core.database_file_name, &settings.core.database_dir
        );
    }

    let mut storage = Storage::open_as_read_only(
        &database_file_path.expect("Database file path should be valid"),
        RECORD_INTERVAL_SECONDS,
    )?;

    let (presets, missing_preset_names) = create_presets(
        settings.print.time_scale,
        settings.print.format_datetime,
        settings.print.format_duration,
        settings.print.time_block_unit,
        settings.print.bar_graph_character_num_width,
        settings.print.use_color,
        &settings.core.environment_variables.names,
        &settings.print.display_presets,
        &settings.print.presets,
    )?;

    // TODO: Fetch all the Storage entries we will need for the full
    // time range, then pass that fetched data to all the
    // functions. (this assumes that fetching data from the database
    // is likely the slowest runtime)
    let (week_start_datetime, week_end_datetime) = week_datetime_pair;
    let week_start_of_time = week_start_datetime.timestamp() as u64;
    let week_end_of_time = week_end_datetime.timestamp() as u64;
    let week_entries = storage.read_entries(week_start_of_time, week_end_of_time)?;

    // TODO: Stop using color in the text output.
    let lines = generate_presets(&presets, &week_entries)?;
    let all_lines_text = lines.join("\n");

    if !missing_preset_names.is_empty() {
        let all_preset_names = get_map_keys_sorted_strings(&settings.print.presets.keys());
        warn!(
            "Preset names {:?} are invalid. possible preset names are: {:?}",
            missing_preset_names, all_preset_names,
        );
    }

    Ok(all_lines_text)
}

fn update_date_range_label(
    date_range_label: &Label,
    week_datetime_pair: DateTimeLocalPair,
    settings: &DisplayAppSettings,
) -> Result<()> {
    let date_range_string = format!(
        "from {} to {}",
        format_date(week_datetime_pair.0, settings.print.format_datetime),
        format_date(week_datetime_pair.1, settings.print.format_datetime),
    )
    .to_string();
    date_range_label.set_text(&date_range_string);

    Ok(())
}

fn update_text_view(
    week_datetime_pair: DateTimeLocalPair,
    status_bar: &Statusbar,
    text_buffer: &TextBuffer,
    settings: &DisplayAppSettings,
) -> Result<()> {
    let context_id = status_bar.context_id("update_text_view");

    let msg = format!(
        "Generating data from {} to {}...",
        format_date(week_datetime_pair.0, settings.print.format_datetime),
        format_date(week_datetime_pair.1, settings.print.format_datetime),
    )
    .to_string();
    status_bar.push(context_id, &msg);

    let now = SystemTime::now();
    let text = generate_text(week_datetime_pair, settings)?;
    text_buffer.set_text(&text);
    let duration = now.elapsed()?.as_secs_f32();

    let msg = format!(
        "Generated data for {} to {} (took {:.4} seconds)",
        format_date(week_datetime_pair.0, settings.print.format_datetime),
        format_date(week_datetime_pair.1, settings.print.format_datetime),
        duration
    );
    status_bar.push(context_id, &msg);

    Ok(())
}

fn week_number_changed(widget: &SpinButton, global_state: GlobalStateRcRefCell) -> Result<()> {
    let mut borrowed_state = global_state.borrow_mut();

    let status_bar = borrowed_state.status_bar.as_ref().unwrap();
    let context_id = status_bar.context_id("week_number_changed");
    status_bar.push(context_id, "week_number_changed");

    let week_number: u32 = widget.value_as_int().try_into().unwrap();
    let week_datetime_pair = get_absolute_week_start_end(week_number)?;

    // Update label text with start and end date formatted as user
    // wants it (requires shared settings).
    let date_range_label = borrowed_state.date_range_label.as_ref().unwrap();
    update_date_range_label(
        date_range_label,
        week_datetime_pair,
        &borrowed_state.settings,
    )?;

    // Fetch the database entries and generate the text buffer again.
    update_text_view(
        week_datetime_pair,
        &status_bar,
        &borrowed_state.text_buffer,
        &borrowed_state.settings,
    )?;

    // Update the status bar with text saying ???.

    borrowed_state.week_number = week_number;

    Ok(())
}

fn format_date_time_changed(
    widget: &ComboBoxText,
    global_state: GlobalStateRcRefCell,
) -> Result<()> {
    let mut borrowed_state = global_state.borrow_mut();

    let active_id = widget.active_id();
    match id_as_datetime_format(active_id.as_ref()) {
        Some(value) => borrowed_state.settings.print.format_datetime = value,
        None => (),
    }

    let status_bar = borrowed_state.status_bar.as_ref().unwrap();
    let context_id = status_bar.context_id("format_date_time_changed");
    status_bar.push(context_id, "format_date_time_changed");

    let week_number: u32 = borrowed_state.week_number;
    let week_datetime_pair = get_absolute_week_start_end(week_number)?;

    let date_range_label = borrowed_state.date_range_label.as_ref().unwrap();
    update_date_range_label(
        date_range_label,
        week_datetime_pair,
        &borrowed_state.settings,
    )?;

    update_text_view(
        week_datetime_pair,
        &status_bar,
        &borrowed_state.text_buffer,
        &borrowed_state.settings,
    )?;

    borrowed_state.week_number = week_number;

    Ok(())
}

fn format_duration_changed(
    widget: &ComboBoxText,
    global_state: GlobalStateRcRefCell,
) -> Result<()> {
    let mut borrowed_state = global_state.borrow_mut();

    let active_id = widget.active_id();
    match id_as_duration_format(active_id.as_ref()) {
        Some(value) => borrowed_state.settings.print.format_duration = value,
        None => (),
    }

    let status_bar = borrowed_state.status_bar.as_ref().unwrap();
    let context_id = status_bar.context_id("format_duration_changed");
    status_bar.push(context_id, "format_duration_changed");

    let week_number: u32 = borrowed_state.week_number;
    let week_datetime_pair = get_absolute_week_start_end(week_number)?;

    let date_range_label = borrowed_state.date_range_label.as_ref().unwrap();
    update_date_range_label(
        date_range_label,
        week_datetime_pair,
        &borrowed_state.settings,
    )?;

    update_text_view(
        week_datetime_pair,
        &status_bar,
        &borrowed_state.text_buffer,
        &borrowed_state.settings,
    )?;

    borrowed_state.week_number = week_number;

    Ok(())
}

fn window_startup(_window: &ApplicationWindow, global_state: GlobalStateRcRefCell) -> Result<()> {
    let borrowed_state = global_state.borrow_mut();

    let status_bar = borrowed_state.status_bar.as_ref().unwrap();
    let context_id = status_bar.context_id("window_startup");
    status_bar.push(context_id, "window_startup");

    let week_datetime_pair = get_absolute_week_start_end(borrowed_state.week_number)?;

    let date_range_label = borrowed_state.date_range_label.as_ref().unwrap();
    update_date_range_label(
        date_range_label,
        week_datetime_pair,
        &borrowed_state.settings,
    )?;

    update_text_view(
        week_datetime_pair,
        &status_bar,
        &borrowed_state.text_buffer,
        &borrowed_state.settings,
    )?;

    Ok(())
}

/// When one of the preset buttons is toggled.
fn preset_toggle_clicked(
    _widget: &ToggleButton,
    preset_name: String,
    global_state: GlobalStateRcRefCell,
) -> Result<()> {
    let mut borrowed_state = global_state.borrow_mut();

    let toggled_state = match borrowed_state.preset_states.get(&preset_name) {
        Some(PresetState::Enable) => PresetState::Disable,
        Some(PresetState::Disable) => PresetState::Enable,
        None => PresetState::Disable,
    };
    borrowed_state
        .preset_states
        .insert(preset_name, toggled_state);

    borrowed_state.settings.print.display_presets.clear();
    for name in borrowed_state.all_preset_names.clone() {
        match borrowed_state.preset_states.get(&name) {
            Some(PresetState::Enable) => borrowed_state.settings.print.display_presets.push(name),
            _ => (),
        };
    }

    let week_datetime_pair = get_absolute_week_start_end(borrowed_state.week_number)?;
    let status_bar = borrowed_state.status_bar.as_ref().unwrap();
    update_text_view(
        week_datetime_pair,
        &status_bar,
        &borrowed_state.text_buffer,
        &borrowed_state.settings,
    )?;

    Ok(())
}

/// Build a button for each preset, so each preset can be toggled
/// on/off.
fn build_preset_buttons(
    layout_widget: &Box,
    global_state: GlobalStateRcRefCell,
    preset_names: &[String],
    preset_states: &HashMap<String, PresetState>,
) {
    for preset_name in preset_names {
        let preset_name = preset_name.clone();
        let enabled = match preset_states.get(&preset_name) {
            Some(PresetState::Enable) => true,
            _ => false,
        };

        let toggle_button = ToggleButton::with_label(&preset_name);
        toggle_button.set_active(enabled);

        toggle_button.connect_clicked(clone!(
            @strong global_state => move |widget| {
                preset_toggle_clicked(widget, preset_name.clone(), global_state.clone()).unwrap()
        }));

        layout_widget.add(&toggle_button);
    }
}

/// Create the window, and all the widgets in the window.
fn construct_window(week_number: u32, global_state: GlobalStateRcRefCell) -> ApplicationWindow {
    let mut borrowed_state = global_state.borrow_mut();

    let builder = Builder::from_string(constants::MAIN_WINDOW_GLADE);

    borrowed_state.status_bar = Some(
        builder
            .object("status_bar")
            .expect("Couldn't get 'status_bar'."),
    );
    let status_bar = borrowed_state.status_bar.as_ref().unwrap();

    let context_id = status_bar.context_id("build_ui");
    status_bar.push(context_id, "Building UI...");

    borrowed_state.week_number_spin_button = Some(
        builder
            .object("week_number_spin_button")
            .expect("Couldn't get 'week_number_spin_button' widget."),
    );
    let week_number_spin_button = borrowed_state.week_number_spin_button.as_ref().unwrap();
    week_number_spin_button.set_value(week_number as f64);
    borrowed_state.week_number = week_number;

    borrowed_state.text_view = Some(
        builder
            .object("text_view")
            .expect("Couldn't get 'text_view'."),
    );
    let text_view = borrowed_state.text_view.as_ref().unwrap();
    text_view.set_buffer(Some(&borrowed_state.text_buffer));

    borrowed_state.preset_buttons_layout = Some(
        builder
            .object("preset_buttons_layout")
            .expect("Couldn't get 'preset_button_layout' widget."),
    );
    let preset_buttons_layout = borrowed_state.preset_buttons_layout.as_ref().unwrap();
    build_preset_buttons(
        &preset_buttons_layout,
        global_state.clone(),
        &borrowed_state.all_preset_names,
        &borrowed_state.preset_states,
    );

    borrowed_state.format_date_time_combo_box = Some(
        builder
            .object("format_date_time_combo_box")
            .expect("Couldn't get 'format_date_time_combo_box'."),
    );
    let format_date_time_combo_box = borrowed_state.format_date_time_combo_box.as_ref().unwrap();
    format_date_time_combo_box.append(Some(DATETIME_FORMAT_ISO_ID), &DATETIME_FORMAT_ISO_LABEL);
    format_date_time_combo_box.append(
        Some(DATETIME_FORMAT_USA_MONTH_DAY_YEAR_ID),
        DATETIME_FORMAT_USA_MONTH_DAY_YEAR_LABEL,
    );
    format_date_time_combo_box.append(
        Some(DATETIME_FORMAT_LOCALE_ID),
        &DATETIME_FORMAT_LOCALE_LABEL,
    );
    let datetime_format_id = datetime_format_as_id(borrowed_state.settings.print.format_datetime);
    format_date_time_combo_box.set_active_id(Some(datetime_format_id));

    borrowed_state.format_duration_combo_box = Some(
        builder
            .object("format_duration_combo_box")
            .expect("Couldn't get 'format_duration_combo_box'."),
    );
    let format_duration_combo_box = borrowed_state.format_duration_combo_box.as_ref().unwrap();
    format_duration_combo_box.append(
        Some(DURATION_FORMAT_HOURS_MINUTES_ID),
        DURATION_FORMAT_HOURS_MINUTES_LABEL,
    );
    format_duration_combo_box.append(
        Some(DURATION_FORMAT_HOURS_MINUTES_SECONDS_ID),
        DURATION_FORMAT_HOURS_MINUTES_SECONDS_LABEL,
    );
    format_duration_combo_box.append(
        Some(DURATION_FORMAT_DECIMAL_HOURS_ID),
        DURATION_FORMAT_DECIMAL_HOURS_LABEL,
    );
    let duration_format_id = duration_format_as_id(borrowed_state.settings.print.format_duration);
    format_duration_combo_box.set_active_id(Some(duration_format_id));

    borrowed_state.date_range_label = Some(
        builder
            .object("date_range_label")
            .expect("Couldn't get 'date_range_label'."),
    );

    borrowed_state.window = Some(
        builder
            .object("window")
            .expect("Couldn't get 'window' widget."),
    );
    let window = borrowed_state.window.as_ref().unwrap();
    window.set_title(constants::WINDOW_TITLE);
    window.set_default_width(constants::WINDOW_DEFAULT_WIDTH);
    window.set_default_height(constants::WINDOW_DEFAULT_HEIGHT);
    window.show_all();

    window.clone()
}

/// Adds callbacks (known as "signals") to various events in GTK and
/// widgets.
fn setup_signals(global_state: GlobalStateRcRefCell) {
    let borrowed_state = global_state.borrow_mut();

    let week_number_spin_button = borrowed_state.week_number_spin_button.as_ref().unwrap();
    week_number_spin_button.connect_value_changed(clone!(
    @strong global_state =>
            move |widget| {
                week_number_changed(&widget, global_state.clone()).unwrap()
            }));

    let format_date_time_combo_box = borrowed_state.format_date_time_combo_box.as_ref().unwrap();
    format_date_time_combo_box.connect_changed(clone!(
    @strong global_state =>
        move |widget| {
            format_date_time_changed(&widget, global_state.clone()).unwrap()
        }));

    let format_duration_combo_box = borrowed_state.format_duration_combo_box.as_ref().unwrap();
    format_duration_combo_box.connect_changed(clone!(
    @strong global_state =>
        move |widget| {
            format_duration_changed(&widget, global_state.clone()).unwrap()
        }));
}

pub fn build_ui(app: &Application, global_state: GlobalStateRcRefCell) {
    // Get the current week as the default value.
    let today_local_timezone = chrono::Local::now();
    let today_week = today_local_timezone.iso_week().week();

    let window = construct_window(today_week, global_state.clone());
    window.set_application(Some(app));

    setup_signals(global_state.clone());

    window_startup(&window, global_state.clone()).unwrap();
}
