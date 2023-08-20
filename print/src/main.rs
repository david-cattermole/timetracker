use crate::print::get_relative_week_start_end;
use crate::print::print_entries;
use crate::print::print_preset;
use crate::settings::CommandArguments;
use crate::settings::PrintAppSettings;
use crate::utils::DateTimeLocalPair;
use anyhow::bail;
use anyhow::Result;
use clap::Parser;
use log::debug;
use std::time::SystemTime;
use timetracker_core::filesystem::get_database_file_path;
use timetracker_core::format::FirstDayOfWeek;
use timetracker_core::format::TimeDuration;
use timetracker_core::storage::Storage;

mod print;
mod settings;
mod utils;

fn main() -> Result<()> {
    let env = env_logger::Env::default()
        .filter("TIMETRACKER_LOG")
        .write_style("TIMETRACKER_LOG_STYLE");
    env_logger::init_from_env(env);

    let args = CommandArguments::parse();

    let settings = PrintAppSettings::new(args);
    if !settings.is_ok() {
        bail!("Settings are invalid: {:?}", settings);
    }
    let settings = settings.unwrap();
    debug!("Settings validated: {:#?}", settings);

    let now = SystemTime::now();

    let database_file_path = get_database_file_path(
        &settings.core.database_dir,
        &settings.core.database_file_name,
    );

    let mut storage = Storage::open_as_read_only(
        &database_file_path.expect("Database file path should be valid"),
        settings.core.record_interval_seconds,
    )?;

    // TODO: Write a more general print function that allows users to
    // give an arbitrary list of environment variable names, and the
    // function will query and store the data and present it to the
    // user.

    // 'relative_week_index' is added to the week number to find. A
    // value of '-1' will get the previous week, a value of '0' will
    // get the current week, and a value of '1' will get the next week
    // (which shouldn't really give any results, so it's probably
    // pointless).
    let week_datetime_pair = get_relative_week_start_end(settings.print.relative_week);

    let time_duration = TimeDuration::FullWeek;
    let first_day_of_week = FirstDayOfWeek::Monday;

    print_entries(
        &mut storage,
        settings.print.relative_week,
        settings.print.format_datetime,
        settings.print.format_duration,
        settings.print.display_week,
        settings.print.display_weekday,
        settings.print.display_week_task,
        settings.print.display_weekday_task,
        settings.print.display_week_software,
    )?;

    print_preset(
        &mut storage,
        week_datetime_pair,
        // filter_executable: bool,
        // filter_env_vars: Vec<String>,
        time_duration,
        settings.print.format_datetime,
        settings.print.format_duration,
        first_day_of_week,
        // output_stream: MyOutputStream
    )?;

    let duration = now.elapsed()?.as_secs_f32();
    debug!("Time taken: {:.1} seconds", duration);

    Ok(())
}
