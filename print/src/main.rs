use crate::aggregate::get_map_keys_sorted;
use crate::print::generate_preset_lines;
use crate::print::get_relative_week_start_end;
use crate::settings::CommandArguments;
use crate::settings::PrintAppSettings;
use crate::variable::Variable;
use anyhow::bail;
use anyhow::Result;
use clap::Parser;
use log::{debug, warn};
use std::time::SystemTime;
use timetracker_core::filesystem::get_database_file_path;
use timetracker_core::format::PrintType;
use timetracker_core::settings::PrintPresetSettings;
use timetracker_core::settings::RECORD_INTERVAL_SECONDS;
use timetracker_core::storage::Storage;

mod aggregate;
mod datetime;
mod print;
mod settings;
mod utils;
mod variable;

fn override_preset_value<T>(new_value: Option<T>, old_value: Option<T>) -> Option<T> {
    match new_value {
        Some(value) => Some(value),
        None => old_value,
    }
}

fn main() -> Result<()> {
    let env = env_logger::Env::default()
        .filter("TIMETRACKER_LOG")
        .write_style("TIMETRACKER_LOG_STYLE");
    env_logger::init_from_env(env);

    let args = CommandArguments::parse();

    let settings = PrintAppSettings::new(&args);
    if settings.is_err() {
        bail!("Settings are invalid: {:?}", settings);
    }
    let settings = settings?;
    debug!("Settings validated: {:#?}", settings);

    let now = SystemTime::now();

    let database_file_path = get_database_file_path(
        &settings.core.database_dir,
        &settings.core.database_file_name,
    );

    let mut storage = Storage::open_as_read_only(
        &database_file_path.expect("Database file path should be valid"),
        RECORD_INTERVAL_SECONDS,
    )?;

    // 'relative_week' is added to the week number to find. A value of
    // '-1' will get the previous week, a value of '0' will get the
    // current week, and a value of '1' will get the next week (which
    // shouldn't really give any results, so it's probably pointless).
    let week_datetime_pair = get_relative_week_start_end(args.relative_week)?;

    let mut core_variables = Vec::new();
    for name in &settings.core.environment_variables.names {
        let variable = Variable::VariableName(name.clone());
        core_variables.push(variable);
    }

    let core_preset = PrintPresetSettings::new(
        // The 'print_type' must be valid for the preset to be used,
        // but the core settings (intentionally) do not define any
        // default value - it must be defined by the user-created
        // preset.
        None,
        Some(settings.print.time_scale),
        Some(settings.print.format_datetime),
        Some(settings.print.format_duration),
        Some(settings.print.time_block_unit),
        Some(settings.print.bar_graph_character_num_width),
        Some(settings.core.environment_variables.names.clone()),
    );

    let mut missing_preset_names = Vec::new();
    let mut presets = Vec::new();
    for preset_name in settings.print.display_presets {
        let preset = match settings.print.presets.get(&preset_name) {
            Some(value) => {
                let print_type = override_preset_value(value.print_type, core_preset.print_type);
                let time_scale = override_preset_value(value.time_scale, core_preset.time_scale);
                let format_datetime =
                    override_preset_value(value.format_datetime, core_preset.format_datetime);
                let format_duration =
                    override_preset_value(value.format_duration, core_preset.format_duration);
                let time_block_unit =
                    override_preset_value(value.time_block_unit, core_preset.time_block_unit);
                let bar_graph_character_num_width = override_preset_value(
                    value.bar_graph_character_num_width,
                    core_preset.bar_graph_character_num_width,
                );

                PrintPresetSettings::new(
                    print_type,
                    time_scale,
                    format_datetime,
                    format_duration,
                    time_block_unit,
                    bar_graph_character_num_width,
                    Some(settings.core.environment_variables.names.clone()),
                )
            }
            None => {
                warn!("Preset name {:?} is unavailable.", preset_name);
                missing_preset_names.push(preset_name);
                core_preset.clone()
            }
        };

        presets.push(preset);
    }

    // let color = colored::Color::Red;
    let color = colored::Color::Green;

    let mut lines = Vec::new();
    for preset in presets {
        if preset.print_type.is_none() {
            continue;
        }
        let print_type = preset.print_type.unwrap();

        let preset_variables = match print_type {
            PrintType::Software => vec![Variable::Executable; 1],
            PrintType::Variables => {
                let mut variables = Vec::new();
                if let Some(variable_names) = preset.variable_names {
                    for name in variable_names {
                        let variable = Variable::VariableName(name);
                        variables.push(variable);
                    }
                }
                variables
            }
            _ => Vec::new(),
        };

        generate_preset_lines(
            &mut storage,
            &mut lines,
            week_datetime_pair,
            print_type,
            &preset_variables,
            preset.time_scale.unwrap(),
            preset.format_datetime.unwrap(),
            preset.format_duration.unwrap(),
            preset.time_block_unit.unwrap(),
            preset.bar_graph_character_num_width.unwrap(),
            color,
        )?;
    }
    for line in &lines {
        println!("{}", line);
    }

    if !missing_preset_names.is_empty() {
        let all_preset_names = get_map_keys_sorted(&settings.print.presets.keys());
        warn!(
            "Preset names {:?} are invalid. possible preset names are: {:?}",
            missing_preset_names, all_preset_names,
        );
    }

    let duration = now.elapsed()?.as_secs_f32();
    debug!("Time taken: {:.2} seconds", duration);

    Ok(())
}
