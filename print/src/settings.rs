use clap::Parser;
use config::ConfigError;
use serde_derive::Deserialize;
use timetracker_core::format::DateTimeFormat;
use timetracker_core::format::DurationFormat;
use timetracker_core::settings::new_core_settings;
use timetracker_core::settings::new_print_settings;
use timetracker_core::settings::CoreSettings;
use timetracker_core::settings::PrintSettings;

#[derive(Parser, Debug)]
#[clap(author = "David Cattermole, Copyright 2023", version, about)]
pub struct CommandArguments {
    /// Relative week number. '0' is the current week, '-1' is the
    /// previous week, etc.
    #[clap(short = 'w', long, value_parser, default_value_t = 0)]
    relative_week: i32,

    /// How should dates/times be displayed?
    #[clap(long, value_enum)]
    format_datetime: Option<DateTimeFormat>,

    /// How should duration be displayed?
    #[clap(long, value_enum)]
    format_duration: Option<DurationFormat>,

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
pub struct PrintAppSettings {
    pub core: CoreSettings,
    pub print: PrintSettings,
}

impl PrintAppSettings {
    pub fn new(arguments: CommandArguments) -> Result<Self, ConfigError> {
        let builder =
            new_core_settings(arguments.database_dir, arguments.database_file_name, true)?;
        let mut builder = new_print_settings(builder)?;

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
