use crate::settings::CommandArguments;
use crate::settings::PrintAppSettings;
use anyhow::bail;
use anyhow::Result;
use clap::Parser;
use log::{debug, warn};
use std::time::SystemTime;
use timetracker_core::filesystem::get_database_file_path;
use timetracker_core::format::format_datetime;
use timetracker_core::settings::RECORD_INTERVAL_SECONDS;
use timetracker_core::storage::Storage;
use timetracker_print_lib::aggregate::get_map_keys_sorted_strings;
use timetracker_print_lib::preset::create_presets;
use timetracker_print_lib::preset::generate_presets;
use timetracker_print_lib::print::get_relative_week_start_end;

mod settings;

fn print_presets(args: &CommandArguments, settings: &PrintAppSettings) -> Result<()> {
    let database_file_path = get_database_file_path(
        &settings.core.database_dir,
        &settings.core.database_file_name,
    );
    if database_file_path.is_some() {
        println!(
            "Database file path: {}",
            database_file_path.as_ref().unwrap().display()
        );
    } else {
        warn!(
            "Database file {:?} not found in {:?}",
            &settings.core.database_file_name, &settings.core.database_dir
        );
    }

    let mut storage = Storage::open_as_read_only(
        &database_file_path.expect("Database file path should be valid"),
        RECORD_INTERVAL_SECONDS,
    )?;

    let relative_week = if args.last_week {
        -1
    } else {
        args.relative_week
    };

    // 'relative_week' is added to the week number to find. A value of
    // '-1' will get the previous week, a value of '0' will get the
    // current week, and a value of '1' will get the next week (which
    // shouldn't really give any results, so it's probably pointless).
    let week_datetime_pair = get_relative_week_start_end(relative_week)?;
    println!(
        "Gathering data from {} to {}.",
        format_datetime(week_datetime_pair.0, settings.print.format_datetime),
        format_datetime(week_datetime_pair.1, settings.print.format_datetime),
    );
    println!("");

    let (presets, missing_preset_names) = create_presets(
        settings.print.time_scale,
        settings.print.format_datetime,
        settings.print.format_duration,
        settings.print.time_block_unit,
        settings.print.bar_graph_character_num_width,
        &settings.core.environment_variables.names,
        &settings.print.display_presets,
        &settings.print.presets,
    )?;

    let lines = generate_presets(&presets, &mut storage, week_datetime_pair)?;
    for line in &lines {
        println!("{}", line);
    }

    if !missing_preset_names.is_empty() {
        let all_preset_names = get_map_keys_sorted_strings(&settings.print.presets.keys());
        warn!(
            "Preset names {:?} are invalid. possible preset names are: {:?}",
            missing_preset_names, all_preset_names,
        );
    }

    Ok(())
}

fn list_presets(settings: &PrintAppSettings) -> Result<()> {
    let all_preset_names = get_map_keys_sorted_strings(&settings.print.presets.keys());
    for preset_name in &all_preset_names {
        println!("{}", preset_name);
    }

    Ok(())
}

fn main() -> Result<()> {
    let env = env_logger::Env::default()
        .filter_or("TIMETRACKER_LOG", "warn")
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

    match &args.list_presets {
        true => list_presets(&settings)?,
        false => print_presets(&args, &settings)?,
    };

    let duration = now.elapsed()?.as_secs_f32();
    debug!("Time taken: {:.2} seconds", duration);

    Ok(())
}
