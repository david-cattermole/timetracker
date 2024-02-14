use crate::constants::DATETIME_FORMAT_ISO_ID;
use crate::constants::DATETIME_FORMAT_LOCALE_ID;
use crate::constants::DATETIME_FORMAT_USA_MONTH_DAY_YEAR_ID;
use crate::constants::DURATION_FORMAT_DECIMAL_HOURS_ID;
use crate::constants::DURATION_FORMAT_HOURS_MINUTES_ID;
use crate::constants::DURATION_FORMAT_HOURS_MINUTES_SECONDS_ID;

use anyhow::Result;
use chrono::Datelike;

use timetracker_core::format::DateTimeFormat;
use timetracker_core::format::DurationFormat;
use timetracker_print_lib::datetime::get_week_datetime_local;
use timetracker_print_lib::datetime::DateTimeLocalPair;

/// Convert the week number into a start datetime and end datetime.
///
/// Assumes the week number is contained in the current year.
pub fn get_absolute_week_start_end(week_num: u32) -> Result<DateTimeLocalPair> {
    let today_local_timezone = chrono::Local::now();
    let today_year = today_local_timezone.year();
    Ok(get_week_datetime_local(today_year, week_num))
}

pub fn datetime_format_as_id(value: DateTimeFormat) -> &'static str {
    match value {
        DateTimeFormat::Iso => DATETIME_FORMAT_ISO_ID,
        DateTimeFormat::Locale => DATETIME_FORMAT_LOCALE_ID,
        DateTimeFormat::UsaMonthDayYear => DATETIME_FORMAT_USA_MONTH_DAY_YEAR_ID,
    }
}

pub fn id_as_datetime_format(value: Option<&glib::GString>) -> Option<DateTimeFormat> {
    match value {
        Some(v) => match v.as_str() {
            DATETIME_FORMAT_ISO_ID => Some(DateTimeFormat::Iso),
            DATETIME_FORMAT_LOCALE_ID => Some(DateTimeFormat::Locale),
            DATETIME_FORMAT_USA_MONTH_DAY_YEAR_ID => Some(DateTimeFormat::UsaMonthDayYear),
            &_ => todo!(),
        },
        None => None,
    }
}

pub fn duration_format_as_id(value: DurationFormat) -> &'static str {
    match value {
        DurationFormat::HoursMinutes => DURATION_FORMAT_HOURS_MINUTES_ID,
        DurationFormat::HoursMinutesSeconds => DURATION_FORMAT_HOURS_MINUTES_SECONDS_ID,
        DurationFormat::DecimalHours => DURATION_FORMAT_DECIMAL_HOURS_ID,
    }
}

pub fn id_as_duration_format(value: Option<&glib::GString>) -> Option<DurationFormat> {
    match value {
        Some(v) => match v.as_str() {
            DURATION_FORMAT_HOURS_MINUTES_ID => Some(DurationFormat::HoursMinutes),
            DURATION_FORMAT_HOURS_MINUTES_SECONDS_ID => Some(DurationFormat::HoursMinutesSeconds),
            DURATION_FORMAT_DECIMAL_HOURS_ID => Some(DurationFormat::DecimalHours),
            &_ => todo!(),
        },
        None => None,
    }
}
