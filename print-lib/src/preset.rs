use crate::datetime::DateTimeLocalPair;
use crate::print::generate_preset_lines;
use crate::variable::Variable;
use anyhow::Result;
use log::warn;
use std::collections::HashMap;
use timetracker_core::format::DateTimeFormat;
use timetracker_core::format::DurationFormat;
use timetracker_core::format::PrintType;
use timetracker_core::format::TimeBlockUnit;
use timetracker_core::format::TimeScale;
use timetracker_core::settings::PrintPresetSettings;
use timetracker_core::storage::Storage;

pub fn override_preset_value<T>(new_value: Option<T>, old_value: Option<T>) -> Option<T> {
    match new_value {
        Some(value) => Some(value),
        None => old_value,
    }
}

pub fn create_presets(
    default_time_scale: TimeScale,
    default_format_datetime: DateTimeFormat,
    default_format_duration: DurationFormat,
    default_time_block_unit: TimeBlockUnit,
    default_bar_graph_character_num_width: u8,
    environment_variables_names: &[String],
    display_presets: &Vec<String>,
    print_presets: &HashMap<String, PrintPresetSettings>,
) -> Result<(Vec<PrintPresetSettings>, Vec<String>)> {
    let core_preset = PrintPresetSettings::new(
        // The 'print_type' must be valid for the preset to be used,
        // but the core settings (intentionally) do not define any
        // default value - it must be defined by the user-created
        // preset.
        None,
        Some(default_time_scale),
        Some(default_format_datetime),
        Some(default_format_duration),
        Some(default_time_block_unit),
        Some(default_bar_graph_character_num_width),
        Some(environment_variables_names.to_vec()),
    );

    let mut missing_preset_names = Vec::new();
    let mut presets = Vec::new();
    for preset_name in display_presets {
        let preset = match print_presets.get(&preset_name.clone()) {
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
                    Some(environment_variables_names.to_vec()),
                )
            }
            None => {
                warn!("Preset name {:?} is unavailable.", preset_name);
                missing_preset_names.push(preset_name.clone());
                core_preset.clone()
            }
        };

        presets.push(preset);
    }

    Ok((presets, missing_preset_names))
}

pub fn generate_presets(
    presets: &Vec<PrintPresetSettings>,
    storage: &mut Storage,
    week_datetime_pair: DateTimeLocalPair,
) -> Result<Vec<String>> {
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
                if let Some(variable_names) = &preset.variable_names {
                    for name in variable_names {
                        let variable = Variable::VariableName(name.clone());
                        variables.push(variable);
                    }
                }
                variables
            }
            _ => Vec::new(),
        };

        generate_preset_lines(
            storage,
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

    Ok(lines)
}
