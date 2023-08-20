use crate::print::print_entries;
use crate::settings::AppSettings;
use crate::settings::CommandArguments;
use crate::settings::DateTimeFormatSetting;
use crate::settings::DurationFormatSetting;
use anyhow::bail;
use anyhow::Result;
use clap::Parser;
use log::debug;
use std::time::SystemTime;
use timetracker_core::filesystem::get_database_file_path;
use timetracker_core::format::DateTimeFormat;
use timetracker_core::format::DurationFormat;
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

    let settings = AppSettings::new(args);
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

    let print_format_datetime = match settings.print.format_datetime {
        DateTimeFormatSetting::UsaMonthDayYear => DateTimeFormat::UsaMonthDayYear,
        DateTimeFormatSetting::Iso => DateTimeFormat::Iso,
        DateTimeFormatSetting::Locale => DateTimeFormat::Locale,
    };

    let print_format_duration = match settings.print.format_duration {
        DurationFormatSetting::HoursMinutes => DurationFormat::HoursMinutes,
        DurationFormatSetting::DecimalHours => DurationFormat::DecimalHours,
    };

    let mut storage = Storage::open_as_read_only(
        &database_file_path.expect("Database file path should be valid"),
        settings.core.record_interval_seconds,
    )?;

    print_entries(
        &mut storage,
        settings.print.relative_week,
        print_format_datetime,
        print_format_duration,
        settings.print.display_week,
        settings.print.display_weekday,
        settings.print.display_week_task,
        settings.print.display_weekday_task,
        settings.print.display_week_software,
    )?;

    let duration = now.elapsed()?.as_secs_f32();
    debug!("Time taken: {:.1} seconds", duration);

    Ok(())
}
