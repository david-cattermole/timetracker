use clap::Parser;
use config::ConfigError;
use serde_derive::Deserialize;
use timetracker_core::format::color_mode_to_use_color;
use timetracker_core::format::ColorMode;
use timetracker_core::format::DateTimeFormat;
use timetracker_core::format::DurationFormat;
use timetracker_core::settings::new_core_settings;
use timetracker_core::settings::new_print_gui_settings;
use timetracker_core::settings::validate_core_settings;
use timetracker_core::settings::CoreSettings;
use timetracker_core::settings::PrintSettings;

// This command arguments are similar to the timetracker-print
// arguments, since this program is intended to be the "same" program,
// but with a GUI.
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

    /// Which presets to print with?
    #[clap(short = 'p', long, value_parser)]
    pub presets: Option<Vec<String>>,

    /// How should dates/times be displayed?
    #[clap(long, value_enum)]
    pub format_datetime: Option<DateTimeFormat>,

    /// How should duration be displayed?
    #[clap(long, value_enum)]
    pub format_duration: Option<DurationFormat>,

    /// Show colored text?
    // Similar to 'git diff --color' flag.
    #[clap(long, value_enum)]
    pub color: Option<ColorMode>,

    /// Override the directory to search for the database file.
    #[clap(long, value_parser)]
    pub database_dir: Option<String>,

    /// Override the name of the database file to open.
    #[clap(long, value_parser)]
    pub database_file_name: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct PrintGuiAppSettings {
    pub core: CoreSettings,
    pub print: PrintSettings,
}

impl PrintGuiAppSettings {
    pub fn new(arguments: &CommandArguments) -> Result<Self, ConfigError> {
        let builder = new_core_settings(
            arguments.database_dir.clone(),
            arguments.database_file_name.clone(),
            false,
        )?;
        let mut builder = new_print_gui_settings(builder)?;

        // Use command line 'arguments' to override the default
        // values. These will always override any configuration file
        // or environment variable.
        let use_color = color_mode_to_use_color(arguments.color, false, false);
        builder = builder
            .set_override_option("print.display_presets", arguments.presets.clone())?
            .set_override_option("print.format_datetime", arguments.format_datetime)?
            .set_override_option("print.format_duration", arguments.format_duration)?
            .set_override_option("print.use_color", Some(use_color))?;

        let settings: Self = builder.build()?.try_deserialize()?;
        validate_core_settings(&settings.core).unwrap();

        Ok(settings)
    }
}
