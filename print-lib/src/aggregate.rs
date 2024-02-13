use crate::datetime::utc_seconds_to_datetime_local;
use crate::datetime::DateTimeLocalPair;
use crate::variable::combine_variable_values;
use crate::variable::multi_variable_values;
use crate::variable::Variable;

use chrono::Timelike;
use std::collections::hash_map::Keys;
use std::collections::HashMap;
use timetracker_core::entries::Entry;
use timetracker_core::entries::EntryStatus;
use timetracker_core::format::TimeBlockUnit;

pub fn sum_entry_duration(entries: &[Entry], only_status: EntryStatus) -> chrono::Duration {
    let mut total_duration_seconds = 0;
    for entry in entries {
        if entry.status != only_status {
            continue;
        }
        total_duration_seconds += entry.duration_seconds;
    }

    chrono::Duration::seconds(total_duration_seconds.try_into().unwrap())
}

pub fn sum_entry_variables_duration(
    entries: &[Entry],
    variables: &[Variable],
    only_status: EntryStatus,
) -> HashMap<String, (Vec<String>, chrono::Duration)> {
    let mut map = HashMap::<String, (Vec<String>, chrono::Duration)>::new();

    for entry in entries {
        if entry.status != only_status {
            continue;
        }

        let key = combine_variable_values(entry, variables);
        let vars = multi_variable_values(entry, variables);

        match map.get_mut(&key) {
            Some((_vars, old_duration)) => {
                let duration =
                    chrono::Duration::seconds(entry.duration_seconds.try_into().unwrap());
                let total = old_duration.checked_add(&duration).unwrap();
                map.insert(key, (vars, total));
            }
            None => {
                let duration =
                    chrono::Duration::seconds(entry.duration_seconds.try_into().unwrap());
                map.insert(key, (vars, duration));
            }
        };
    }

    map
}

pub fn sum_entry_executable_duration(
    entries: &[Entry],
    only_status: EntryStatus,
) -> HashMap<String, (Vec<String>, chrono::Duration)> {
    let variables = vec![Variable::Executable; 1];
    sum_entry_variables_duration(entries, &variables, only_status)
}

fn utc_seconds_rounded(
    utc_time_seconds: u64,
    time_block_unit: TimeBlockUnit,
) -> chrono::DateTime<chrono::Local> {
    let datetime = utc_seconds_to_datetime_local(utc_time_seconds);

    let increment_minutes = time_block_unit.as_minutes();
    let number = ((datetime.minute() as f32) / (increment_minutes as f32)).trunc() as u64;
    datetime
        .with_minute((number * increment_minutes).try_into().unwrap())
        .unwrap()
        .with_second(0)
        .unwrap()
}

fn add_min(value_min: &mut u64, value_previous: u64) {
    if value_previous < *value_min {
        *value_min = value_previous;
    }
}

fn add_max(value_max: &mut u64, value_next: u64) {
    if value_next > *value_max {
        *value_max = value_next;
    }
}

pub fn sum_entry_activity_duration(
    entries: &[Entry],
    start_end_datetime_pairs: DateTimeLocalPair,
    add_fringe_datetimes: bool,
    fill_datetimes_gaps: bool,
    time_block_unit: TimeBlockUnit,
    only_status: EntryStatus,
) -> HashMap<chrono::NaiveTime, chrono::Duration> {
    let mut map = HashMap::<chrono::NaiveTime, chrono::Duration>::new();

    let mut seconds_min = u64::MAX;
    let mut seconds_max = u64::MIN;

    let mut fringe_keys = Vec::new();
    for entry in entries {
        if entry.status != only_status {
            continue;
        }

        let increment_seconds = (time_block_unit.as_minutes() * 60) + 1;
        let seconds_current = entry.utc_time_seconds;
        let seconds_previous = seconds_current - increment_seconds;
        let seconds_next = seconds_current + increment_seconds;

        let key_current = utc_seconds_rounded(seconds_current, time_block_unit).time();
        let key_previous = utc_seconds_rounded(seconds_previous, time_block_unit).time();
        let key_next = utc_seconds_rounded(seconds_next, time_block_unit).time();

        let (start_datetime, end_datetime) = start_end_datetime_pairs;
        let datetime_previous = utc_seconds_to_datetime_local(seconds_previous);
        let datetime_next = utc_seconds_to_datetime_local(seconds_next);

        add_min(&mut seconds_min, seconds_current);
        add_max(&mut seconds_max, seconds_current);

        if add_fringe_datetimes {
            if datetime_previous >= start_datetime {
                add_min(&mut seconds_min, seconds_previous);
                fringe_keys.push(key_previous);
            }
            if datetime_next <= end_datetime {
                add_max(&mut seconds_max, seconds_next);
                fringe_keys.push(key_next);
            }
        }

        match map.get_mut(&key_current) {
            Some(value) => {
                let duration =
                    chrono::Duration::seconds(entry.duration_seconds.try_into().unwrap());
                let total = value.checked_add(&duration).unwrap();
                map.insert(key_current, total);
            }
            None => {
                let duration =
                    chrono::Duration::seconds(entry.duration_seconds.try_into().unwrap());
                map.insert(key_current, duration);
            }
        };
    }

    // Initialize the previous and next increments of time with empty values.
    for fringe_key in fringe_keys {
        match map.get(&fringe_key) {
            Some(_) => (),
            None => {
                let empty_duration = chrono::Duration::seconds(0);
                map.insert(fringe_key, empty_duration);
            }
        };
    }

    if fill_datetimes_gaps {
        let increment_seconds = ((time_block_unit.as_minutes() * 60) - 1) as usize;
        for seconds in (seconds_min..seconds_max).step_by(increment_seconds) {
            let key = utc_seconds_rounded(seconds, time_block_unit).time();

            match map.get(&key) {
                Some(_) => (),
                None => {
                    let empty_duration = chrono::Duration::seconds(0);
                    map.insert(key, empty_duration);
                }
            };
        }
    }

    map
}

pub fn get_map_keys_sorted_general<KeyType: Clone + Ord, ValueType: Clone>(
    map_keys: &Keys<KeyType, ValueType>,
) -> Vec<KeyType> {
    let mut sorted_keys = Vec::new();
    for key in map_keys.clone() {
        sorted_keys.push(key.clone());
    }
    sorted_keys.sort();
    sorted_keys
}

pub fn get_map_keys_sorted_strings<T>(map_keys: &Keys<String, T>) -> Vec<String> {
    let mut sorted_keys = Vec::new();
    for key in map_keys.clone() {
        // Ignores 'unknown' tasks; tasks without a valid value.
        if !key.is_empty() {
            sorted_keys.push(key.clone());
        }
    }
    sorted_keys.sort();
    sorted_keys
}

#[cfg(test)]
mod tests {

    use crate::aggregate::*;
    use timetracker_core::format::format_time_no_seconds;
    use timetracker_core::format::DateTimeFormat;

    #[test]
    fn test_get_map_keys_sorted_strings() {
        let mut map = std::collections::HashMap::<String, chrono::Duration>::new();
        map.insert("key".to_string(), chrono::Duration::seconds(1));
        map.insert("key2".to_string(), chrono::Duration::seconds(1));
        map.insert("".to_string(), chrono::Duration::seconds(1));
        let sorted_keys = get_map_keys_sorted_strings(&mut map.keys());
        assert_eq!(sorted_keys.len(), 2);
        assert_eq!(sorted_keys[0], "key");
        assert_eq!(sorted_keys[1], "key2");
    }

    fn generate_sorted_datetimes() -> Vec<chrono::DateTime<chrono::Utc>> {
        let mut map = std::collections::HashMap::new();

        let datetime1 = "2023-08-25T01:00:00Z"
            .parse::<chrono::DateTime<chrono::Utc>>()
            .unwrap();
        let datetime2 = "2023-08-25T02:00:00Z"
            .parse::<chrono::DateTime<chrono::Utc>>()
            .unwrap();
        let datetime3 = "2023-08-25T11:00:00Z"
            .parse::<chrono::DateTime<chrono::Utc>>()
            .unwrap();
        let datetime4 = "2023-08-25T13:00:00Z"
            .parse::<chrono::DateTime<chrono::Utc>>()
            .unwrap();
        let datetime5 = "2023-08-25T15:00:00Z"
            .parse::<chrono::DateTime<chrono::Utc>>()
            .unwrap();
        let datetime6 = "2023-08-25T16:00:00Z"
            .parse::<chrono::DateTime<chrono::Utc>>()
            .unwrap();
        let datetime7 = "2023-08-25T23:00:00Z"
            .parse::<chrono::DateTime<chrono::Utc>>()
            .unwrap();
        map.insert(datetime1, chrono::Duration::seconds(1));
        map.insert(datetime2, chrono::Duration::seconds(1));
        map.insert(datetime3, chrono::Duration::seconds(1));
        map.insert(datetime4, chrono::Duration::seconds(1));
        map.insert(datetime5, chrono::Duration::seconds(1));
        map.insert(datetime6, chrono::Duration::seconds(1));
        map.insert(datetime7, chrono::Duration::seconds(1));

        let sorted_keys = get_map_keys_sorted_general(&mut map.keys());
        assert_eq!(sorted_keys.len(), 7);
        for (i, key) in sorted_keys.iter().enumerate() {
            println!("sorted key {} = {}", i, key);
        }

        sorted_keys
    }

    #[test]
    fn test_get_map_keys_sorted_general_iso_format() {
        let sorted_keys = generate_sorted_datetimes();

        let datetime_format = DateTimeFormat::Iso;
        let sorted_string1 = format_time_no_seconds(sorted_keys[0], datetime_format);
        let sorted_string2 = format_time_no_seconds(sorted_keys[1], datetime_format);
        let sorted_string3 = format_time_no_seconds(sorted_keys[2], datetime_format);
        let sorted_string4 = format_time_no_seconds(sorted_keys[3], datetime_format);
        let sorted_string5 = format_time_no_seconds(sorted_keys[4], datetime_format);
        let sorted_string6 = format_time_no_seconds(sorted_keys[5], datetime_format);
        let sorted_string7 = format_time_no_seconds(sorted_keys[6], datetime_format);
        assert_eq!(sorted_string1, "01:00");
        assert_eq!(sorted_string2, "02:00");
        assert_eq!(sorted_string3, "11:00");
        assert_eq!(sorted_string4, "13:00");
        assert_eq!(sorted_string5, "15:00");
        assert_eq!(sorted_string6, "16:00");
        assert_eq!(sorted_string7, "23:00");
    }

    #[test]
    fn test_get_map_keys_sorted_general_usa_format() {
        let sorted_keys = generate_sorted_datetimes();

        let datetime_format = DateTimeFormat::UsaMonthDayYear;
        let sorted_string1 = format_time_no_seconds(sorted_keys[0], datetime_format);
        let sorted_string2 = format_time_no_seconds(sorted_keys[1], datetime_format);
        let sorted_string3 = format_time_no_seconds(sorted_keys[2], datetime_format);
        let sorted_string4 = format_time_no_seconds(sorted_keys[3], datetime_format);
        let sorted_string5 = format_time_no_seconds(sorted_keys[4], datetime_format);
        let sorted_string6 = format_time_no_seconds(sorted_keys[5], datetime_format);
        let sorted_string7 = format_time_no_seconds(sorted_keys[6], datetime_format);
        assert_eq!(sorted_string1, "01:00 AM");
        assert_eq!(sorted_string2, "02:00 AM");
        assert_eq!(sorted_string3, "11:00 AM");
        assert_eq!(sorted_string4, "01:00 PM");
        assert_eq!(sorted_string5, "03:00 PM");
        assert_eq!(sorted_string6, "04:00 PM");
        assert_eq!(sorted_string7, "11:00 PM");
    }
}
