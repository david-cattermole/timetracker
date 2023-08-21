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
use timetracker_core::format::format_time_no_seconds;
use timetracker_core::format::DateTimeFormat;
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

fn utc_seconds_rounded_as_time_str(
    utc_time_seconds: u64,
    time_block_unit: TimeBlockUnit,
    datetime_format: DateTimeFormat,
) -> String {
    let datetime = utc_seconds_to_datetime_local(utc_time_seconds);

    let increment_minutes = time_block_unit.as_minutes();
    let number = ((datetime.minute() as f32) / (increment_minutes as f32)).trunc() as u64;
    let datetime = datetime
        .with_minute((number * increment_minutes).try_into().unwrap())
        .unwrap()
        .with_second(0)
        .unwrap();

    format_time_no_seconds(datetime, datetime_format)
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
    datetime_format: DateTimeFormat,
    only_status: EntryStatus,
) -> HashMap<String, chrono::Duration> {
    let mut map = HashMap::<String, chrono::Duration>::new();

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

        let key_current =
            utc_seconds_rounded_as_time_str(seconds_current, time_block_unit, datetime_format);
        let key_previous =
            utc_seconds_rounded_as_time_str(seconds_previous, time_block_unit, datetime_format);
        let key_next =
            utc_seconds_rounded_as_time_str(seconds_next, time_block_unit, datetime_format);

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
        for seconds in (seconds_min..seconds_max).skip(increment_seconds) {
            let key = utc_seconds_rounded_as_time_str(seconds, time_block_unit, datetime_format);
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

pub fn get_map_keys_sorted<T>(map_keys: &Keys<String, T>) -> Vec<String> {
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
