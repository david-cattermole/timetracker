use crate::filesystem::find_existing_file_path;
use crate::format::DateTimeFormat;
use crate::format::DurationFormat;
use crate::format::FirstDayOfWeek;
use crate::format::TimeDuration;
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
const DATABASE_FILE_NAME: &str = ".timetracker.sqlite3";

/// The database default directory - the home directory. This path is
/// expanded using the 'shellexpand' crate.
const DATABASE_DIR: &str = "~/";

/// The configuration default directory - the home directory. This path is
/// expanded using the 'shellexpand' crate.
pub const CONFIG_DIR: &str = "~/";

/// The name of the file used to read timetracker configuration data.
///
/// The configuration file is found by searching in the
/// "TIMETRACKER_CONFIG_PATH" environment variable (if it exists),
/// then in the home directory.
pub const CONFIG_FILE_NAME: &str = ".timetracker.toml";

#[derive(Debug, Serialize, Deserialize)]
pub struct EnvVarSettings {
    pub names: Vec<String>,
    pub labels: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CoreSettings {
    pub database_dir: String,
    pub database_file_name: String,
    pub record_interval_seconds: u64,
    pub user_is_idle_limit_seconds: u64,
    pub environment_variables: EnvVarSettings,
}

pub fn new_core_settings(
    database_dir: Option<String>,
    database_file_name: Option<String>,
    load_user_overrides: bool,
) -> Result<ConfigBuilder<DefaultState>, ConfigError> {
    let mut env_var_names = Vec::new();
    env_var_names.push("PWD".to_string());

    let env_var_labels = HashMap::<String, String>::new();

    let mut builder = Config::builder()
        .set_default("core.record_interval_seconds", RECORD_INTERVAL_SECONDS)?
        .set_default(
            "core.user_is_idle_limit_seconds",
            USER_IS_IDLE_LIMIT_SECONDS,
        )?
        .set_default("core.database_dir", DATABASE_DIR)?
        .set_default("core.database_file_name", DATABASE_FILE_NAME)?
        .set_default("core.environment_variables.names", env_var_names)?
        .set_default("core.environment_variables.labels", env_var_labels)?
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
        let config_file_name = CONFIG_FILE_NAME;
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

#[derive(Debug, Serialize, Deserialize)]
pub struct PrintPresetSettings {
    // TODO: Do we need to control printing the summary times? Or can
    // we just assume we always want to print them?
    //
    // TODO: Pick better names than 'filter_*'.
    pub filter_executable: bool,
    pub filter_env_vars: Vec<String>,
    // TODO: Add an option to display when a user is active during a
    // day/week. How can we best display that information in a
    // text-based print out?
    pub time_duration: Option<TimeDuration>,
    pub format_datetime: Option<DateTimeFormat>,
    pub format_duration: Option<DurationFormat>,
    pub first_day_of_week: Option<FirstDayOfWeek>,
}

impl PrintPresetSettings {
    fn new(
        filter_executable: bool,
        filter_env_vars: Vec<String>,
        time_duration: Option<TimeDuration>,
        format_datetime: Option<DateTimeFormat>,
        format_duration: Option<DurationFormat>,
        first_day_of_week: Option<FirstDayOfWeek>,
    ) -> Self {
        Self {
            filter_executable,
            filter_env_vars,
            time_duration,
            format_datetime,
            format_duration,
            first_day_of_week,
        }
    }
}

impl From<PrintPresetSettings> for ValueKind {
    fn from(preset: PrintPresetSettings) -> Self {
        let mut map = HashMap::<std::string::String, Value>::new();
        map.insert(
            "filter_executable".to_string(),
            Value::new(
                Some(&"filter_executable".to_string()),
                ValueKind::Boolean(preset.filter_executable),
            ),
        );

        let envvars_array: Vec<_> = preset
            .filter_env_vars
            .iter()
            .map(|x| Value::new(None, ValueKind::String(x.clone())))
            .collect();
        map.insert(
            "filter_env_vars".to_string(),
            Value::new(
                Some(&"filter_env_vars".to_string()),
                ValueKind::Array(envvars_array),
            ),
        );

        match preset.time_duration {
            Some(value) => map.insert(
                "time_duration".to_string(),
                Value::new(
                    Some(&"time_duration".to_string()),
                    ValueKind::String(value.to_string()),
                ),
            ),
            None => map.insert(
                "time_duration".to_string(),
                Value::new(None, ValueKind::Nil),
            ),
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

        match preset.first_day_of_week {
            Some(value) => map.insert(
                "first_day_of_week".to_string(),
                Value::new(
                    Some(&"first_day_of_week".to_string()),
                    ValueKind::String(value.to_string()),
                ),
            ),
            None => map.insert(
                "first_day_of_week".to_string(),
                Value::new(None, ValueKind::Nil),
            ),
        };

        ValueKind::Table(map)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PrintSettings {
    pub relative_week: i32,
    pub time_duration: TimeDuration,
    pub format_datetime: DateTimeFormat,
    pub format_duration: DurationFormat,
    pub first_day_of_week: FirstDayOfWeek,
    pub display_week: bool,
    pub display_weekday: bool,
    pub display_week_task: bool,
    pub display_weekday_task: bool,
    pub display_week_software: bool,
    pub display_presets: Vec<String>,
    pub presets: HashMap<String, PrintPresetSettings>,
}

pub fn new_print_settings(
    config_builder: ConfigBuilder<DefaultState>,
) -> Result<ConfigBuilder<DefaultState>, ConfigError> {
    let mut preset_names = Vec::<String>::new();
    preset_names.push("summary_week".to_string());
    preset_names.push("summary_weekdays".to_string());
    preset_names.push("executable_week".to_string());
    preset_names.push("executable_weekdays".to_string());
    preset_names.push("working_directory_week".to_string());
    preset_names.push("working_directory_weekdays".to_string());

    // Default presets that will always be available to users, unless
    // they override the names.
    let mut presets = HashMap::<String, PrintPresetSettings>::new();
    presets.insert(
        "summary_week".to_string(),
        PrintPresetSettings::new(
            false,
            Vec::new(),
            Some(TimeDuration::FullWeek),
            None,
            None,
            None,
        ),
    );
    presets.insert(
        "summary_weekdays".to_string(),
        PrintPresetSettings::new(
            false,
            Vec::new(),
            Some(TimeDuration::FullWeekPerDay),
            None,
            None,
            None,
        ),
    );

    presets.insert(
        "executable_week".to_string(),
        PrintPresetSettings::new(
            true,
            Vec::new(),
            Some(TimeDuration::FullWeek),
            None,
            None,
            None,
        ),
    );

    presets.insert(
        "executable_weekdays".to_string(),
        PrintPresetSettings::new(
            true,
            Vec::new(),
            Some(TimeDuration::FullWeekPerDay),
            None,
            None,
            None,
        ),
    );

    presets.insert(
        "working_directory_week".to_string(),
        PrintPresetSettings::new(
            false,
            vec!["PWD".to_string()],
            Some(TimeDuration::FullWeek),
            None,
            None,
            None,
        ),
    );
    presets.insert(
        "working_directory_weekdays".to_string(),
        PrintPresetSettings::new(
            false,
            vec!["PWD".to_string()],
            Some(TimeDuration::FullWeekPerDay),
            None,
            None,
            None,
        ),
    );

    let config_builder = config_builder
        .set_default("print.relative_week", 0)?
        .set_default("print.time_duration", "FullWeek")?
        .set_default("print.format_datetime", "Locale")?
        .set_default("print.format_duration", "HoursMinutes")?
        .set_default("print.first_day_of_week", "Monday")?
        .set_default("print.display_week", true)?
        .set_default("print.display_weekday", true)?
        .set_default("print.display_week_task", true)?
        .set_default("print.display_weekday_task", false)?
        .set_default("print.display_week_software", false)?
        .set_default("print.display_presets", preset_names)?
        .set_default("print.presets", presets)?;
    Result::Ok(config_builder)
}

pub fn new_recorder_settings(
    config_builder: ConfigBuilder<DefaultState>,
) -> Result<ConfigBuilder<DefaultState>, ConfigError> {
    Result::Ok(config_builder)
}
