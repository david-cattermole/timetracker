use chrono;
use chrono::TimeZone;
use clap::ValueEnum;
use config::ValueKind;
use serde_derive::{Deserialize, Serialize};
use std::fmt;

/// Determines the formatting used for dates/times.
#[derive(Debug, Copy, Clone, ValueEnum, Serialize, Deserialize)]
pub enum DateTimeFormat {
    /// Follows the ISO8601 standard.
    Iso,

    /// Follows common date-time conventions in the USA.
    UsaMonthDayYear,

    /// Follows user's preferences for local date/time formating
    /// rules.
    Locale,
}

impl fmt::Display for DateTimeFormat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DateTimeFormat::Locale => write!(f, "Locale"),
            DateTimeFormat::Iso => write!(f, "Iso"),
            DateTimeFormat::UsaMonthDayYear => write!(f, "UsaMonthDayYear"),
        }
    }
}

impl From<DateTimeFormat> for ValueKind {
    fn from(value: DateTimeFormat) -> Self {
        ValueKind::String(format!("{}", value))
    }
}

/// Determines the formatting used for durations.
#[derive(Debug, Copy, Clone, ValueEnum, Serialize, Deserialize)]
pub enum DurationFormat {
    /// Display exact hours and minutes.
    HoursMinutes,

    /// Display exact hours and minutes and seconds.
    HoursMinutesSeconds,

    /// Hours as decimal number rounded to 6 minute increments.
    DecimalHours,
}

impl fmt::Display for DurationFormat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DurationFormat::HoursMinutes => write!(f, "HoursMinutes"),
            DurationFormat::HoursMinutesSeconds => write!(f, "HoursMinutesSeconds"),
            DurationFormat::DecimalHours => write!(f, "DecimalHours"),
        }
    }
}

impl From<DurationFormat> for ValueKind {
    fn from(value: DurationFormat) -> Self {
        ValueKind::String(format!("{}", value))
    }
}

/// The options for representing a duration of time.
#[derive(Debug, Copy, Clone, ValueEnum, Serialize, Deserialize)]
pub enum TimeScale {
    /// A week-long duration of first day (usually Monday) at 00:00 AM
    /// to last day (usually Sunday) 23:59 PM.
    Week,

    /// A week duration (usually Monday to Sunday), split into each day
    /// 00:00 AM) to 23:59 PM.
    Weekday,
}

impl fmt::Display for TimeScale {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TimeScale::Week => write!(f, "Week"),
            TimeScale::Weekday => {
                write!(f, "Weekday")
            }
        }
    }
}

impl From<TimeScale> for ValueKind {
    fn from(value: TimeScale) -> Self {
        ValueKind::String(format!("{}", value))
    }
}

pub fn format_duration(duration: chrono::Duration, duration_format: DurationFormat) -> String {
    let hours = duration.num_hours();
    let minutes = duration.num_minutes();
    let seconds = duration.num_seconds();
    match duration_format {
        DurationFormat::DecimalHours => {
            if hours == 0 && minutes == 0 {
                "0.0".to_string()
            } else {
                // The decimal-hours duration format is a
                // floating-point-like number, such as 1.0
                // representing 1 hour, and 1.5 representing 1 hour
                // and 30 minutes. Here we convert the minutes to a
                // fraction in alignment with this formatting, which
                // gives each 0.1 increment a value of 6 minutes
                // because "6.0 / 60.0 = 0.1".
                let mut minutes_ratio = (minutes as f64) / 60.0;
                minutes_ratio *= 10.0;
                minutes_ratio = minutes_ratio.round();
                minutes_ratio *= 0.1;
                format!("{:.1}", minutes_ratio)
            }
        }
        DurationFormat::HoursMinutes => {
            if hours == 0 && minutes == 0 {
                "00h 00m".to_string()
            } else {
                let minutes_rem = minutes.checked_rem(60).unwrap();
                format!("{:02}h {:02}m", hours, minutes_rem)
            }
        }
        DurationFormat::HoursMinutesSeconds => {
            if hours == 0 && minutes == 0 && seconds == 0 {
                "00h 00m 00s".to_string()
            } else {
                let seconds_rem = seconds.checked_rem(60).unwrap();
                let minutes_rem = (seconds / 60).checked_rem(60).unwrap();
                let hours_rem = seconds / (60 * 60);
                format!("{:02}h {:02}m {:02}s", hours_rem, minutes_rem, seconds_rem)
            }
        }
    }
}

pub fn format_time_no_seconds<Tz: TimeZone>(
    datetime: chrono::DateTime<Tz>,
    datetime_format: DateTimeFormat,
) -> String
where
    Tz::Offset: std::fmt::Display,
{
    match datetime_format {
        DateTimeFormat::Iso => datetime.format("%H:%M").to_string(),
        DateTimeFormat::UsaMonthDayYear => datetime.format("%I:%M %p").to_string(),
        DateTimeFormat::Locale => datetime.format("%X").to_string(),
    }
}

pub fn format_naive_time_no_seconds(
    datetime: chrono::NaiveTime,
    datetime_format: DateTimeFormat,
) -> String {
    match datetime_format {
        DateTimeFormat::Iso => datetime.format("%H:%M").to_string(),
        DateTimeFormat::UsaMonthDayYear => datetime.format("%I:%M %p").to_string(),
        DateTimeFormat::Locale => datetime.format("%X").to_string(),
    }
}

pub fn format_time<Tz: TimeZone>(
    datetime: chrono::DateTime<Tz>,
    datetime_format: DateTimeFormat,
) -> String
where
    Tz::Offset: std::fmt::Display,
{
    match datetime_format {
        DateTimeFormat::Iso => datetime.format("%H:%M:%S").to_string(),
        DateTimeFormat::UsaMonthDayYear => datetime.format("%I:%M:%S %p").to_string(),
        DateTimeFormat::Locale => datetime.format("%X").to_string(),
    }
}

pub fn format_date<Tz: TimeZone>(
    datetime: chrono::DateTime<Tz>,
    datetime_format: DateTimeFormat,
) -> String
where
    Tz::Offset: std::fmt::Display,
{
    match datetime_format {
        DateTimeFormat::Iso => datetime.format("%Y-%m-%d").to_string(),
        DateTimeFormat::UsaMonthDayYear => datetime.format("%m/%d/%Y").to_string(),
        DateTimeFormat::Locale => datetime.format("%x").to_string(),
    }
}

pub fn format_datetime<Tz: TimeZone>(
    datetime: chrono::DateTime<Tz>,
    datetime_format: DateTimeFormat,
) -> String
where
    Tz::Offset: std::fmt::Display,
{
    match datetime_format {
        DateTimeFormat::Iso => datetime.format("%Y-%m-%d %H:%M:%S").to_string(),
        DateTimeFormat::UsaMonthDayYear => datetime.format("%m/%d/%Y %I:%M:%S %p").to_string(),
        DateTimeFormat::Locale => datetime.format("%x %X").to_string(),
    }
}

#[derive(Debug, Copy, Clone, ValueEnum, Serialize, Deserialize)]
pub enum TimeBlockUnit {
    FiveMinutes,
    TenMinutes,
    FifteenMinutes,
    ThirtyMinutes,
    SixtyMinutes,
}

impl TimeBlockUnit {
    pub fn as_minutes(self) -> u64 {
        match self {
            TimeBlockUnit::FiveMinutes => 5,
            TimeBlockUnit::TenMinutes => 10,
            TimeBlockUnit::FifteenMinutes => 15,
            TimeBlockUnit::ThirtyMinutes => 30,
            TimeBlockUnit::SixtyMinutes => 60,
        }
    }
}

impl fmt::Display for TimeBlockUnit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TimeBlockUnit::FiveMinutes => write!(f, "FiveMinutes"),
            TimeBlockUnit::TenMinutes => write!(f, "TenMinutes"),
            TimeBlockUnit::FifteenMinutes => write!(f, "FifteenMinutes"),
            TimeBlockUnit::ThirtyMinutes => write!(f, "ThirtyMinutes"),
            TimeBlockUnit::SixtyMinutes => write!(f, "SixtyMinutes"),
        }
    }
}

impl From<TimeBlockUnit> for ValueKind {
    fn from(value: TimeBlockUnit) -> Self {
        ValueKind::String(format!("{}", value))
    }
}

#[derive(Debug, Copy, Clone, ValueEnum, Serialize, Deserialize)]
pub enum PrintType {
    Summary,
    Activity,
    Variables,
    Software,
}

impl fmt::Display for PrintType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            PrintType::Summary => write!(f, "Summary"),
            PrintType::Activity => {
                write!(f, "Activity")
            }
            PrintType::Variables => write!(f, "Variables"),
            PrintType::Software => write!(f, "Software"),
        }
    }
}

impl From<PrintType> for ValueKind {
    fn from(value: PrintType) -> Self {
        ValueKind::String(format!("{}", value))
    }
}

#[derive(Debug, Copy, Clone, ValueEnum, Serialize, Deserialize)]
pub enum ColorMode {
    Auto,
    Never,
    Always,
}

impl fmt::Display for ColorMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ColorMode::Auto => write!(f, "Auto"),
            ColorMode::Never => write!(f, "Never"),
            ColorMode::Always => write!(f, "Always"),
        }
    }
}

impl From<ColorMode> for ValueKind {
    fn from(value: ColorMode) -> Self {
        ValueKind::String(format!("{}", value))
    }
}

pub fn color_mode_to_use_color(
    color_mode: Option<ColorMode>,
    auto_value: bool,
    fallback_value: bool,
) -> bool {
    match color_mode {
        None => fallback_value,
        Some(ColorMode::Auto) => auto_value,
        Some(ColorMode::Always) => true,
        Some(ColorMode::Never) => false,
    }
}

#[cfg(test)]
mod tests {

    use crate::format::*;

    #[test]
    fn test_format_duration_decimal_hours_round_down_1() {
        let duration = chrono::Duration::seconds(1);
        let duration_text = format_duration(duration, DurationFormat::DecimalHours);
        assert_eq!(duration_text, "0.0");
    }

    #[test]
    fn test_format_duration_decimal_hours_round_down_2() {
        let duration = chrono::Duration::minutes(2);
        let duration_text = format_duration(duration, DurationFormat::DecimalHours);
        assert_eq!(duration_text, "0.0");
    }

    #[test]
    fn test_format_duration_decimal_hours_round_up_1() {
        let duration = chrono::Duration::minutes(59);
        let duration_text = format_duration(duration, DurationFormat::DecimalHours);
        assert_eq!(duration_text, "1.0");
    }

    #[test]
    fn test_format_duration_decimal_hours_round_up_2() {
        let duration = chrono::Duration::minutes(57);
        let duration_text = format_duration(duration, DurationFormat::DecimalHours);
        assert_eq!(duration_text, "1.0");
    }

    #[test]
    fn test_format_duration_hours_minutes_1() {
        let duration = chrono::Duration::minutes(0);
        let duration_text = format_duration(duration, DurationFormat::HoursMinutes);
        assert_eq!(duration_text, "00h 00m");
    }

    #[test]
    fn test_format_duration_hours_minutes_2() {
        let duration = chrono::Duration::minutes(10);
        let duration_text = format_duration(duration, DurationFormat::HoursMinutes);
        assert_eq!(duration_text, "00h 10m");
    }

    #[test]
    fn test_format_duration_hours_minutes_3() {
        let duration = chrono::Duration::minutes(61);
        let duration_text = format_duration(duration, DurationFormat::HoursMinutes);
        assert_eq!(duration_text, "01h 01m");
    }

    #[test]
    fn test_format_duration_hours_minutes_4() {
        let duration = chrono::Duration::minutes(179);
        let duration_text = format_duration(duration, DurationFormat::HoursMinutes);
        assert_eq!(duration_text, "02h 59m");
    }

    #[test]
    fn test_format_duration_hours_mins_secs_1() {
        let duration = chrono::Duration::minutes(0);
        let duration_text = format_duration(duration, DurationFormat::HoursMinutesSeconds);
        assert_eq!(duration_text, "00h 00m 00s");
    }

    #[test]
    fn test_format_duration_hours_mins_secs_2() {
        let duration = chrono::Duration::minutes(10);
        let duration_text = format_duration(duration, DurationFormat::HoursMinutesSeconds);
        assert_eq!(duration_text, "00h 10m 00s");
    }

    #[test]
    fn test_format_duration_hours_mins_secs_3() {
        let duration = chrono::Duration::minutes(61);
        let duration_text = format_duration(duration, DurationFormat::HoursMinutesSeconds);
        assert_eq!(duration_text, "01h 01m 00s");
    }

    #[test]
    fn test_format_duration_hours_mins_secs_4() {
        let duration = chrono::Duration::minutes(179);
        let duration_text = format_duration(duration, DurationFormat::HoursMinutesSeconds);
        assert_eq!(duration_text, "02h 59m 00s");
    }

    #[test]
    fn test_format_date_iso_1() {
        let datetime = chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(
            chrono::NaiveDate::from_ymd_opt(2016, 7, 8)
                .unwrap()
                .and_hms_opt(9, 10, 11)
                .unwrap(),
            chrono::Utc,
        );
        let datetime_text = format_date(datetime, DateTimeFormat::Iso);
        assert_eq!(datetime_text, "2016-07-08");
    }

    #[test]
    fn test_format_date_month_day_year_usa_1() {
        let datetime = chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(
            chrono::NaiveDate::from_ymd_opt(2016, 7, 8)
                .unwrap()
                .and_hms_opt(9, 10, 11)
                .unwrap(),
            chrono::Utc,
        );
        let datetime_text = format_date(datetime, DateTimeFormat::UsaMonthDayYear);
        assert_eq!(datetime_text, "07/08/2016");
    }

    #[test]
    fn test_format_datetime_iso_1() {
        let datetime = chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(
            chrono::NaiveDate::from_ymd_opt(2016, 7, 8)
                .unwrap()
                .and_hms_opt(9, 10, 11)
                .unwrap(),
            chrono::Utc,
        );
        let datetime_text = format_datetime(datetime, DateTimeFormat::Iso);
        assert_eq!(datetime_text, "2016-07-08 09:10:11");
    }

    #[test]
    fn test_format_datetime_month_day_year_usa_1() {
        let datetime = chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(
            chrono::NaiveDate::from_ymd_opt(2016, 7, 8)
                .unwrap()
                .and_hms_opt(9, 10, 11)
                .unwrap(),
            chrono::Utc,
        );
        let datetime_text = format_datetime(datetime, DateTimeFormat::UsaMonthDayYear);
        assert_eq!(datetime_text, "07/08/2016 09:10:11 AM");
    }
}
