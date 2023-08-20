use clap::Parser;
use clap::ValueEnum;
use config::{ConfigError, ValueKind};
use serde_derive::Deserialize;
use std::fmt;
use timetracker_core::settings::new_core_settings;
use timetracker_core::settings::CoreSettings;

/// Determines the formatting used for dates/times.
#[derive(Debug, Copy, Clone, ValueEnum, Deserialize)]
pub enum DateTimeFormatSetting {
    /// Follows the ISO8601 standard.
    Iso,

    /// Follows common date-time conventions in the USA.
    UsaMonthDayYear,

    /// Follows user's preferences for local date/time formating
    /// rules.
    Locale,
}

impl fmt::Display for DateTimeFormatSetting {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DateTimeFormatSetting::Locale => write!(f, "Locale"),
            DateTimeFormatSetting::Iso => write!(f, "Iso"),
            DateTimeFormatSetting::UsaMonthDayYear => write!(f, "UsaMonthDayYear"),
        }
    }
}

impl From<DateTimeFormatSetting> for ValueKind {
    fn from(value: DateTimeFormatSetting) -> Self {
        ValueKind::String(format!("{}", value).to_string())
    }
}

/// Determines the formatting used for durations.
#[derive(Debug, Copy, Clone, ValueEnum, Deserialize)]
pub enum DurationFormatSetting {
    /// Display exact hours and minutes.
    HoursMinutes,

    /// Hours as decimal number rounded to 6 minute increments.
    DecimalHours,
}

impl fmt::Display for DurationFormatSetting {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DurationFormatSetting::HoursMinutes => write!(f, "HoursMinutes"),
            DurationFormatSetting::DecimalHours => write!(f, "DecimalHours"),
        }
    }
}

impl From<DurationFormatSetting> for ValueKind {
    fn from(value: DurationFormatSetting) -> Self {
        ValueKind::String(format!("{}", value).to_string())
    }
}

#[derive(Parser, Debug)]
#[clap(author = "David Cattermole, Copyright 2023", version, about)]
pub struct CommandArguments {
    /// Relative week number. '0' is the current week, '-1' is the
    /// previous week, etc.
    #[clap(short = 'w', long, value_parser, default_value_t = 0)]
    relative_week: i32,

    /// How should dates/times be displayed?
    #[clap(long, value_enum)]
    format_datetime: Option<DateTimeFormatSetting>,

    /// How should duration be displayed?
    #[clap(long, value_enum)]
    format_duration: Option<DurationFormatSetting>,

    /// Display summary of week time. How many hours has been spent
    /// during the week?
    #[clap(long, value_parser)]
    display_week: Option<bool>,

    /// Display summary of weekday time. How many hours has been spent
    /// on each weekday?
    #[clap(long, value_parser)]
    display_weekday: Option<bool>,

    /// Display summary of week per-task time. How much time has been
    /// spent on each task, for the entire week?
    #[clap(long, value_parser)]
    display_week_task: Option<bool>,

    /// Display summary of weekday per-task time. How much time has
    /// been spent on each task, for each weekday?
    #[clap(long, value_parser)]
    display_weekday_task: Option<bool>,

    /// Display summary of week per-software time. How long has each
    /// software command been used?
    #[clap(long, value_parser)]
    display_week_software: Option<bool>,

    /// Override the directory to search for the database file.
    #[clap(long, value_parser)]
    database_dir: Option<String>,

    /// Override the name of the database file to open.
    #[clap(long, value_parser)]
    database_file_name: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct PrintSettings {
    pub relative_week: i32,
    pub format_datetime: DateTimeFormatSetting,
    pub format_duration: DurationFormatSetting,
    pub display_week: bool,
    pub display_weekday: bool,
    pub display_week_task: bool,
    pub display_weekday_task: bool,
    pub display_week_software: bool,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct AppSettings {
    pub core: CoreSettings,
    pub print: PrintSettings,
}

impl AppSettings {
    pub fn new(arguments: CommandArguments) -> Result<Self, ConfigError> {
        let mut builder = new_core_settings(arguments.database_dir, arguments.database_file_name)?;

        builder = builder
            .set_default("print.relative_week", 0)?
            .set_default("print.format_datetime", "Locale")?
            .set_default("print.format_duration", "HoursMinutes")?
            .set_default("print.display_week", true)?
            .set_default("print.display_weekday", true)?
            .set_default("print.display_week_task", true)?
            .set_default("print.display_weekday_task", false)?
            .set_default("print.display_week_software", false)?;

        // Use command line 'arguments' to override the default
        // values. These will always override any configuration file
        // or environment variable.
        builder = builder
            .set_override("print.relative_week", arguments.relative_week)?
            .set_override_option("print.display_week", arguments.display_week)?
            .set_override_option("print.display_weekday", arguments.display_weekday)?
            .set_override_option("print.display_week_task", arguments.display_week_task)?
            .set_override_option("print.display_weekday_task", arguments.display_weekday_task)?
            .set_override_option(
                "print.display_week_software",
                arguments.display_week_software,
            )?
            .set_override_option("print.format_datetime", arguments.format_datetime)?
            .set_override_option("print.format_duration", arguments.format_duration)?;

        let settings = builder.build()?;
        settings.try_deserialize()
    }
}
