use chrono;
use chrono::TimeZone;

/// Determines the formatting used for dates/times.
#[derive(Debug, Copy, Clone)]
pub enum DateTimeFormat {
    /// Follows the ISO8601 standard.
    Iso,

    /// Follows common date-time conventions in the USA.
    UsaMonthDayYear,

    /// Follows user's preferences for local date/time formating
    /// rules.
    Locale,
}

/// Determines the formatting used for durations.
#[derive(Debug, Copy, Clone)]
pub enum DurationFormat {
    /// Display exact hours and minutes.
    HoursMinutes,

    /// Display exact hours and minutes and seconds.
    HoursMinutesSeconds,

    /// Hours as decimal number rounded to 6 minute increments.
    DecimalHours,
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
                format!("{:02}h {:02}m", hours, minutes_rem).to_string()
            }
        }
        DurationFormat::HoursMinutesSeconds => {
            if hours == 0 && minutes == 0 && seconds == 0 {
                "00h 00m 00s".to_string()
            } else {
                let seconds_rem = seconds.checked_rem(60).unwrap();
                let minutes_rem = (seconds / 60).checked_rem(60).unwrap();
                let hours_rem = seconds / (60 * 60);
                format!("{:02}h {:02}m {:02}s", hours_rem, minutes_rem, seconds_rem).to_string()
            }
        }
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
        let datetime = chrono::DateTime::<chrono::Utc>::from_utc(
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
        let datetime = chrono::DateTime::<chrono::Utc>::from_utc(
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
        let datetime = chrono::DateTime::<chrono::Utc>::from_utc(
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
        let datetime = chrono::DateTime::<chrono::Utc>::from_utc(
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
