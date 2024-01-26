use clap::Parser;
use config::ConfigError;
use serde_derive::Deserialize;
use timetracker_core::settings::new_core_settings;
use timetracker_core::settings::new_print_settings;
use timetracker_core::settings::validate_core_settings;
use timetracker_core::settings::CoreSettings;
use timetracker_core::settings::PrintSettings;

#[derive(Parser, Debug)]
#[clap(author = "David Cattermole, Copyright 2023-2024", version, about)]
pub struct CommandArguments {
    /// Return the last week's results, shortcut for
    /// '--relative-week=-1'.
    #[clap(long, value_parser, default_value_t = false)]
    pub last_week: bool,

    /// Relative week number. '0' is the current week, '-1' is the
    /// previous week, etc.
    #[clap(short = 'w', long, value_parser, default_value_t = 0)]
    pub relative_week: i32,

    /// Output file path.
    #[clap(short = 'o', long, value_parser)]
    pub output_file: Option<String>,

    /// Override the directory to search for the database file.
    #[clap(long, value_parser)]
    pub database_dir: Option<String>,

    /// Override the name of the database file to open.
    #[clap(long, value_parser)]
    pub database_file_name: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct DumpAppSettings {
    pub core: CoreSettings,
    pub print: PrintSettings,
}

impl DumpAppSettings {
    pub fn new(arguments: &CommandArguments) -> Result<Self, ConfigError> {
        let builder = new_core_settings(
            arguments.database_dir.clone(),
            arguments.database_file_name.clone(),
            false,
        )?;
        let builder = new_print_settings(builder)?;

        let settings: Self = builder.build()?.try_deserialize()?;
        validate_core_settings(&settings.core).unwrap();

        Ok(settings)
    }
}
