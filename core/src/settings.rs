use crate::filesystem::find_existing_configuration_directory_path;
use crate::filesystem::find_existing_file_path;
use crate::format::DateTimeFormat;
use crate::format::DurationFormat;
use crate::format::PrintType;
use crate::format::TimeBlockUnit;
use crate::format::TimeScale;
use config::{
    builder::DefaultState, Config, ConfigBuilder, ConfigError, Environment, File, FileFormat,
    Value, ValueKind,
};
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

/// How often will the recorder query the system to find data?
pub const RECORD_INTERVAL_SECONDS: u64 = 1;

/// How many seconds does the user need to be idle before we consider
/// the user to be in an idle state?
pub const USER_IS_IDLE_LIMIT_SECONDS: u64 = 30;

/// The name of the file used to save timetracker data.
const DEFAULT_DATABASE_FILE_NAME: &str = ".timetracker.sqlite3";

/// The name of the file used to read timetracker configuration data.
///
/// The configuration file is found by searching in the
/// "TIMETRACKER_CONFIG_PATH" environment variable (if it exists),
/// then in the home directory.
pub const DEFAULT_CONFIG_FILE_NAME: &str = ".timetracker.toml";

#[derive(Debug, Serialize, Deserialize)]
pub struct EnvVarSettings {
    pub names: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CoreSettings {
    pub database_dir: String,
    pub database_file_name: String,
    pub environment_variables: EnvVarSettings,
}

pub fn new_core_settings(
    database_dir: Option<String>,
    database_file_name: Option<String>,
    load_user_overrides: bool,
) -> Result<ConfigBuilder<DefaultState>, ConfigError> {
    let env_var_names = vec!["PWD".to_string(); 1];

    let default_database_dir = find_existing_configuration_directory_path()
        .expect("Could not find a default database directory ($HOME, $HOME/.config or $XDG_CONFIG_HOME).")
        .into_os_string()
        .into_string()
        .unwrap();

    let mut builder = Config::builder()
        .set_default("core.database_dir", default_database_dir)?
        .set_default("core.database_file_name", DEFAULT_DATABASE_FILE_NAME)?
        .set_default("core.environment_variables.names", env_var_names)?
        //
        // Allows settings from environment variables (with a prefix
        // of TIMETRACKER) eg `TIMETRACKER_CORE_DATABASE_DIR=1 ./target/app` to
        // set the `core.database_dir` key.
        .add_source(Environment::with_prefix("timetracker"))
        //
        // Overrides
        .set_override_option("core.database_dir", database_dir)?
        .set_override_option("core.database_file_name", database_file_name)?;

    // Runtime configuration file options.
    if load_user_overrides {
        let config_file_name = DEFAULT_CONFIG_FILE_NAME;
        let env_config_path = std::env::var("TIMETRACKER_CONFIG_PATH");
        let user_config_path: Option<String> = match env_config_path {
            Ok(value) => Some(value),
            Err(..) => None,
        };
        let config_file_path = find_existing_file_path(user_config_path, config_file_name);
        if let Some(file_path) = config_file_path {
            if let Some(file_path) = file_path.to_str() {
                builder =
                    builder.add_source(File::new(file_path, FileFormat::Toml).required(false));
            }
        }
    }

    Result::Ok(builder)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrintPresetSettings {
    pub print_type: Option<PrintType>,
    pub time_scale: Option<TimeScale>,
    pub format_datetime: Option<DateTimeFormat>,
    pub format_duration: Option<DurationFormat>,
    pub time_block_unit: Option<TimeBlockUnit>,
    pub bar_graph_character_num_width: Option<u8>,
    pub variable_names: Option<Vec<String>>,
}

impl PrintPresetSettings {
    pub fn new(
        print_type: Option<PrintType>,
        time_scale: Option<TimeScale>,
        format_datetime: Option<DateTimeFormat>,
        format_duration: Option<DurationFormat>,
        time_block_unit: Option<TimeBlockUnit>,
        bar_graph_character_num_width: Option<u8>,
        variable_names: Option<Vec<String>>,
    ) -> Self {
        Self {
            print_type,
            time_scale,
            format_datetime,
            format_duration,
            time_block_unit,
            bar_graph_character_num_width,
            variable_names,
        }
    }
}

impl From<PrintPresetSettings> for ValueKind {
    fn from(preset: PrintPresetSettings) -> Self {
        let mut map = HashMap::<std::string::String, Value>::new();

        match preset.print_type {
            Some(value) => map.insert(
                "print_type".to_string(),
                Value::new(
                    Some(&"print_type".to_string()),
                    ValueKind::String(value.to_string()),
                ),
            ),
            None => map.insert("print_type".to_string(), Value::new(None, ValueKind::Nil)),
        };

        match preset.time_scale {
            Some(value) => map.insert(
                "time_scale".to_string(),
                Value::new(
                    Some(&"time_scale".to_string()),
                    ValueKind::String(value.to_string()),
                ),
            ),
            None => map.insert("time_scale".to_string(), Value::new(None, ValueKind::Nil)),
        };

        match preset.format_datetime {
            Some(value) => map.insert(
                "format_datetime".to_string(),
                Value::new(
                    Some(&"format_datetime".to_string()),
                    ValueKind::String(value.to_string()),
                ),
            ),
            None => map.insert(
                "format_datetime".to_string(),
                Value::new(None, ValueKind::Nil),
            ),
        };

        match preset.format_duration {
            Some(value) => map.insert(
                "format_duration".to_string(),
                Value::new(
                    Some(&"format_duration".to_string()),
                    ValueKind::String(value.to_string()),
                ),
            ),
            None => map.insert(
                "format_duration".to_string(),
                Value::new(None, ValueKind::Nil),
            ),
        };

        match preset.time_block_unit {
            Some(value) => map.insert(
                "time_block_unit".to_string(),
                Value::new(
                    Some(&"time_block_unit".to_string()),
                    ValueKind::String(value.to_string()),
                ),
            ),
            None => map.insert(
                "time_block_unit".to_string(),
                Value::new(None, ValueKind::Nil),
            ),
        };

        match preset.bar_graph_character_num_width {
            Some(value) => map.insert(
                "bar_graph_character_num_width".to_string(),
                Value::new(
                    Some(&"bar_graph_character_num_width".to_string()),
                    ValueKind::U64(value as u64),
                ),
            ),
            None => map.insert(
                "bar_graph_character_num_width".to_string(),
                Value::new(None, ValueKind::Nil),
            ),
        };

        match preset.variable_names {
            Some(value) => {
                let envvars_array: Vec<_> = value
                    .iter()
                    .map(|x| Value::new(None, ValueKind::String(x.clone())))
                    .collect();
                map.insert(
                    "variable_names".to_string(),
                    Value::new(
                        Some(&"variable_names".to_string()),
                        ValueKind::Array(envvars_array),
                    ),
                )
            }
            None => map.insert(
                "variable_names".to_string(),
                Value::new(None, ValueKind::Nil),
            ),
        };

        ValueKind::Table(map)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PrintSettings {
    pub time_scale: TimeScale,
    pub format_datetime: DateTimeFormat,
    pub format_duration: DurationFormat,
    pub time_block_unit: TimeBlockUnit,
    pub bar_graph_character_num_width: u8,
    pub display_presets: Vec<String>,
    pub presets: HashMap<String, PrintPresetSettings>,
}

pub fn new_print_settings(
    config_builder: ConfigBuilder<DefaultState>,
) -> Result<ConfigBuilder<DefaultState>, ConfigError> {
    let preset_names = vec![
        "summary_week".to_string(),
        "summary_weekdays".to_string(),
        "working_directory_week".to_string(),
        "software_week".to_string(),
    ];

    // Default presets that will always be available to users, unless
    // they override the names.
    let mut presets = HashMap::<String, PrintPresetSettings>::new();
    presets.insert(
        "summary_week".to_string(),
        PrintPresetSettings::new(
            Some(PrintType::Summary),
            Some(TimeScale::Week),
            None,
            None,
            None,
            None,
            None,
        ),
    );
    presets.insert(
        "summary_weekdays".to_string(),
        PrintPresetSettings::new(
            Some(PrintType::Summary),
            Some(TimeScale::Weekday),
            None,
            None,
            None,
            None,
            None,
        ),
    );

    presets.insert(
        "activity_week".to_string(),
        PrintPresetSettings::new(
            Some(PrintType::Activity),
            Some(TimeScale::Week),
            None,
            None,
            None,
            None,
            None,
        ),
    );

    presets.insert(
        "activity_weekdays".to_string(),
        PrintPresetSettings::new(
            Some(PrintType::Activity),
            Some(TimeScale::Weekday),
            None,
            None,
            None,
            None,
            None,
        ),
    );

    presets.insert(
        "working_directory_week".to_string(),
        PrintPresetSettings::new(
            Some(PrintType::Variables),
            Some(TimeScale::Week),
            None,
            None,
            None,
            None,
            Some(vec!["PWD".to_string()]),
        ),
    );
    presets.insert(
        "working_directory_weekdays".to_string(),
        PrintPresetSettings::new(
            Some(PrintType::Variables),
            Some(TimeScale::Weekday),
            None,
            None,
            None,
            None,
            Some(vec!["PWD".to_string()]),
        ),
    );

    presets.insert(
        "software_week".to_string(),
        PrintPresetSettings::new(
            Some(PrintType::Software),
            Some(TimeScale::Week),
            None,
            None,
            None,
            None,
            None,
        ),
    );

    presets.insert(
        "software_weekdays".to_string(),
        PrintPresetSettings::new(
            Some(PrintType::Software),
            Some(TimeScale::Weekday),
            None,
            None,
            None,
            None,
            None,
        ),
    );

    let config_builder = config_builder
        .set_default("print.time_scale", "Week")?
        .set_default("print.format_datetime", "Locale")?
        .set_default("print.format_duration", "HoursMinutes")?
        .set_default("print.time_block_unit", "SixtyMinutes")?
        .set_default("print.bar_graph_character_num_width", 60)?
        .set_default("print.display_presets", preset_names)?
        .set_default("print.presets", presets)?;
    Result::Ok(config_builder)
}

pub fn new_recorder_settings(
    config_builder: ConfigBuilder<DefaultState>,
) -> Result<ConfigBuilder<DefaultState>, ConfigError> {
    Result::Ok(config_builder)
}

pub fn new_systray_settings(
    config_builder: ConfigBuilder<DefaultState>,
) -> Result<ConfigBuilder<DefaultState>, ConfigError> {
    Result::Ok(config_builder)
}
