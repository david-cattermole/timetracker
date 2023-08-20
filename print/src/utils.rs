use chrono;
use chrono::Datelike;
use chrono::TimeZone;
use std::collections::hash_map::Keys;
use std::collections::HashMap;
use timetracker_core::entries::Entry;
use timetracker_core::entries::EntryStatus;
use timetracker_core::format_short_executable_name;

pub type DateTimeLocalPair = (
    chrono::DateTime<chrono::Local>,
    chrono::DateTime<chrono::Local>,
);

// At ILM the work week starts at Monday, so we mimic the same
// convention.
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

pub fn sum_entry_duration(entries: &[Entry], only_status: EntryStatus) -> chrono::Duration {
    let mut total_duration_seconds = 0;
    for entry in entries {
        if entry.status != only_status {
            continue;
        }
        total_duration_seconds += entry.duration_seconds;
    }
    let total_duration = chrono::Duration::seconds(total_duration_seconds.try_into().unwrap());
    total_duration
}

pub fn sum_entry_show_shot_task_duration(
    entries: &[Entry],
    only_status: EntryStatus,
) -> HashMap<String, chrono::Duration> {
    let mut map = HashMap::<String, chrono::Duration>::new();

    for entry in entries {
        if entry.status != only_status {
            continue;
        }

        let key = format!(
            "{:?} {:?} {:?} {:?}",
            entry.vars.var1_value,
            entry.vars.var2_value,
            entry.vars.var3_value,
            entry.vars.var4_value
        );

        match map.get_mut(&key) {
            Some(value) => {
                let duration =
                    chrono::Duration::seconds(entry.duration_seconds.try_into().unwrap());
                let total = value.checked_add(&duration).unwrap();
                map.insert(key, total);
            }
            None => {
                let duration =
                    chrono::Duration::seconds(entry.duration_seconds.try_into().unwrap());
                map.insert(key, duration);
            }
        };
    }

    map
}

pub fn sum_entry_executable_duration(
    entries: &[Entry],
    only_status: EntryStatus,
) -> HashMap<String, chrono::Duration> {
    let mut map = HashMap::<String, chrono::Duration>::new();

    for entry in entries {
        if entry.status != only_status {
            continue;
        }
        // TODO: Get the real executable value from the id.
        let executable_value = format!("{:?}", entry.vars.executable).to_string();
        let key = format_short_executable_name(&executable_value).to_string();
        match map.get_mut(&key) {
            Some(value) => {
                let duration =
                    chrono::Duration::seconds(entry.duration_seconds.try_into().unwrap());
                let total = value.checked_add(&duration).unwrap();
                map.insert(key, total);
            }
            None => {
                let duration =
                    chrono::Duration::seconds(entry.duration_seconds.try_into().unwrap());
                map.insert(key, duration);
            }
        };
    }

    map
}

pub fn get_map_keys_sorted(map_keys: &mut Keys<String, chrono::Duration>) -> Vec<String> {
    let mut sorted_keys = Vec::new();
    for key in map_keys.into_iter() {
        // Ignores 'unknown' tasks; tasks without a valid value.
        if key.len() > 0 {
            sorted_keys.push(key.clone());
        }
    }
    sorted_keys.sort();
    sorted_keys
}

#[cfg(test)]
mod tests {

    use crate::utils::*;

    #[test]
    fn test_get_map_keys_sorted() {
        let mut map = std::collections::HashMap::<String, chrono::Duration>::new();
        map.insert("key".to_string(), chrono::Duration::seconds(1));
        map.insert("key2".to_string(), chrono::Duration::seconds(1));
        map.insert("".to_string(), chrono::Duration::seconds(1));
        let sorted_keys = get_map_keys_sorted(&mut map.keys());
        assert_eq!(sorted_keys.len(), 2);
        assert_eq!(sorted_keys[0], "key");
        assert_eq!(sorted_keys[1], "key2");
    }
}
