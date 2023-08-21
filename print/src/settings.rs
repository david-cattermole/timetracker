use clap::Parser;
use clap::ValueEnum;
use config::{ConfigError, ValueKind};
use serde_derive::Deserialize;
use std::fmt;
use timetracker_core::format::DateTimeFormat;
use timetracker_core::format::DurationFormat;
use timetracker_core::settings::new_core_settings;
use timetracker_core::settings::new_print_settings;
use timetracker_core::settings::CoreSettings;
use timetracker_core::settings::PrintSettings;

#[derive(Debug, Copy, Clone, ValueEnum)]
pub enum CommandMode {
    Print,
    List,
}

impl fmt::Display for CommandMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CommandMode::Print => write!(f, "print"),
            CommandMode::List => write!(f, "list"),
        }
    }
}

impl From<CommandMode> for ValueKind {
    fn from(value: CommandMode) -> Self {
        ValueKind::String(format!("{}", value))
    }
}

#[derive(Parser, Debug)]
#[clap(author = "David Cattermole, Copyright 2023", version, about)]
pub struct CommandArguments {
    /// What mode should this command run in?
    #[clap(short = 'm', long, value_parser, default_value_t = CommandMode::Print)]
    pub mode: CommandMode,

    /// Relative week number. '0' is the current week, '-1' is the
    /// previous week, etc.
    #[clap(short = 'w', long, value_parser, default_value_t = 0)]
    pub relative_week: i32,

    /// Which presets to print with?
    #[clap(short = 'p', long, value_parser)]
    pub presets: Option<Vec<String>>,

    /// How should dates/times be displayed?
    #[clap(long, value_enum)]
    pub format_datetime: Option<DateTimeFormat>,

    /// How should duration be displayed?
    #[clap(long, value_enum)]
    pub format_duration: Option<DurationFormat>,

    /// Override the directory to search for the database file.
    #[clap(long, value_parser)]
    pub database_dir: Option<String>,

    /// Override the name of the database file to open.
    #[clap(long, value_parser)]
    pub database_file_name: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct PrintAppSettings {
    pub core: CoreSettings,
    pub print: PrintSettings,
}

impl PrintAppSettings {
    pub fn new(arguments: &CommandArguments) -> Result<Self, ConfigError> {
        let builder = new_core_settings(
            arguments.database_dir.clone(),
            arguments.database_file_name.clone(),
            true,
        )?;
        let mut builder = new_print_settings(builder)?;

        // Use command line 'arguments' to override the default
        // values. These will always override any configuration file
        // or environment variable.
        builder = builder
            .set_override("print.relative_week", arguments.relative_week)?
            .set_override_option("print.display_presets", arguments.presets.clone())?
            .set_override_option("print.format_datetime", arguments.format_datetime)?
            .set_override_option("print.format_duration", arguments.format_duration)?;

        let settings = builder.build()?;
        settings.try_deserialize()
    }
}
