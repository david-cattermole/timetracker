pub const APPLICATION_ID: &str = "org.timetracker.print-gui";
pub const MAIN_WINDOW_GLADE: &'static str = include_str!("main_window.glade");

pub const WINDOW_TITLE: &str = "Timetracker Print GUI";
pub const WINDOW_DEFAULT_WIDTH: i32 = 1260;
pub const WINDOW_DEFAULT_HEIGHT: i32 = 800;

// Follows the ISO8601 standard.
pub const DATETIME_FORMAT_ISO_ID: &str = "DateTimeFormat::Iso";
pub const DATETIME_FORMAT_ISO_LABEL: &str = "ISO (ISO8601 standard)";

// Follows common date-time conventions in the USA.
pub const DATETIME_FORMAT_LOCALE_ID: &str = "DateTimeFormat::Locale";
pub const DATETIME_FORMAT_LOCALE_LABEL: &str = "Locale";

// Follows user's preferences for local date/time formating rules.
pub const DATETIME_FORMAT_USA_MONTH_DAY_YEAR_ID: &str = "DateTimeFormat::UsaMonthDayYear";
pub const DATETIME_FORMAT_USA_MONTH_DAY_YEAR_LABEL: &str = "UsaMonthDayYear";

// Display exact hours and minutes.
pub const DURATION_FORMAT_HOURS_MINUTES_ID: &str = "DurationFormat::HoursMinutes";
pub const DURATION_FORMAT_HOURS_MINUTES_LABEL: &str = "Hours Minutes (12h 34m)";

// Display exact hours and minutes and seconds.
pub const DURATION_FORMAT_HOURS_MINUTES_SECONDS_ID: &str = "DurationFormat::HoursMinutesSeconds";
pub const DURATION_FORMAT_HOURS_MINUTES_SECONDS_LABEL: &str = "Hours Minutes Seconds (12h 34m 56s)";

// Hours as decimal number rounded to 6 minute increments.
pub const DURATION_FORMAT_DECIMAL_HOURS_ID: &str = "DurationFormat::DecimalHours";
pub const DURATION_FORMAT_DECIMAL_HOURS_LABEL: &str = "Decimal Hours (12.5)";
