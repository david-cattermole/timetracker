use chrono::Datelike;
use chrono::TimeZone;

pub type DateTimeLocalPair = (
    chrono::DateTime<chrono::Local>,
    chrono::DateTime<chrono::Local>,
);

// TODO: This assumes starting the week on Monday morning, until
// Sunday night. Some People assume Saturday is the last day, others
// maybe Friday. This needs to be configurable with the
// "FirstDayOfWeek" enum.
const WORK_WEEK_WEEKDAYS: &[chrono::Weekday] = &[
    chrono::Weekday::Mon,
    chrono::Weekday::Tue,
    chrono::Weekday::Wed,
    chrono::Weekday::Thu,
    chrono::Weekday::Fri,
    chrono::Weekday::Sat,
    chrono::Weekday::Sun,
];

/// Get the pair of datetimes representing the first and last
/// datetimes of a sub-set of working days in a week.
///
/// `year` is the year of the week datetime to get, such as `2015`, or
/// `2022`.
///
/// `week` is the week number to get the details for.
///
/// `start_weekday` is the first weekday of the week.
/// `end_weekday` is the first weekday of the week.
fn get_datetime_local_week_range(
    year: i32,
    week: u32,
    start_weekday: chrono::Weekday,
    end_weekday: chrono::Weekday,
) -> DateTimeLocalPair {
    let start_date = chrono::NaiveDate::from_isoywd_opt(year, week, start_weekday)
        .expect("Start date year/week/day should be valid.");
    let end_date = chrono::NaiveDate::from_isoywd_opt(year, week, end_weekday)
        .expect("End date year/week/day should be valid.");

    let start_datetime = start_date
        .and_hms_opt(0, 0, 0)
        .expect("Start datetime should be valid.");
    let end_datetime = end_date
        .and_hms_opt(23, 59, 59)
        .expect("End datetime should be valid.");

    let start_datetime = chrono::Local.from_local_datetime(&start_datetime);
    let end_datetime = chrono::Local.from_local_datetime(&end_datetime);

    (start_datetime.unwrap(), end_datetime.unwrap())
}

/// Get the pair of datetimes representing the first and last
/// datetimes of a working week (starting Monday morning and ending
/// Sunday night).
///
/// `year` is the year of the week datetime to get, such as `2015`, or
/// `2022`.
///
/// `week` is the week number to get the details for.
pub fn get_week_datetime_local(year: i32, week: u32) -> DateTimeLocalPair {
    get_datetime_local_week_range(year, week, chrono::Weekday::Mon, chrono::Weekday::Sun)
}

pub fn get_weekdays_datetime_local(
    week_start_datetime: chrono::DateTime<chrono::Local>,
    week_end_datetime: chrono::DateTime<chrono::Local>,
) -> Vec<(chrono::Weekday, DateTimeLocalPair)> {
    let year = week_start_datetime.year();
    let iso_week = week_start_datetime.iso_week();
    assert_eq!(year, week_end_datetime.year());
    assert_eq!(iso_week, week_end_datetime.iso_week());
    let week: u32 = iso_week.week();

    let mut weekdays_datetime_pairs = Vec::<(chrono::Weekday, DateTimeLocalPair)>::new();

    for weekday in WORK_WEEK_WEEKDAYS {
        let weekdays_datetime_pair = get_datetime_local_week_range(year, week, *weekday, *weekday);
        weekdays_datetime_pairs.push((*weekday, weekdays_datetime_pair));
    }

    weekdays_datetime_pairs
}

pub fn utc_seconds_to_datetime_local(utc_time_seconds: u64) -> chrono::DateTime<chrono::Local> {
    chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(
        chrono::NaiveDateTime::from_timestamp_opt(utc_time_seconds.try_into().unwrap(), 0).unwrap(),
        chrono::Utc,
    )
    .with_timezone(&chrono::Local)
}
