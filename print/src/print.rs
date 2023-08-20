use crate::utils::get_map_keys_sorted;
use crate::utils::get_week_datetime_local;
use crate::utils::get_weekdays_datetime_local;
use crate::utils::sum_entry_duration;
use crate::utils::sum_entry_executable_duration;
use crate::utils::sum_entry_show_shot_task_duration;
use crate::utils::DateTimeLocalPair;
use anyhow::Result;
use chrono::Datelike;
use timetracker_core::entries::Entry;
use timetracker_core::entries::EntryStatus;
use timetracker_core::format::format_date;
use timetracker_core::format::format_duration;
use timetracker_core::format::DateTimeFormat;
use timetracker_core::format::DurationFormat;
use timetracker_core::format::FirstDayOfWeek;
use timetracker_core::format::TimeDuration;
use timetracker_core::storage::Storage;

fn combine_start_end_lines(
    lines: &mut Vec<String>,
    lines_start: &Vec<String>,
    lines_end: &Vec<String>,
    middle_string: &String,
) {
    let mut line_start_max_width = 0;
    for line_start in lines_start.iter() {
        line_start_max_width = std::cmp::max(line_start_max_width, line_start.len());
    }

    for (line_start, line_end) in lines_start.iter().zip(lines_end.iter()) {
        let extra_size = line_start_max_width - line_start.len();
        let mut extra = middle_string.clone();
        for _i in 0..extra_size {
            extra = " ".to_owned() + &extra;
        }
        let line = format!("{line_start}{extra}{line_end}");
        lines.push(line);
    }
}

fn generate_week(
    storage: &mut Storage,
    lines: &mut Vec<String>,
    line_prefix: &str,
    week_datetime_pair: DateTimeLocalPair,
    datetime_format: DateTimeFormat,
    duration_format: DurationFormat,
) -> Result<()> {
    let (week_start_datetime, week_end_datetime) = week_datetime_pair;

    let week_start_of_time = week_start_datetime.timestamp() as u64;
    let week_end_of_time = week_end_datetime.timestamp() as u64;
    let week_entries = storage.read_entries(week_start_of_time, week_end_of_time)?;

    let week_total_duration = sum_entry_duration(&week_entries, EntryStatus::Active);
    let week_start_date_text = format_date(week_start_datetime, datetime_format);
    let week_end_date_text = format_date(week_end_datetime, datetime_format);
    let week_total_duration_text = format_duration(week_total_duration, duration_format);

    let line = format!(
        "{}{} to {} | Total {}",
        line_prefix, week_start_date_text, week_end_date_text, week_total_duration_text
    )
    .to_string();
    lines.push(line);
    Ok(())
}

fn generate_weekday(
    storage: &mut Storage,
    lines: &mut Vec<String>,
    line_prefix: &str,
    week_datetime_pair: DateTimeLocalPair,
    datetime_format: DateTimeFormat,
    duration_format: DurationFormat,
) -> Result<()> {
    let (week_start_datetime, week_end_datetime) = week_datetime_pair;

    let mut lines_start = Vec::new();
    let mut lines_end = Vec::new();

    let weekdays_datetime_pairs =
        get_weekdays_datetime_local(week_start_datetime, week_end_datetime);
    for (weekday, weekdays_datetime_pair) in weekdays_datetime_pairs {
        let (weekday_start_datetime, weekday_end_datetime) = weekdays_datetime_pair;

        let start_of_time = weekday_start_datetime.timestamp() as u64;
        let end_of_time = weekday_end_datetime.timestamp() as u64;
        let entries = storage.read_entries(start_of_time, end_of_time)?;

        let total_duration = sum_entry_duration(&entries, EntryStatus::Active);
        let total_duration_text = format_duration(total_duration, duration_format);
        let line_start = format!(
            "{}{} {}",
            line_prefix,
            weekday,
            format_date(weekday_start_datetime, datetime_format),
        )
        .to_string();
        let line_end = format!("Total {}", total_duration_text).to_string();

        lines_start.push(line_start);
        lines_end.push(line_end);
    }

    let middle_string = " | ".to_string();
    combine_start_end_lines(lines, &lines_start, &lines_end, &middle_string);
    Ok(())
}

fn generate_entry_task_lines(
    entries: &[Entry],
    lines_start: &mut Vec<String>,
    lines_end: &mut Vec<String>,
    line_prefix: &str,
    _datetime_format: DateTimeFormat,
    duration_format: DurationFormat,
) {
    let duration_map = sum_entry_show_shot_task_duration(&entries, EntryStatus::Active);
    let mut keys = duration_map.keys();
    let sorted_keys = get_map_keys_sorted(&mut keys);

    for key in sorted_keys {
        match duration_map.get(&key) {
            Some(value) => {
                let duration_text = format_duration(*value, duration_format);
                let line_start = format!("{}- {}", line_prefix, key).to_string();
                let line_end = duration_text.clone();

                lines_start.push(line_start);
                lines_end.push(line_end);
            }

            // This branch should logically never be run
            // because the key must so we can iterate over the
            // map.
            None => (),
        }
    }
    // Print unknown "other" durations, when the show/shot/task could
    // not be found.
    let empty_key = String::new();
    match duration_map.get(&empty_key) {
        Some(value) => {
            let duration_text = format_duration(*value, duration_format);
            let line_start = format!("{}- other", line_prefix).to_string();
            let line_end = duration_text.clone();

            lines_start.push(line_start);
            lines_end.push(line_end);
        }
        None => (),
    }
}

fn generate_week_task(
    storage: &mut Storage,
    lines: &mut Vec<String>,
    line_prefix: &str,
    week_datetime_pair: DateTimeLocalPair,
    datetime_format: DateTimeFormat,
    duration_format: DurationFormat,
) -> Result<()> {
    let (week_start_datetime, week_end_datetime) = week_datetime_pair;

    let week_start_of_time = week_start_datetime.timestamp() as u64;
    let week_end_of_time = week_end_datetime.timestamp() as u64;
    let week_entries = storage.read_entries(week_start_of_time, week_end_of_time)?;

    let mut lines_start = Vec::new();
    let mut lines_end = Vec::new();

    // Group entries by shot/shot/task name and print details.
    generate_entry_task_lines(
        &week_entries,
        &mut lines_start,
        &mut lines_end,
        line_prefix,
        datetime_format,
        duration_format,
    );

    let middle_string = " ".to_string();
    combine_start_end_lines(lines, &lines_start, &lines_end, &middle_string);
    Ok(())
}

fn generate_weekday_task(
    storage: &mut Storage,
    lines: &mut Vec<String>,
    line_prefix: &str,
    week_datetime_pair: DateTimeLocalPair,
    datetime_format: DateTimeFormat,
    duration_format: DurationFormat,
) -> Result<()> {
    let (week_start_datetime, week_end_datetime) = week_datetime_pair;

    let weekdays_datetime_pairs =
        get_weekdays_datetime_local(week_start_datetime, week_end_datetime);
    for (weekday, weekdays_datetime_pair) in weekdays_datetime_pairs {
        let (weekday_start_datetime, weekday_end_datetime) = weekdays_datetime_pair;

        let start_of_time = weekday_start_datetime.timestamp() as u64;
        let end_of_time = weekday_end_datetime.timestamp() as u64;
        let entries = storage.read_entries(start_of_time, end_of_time)?;

        let total_duration = sum_entry_duration(&entries, EntryStatus::Active);
        let total_duration_text = format_duration(total_duration, duration_format);
        let line = format!(
            "{}{} {} | Total {}",
            line_prefix,
            weekday,
            format_date(weekday_start_datetime, datetime_format),
            total_duration_text
        )
        .to_string();
        lines.push(line);

        let mut lines_start = Vec::new();
        let mut lines_end = Vec::new();

        let line_indent2 = format!("{} ", line_prefix);
        generate_entry_task_lines(
            &entries,
            &mut lines_start,
            &mut lines_end,
            &line_indent2,
            datetime_format,
            duration_format,
        );

        let middle_string = " ".to_string();
        combine_start_end_lines(lines, &lines_start, &lines_end, &middle_string);
    }
    Ok(())
}

fn generate_entry_software_lines(
    entries: &[Entry],
    lines: &mut Vec<String>,
    line_prefix: &str,
    _datetime_format: DateTimeFormat,
    duration_format: DurationFormat,
) {
    let executable_duration_map = sum_entry_executable_duration(&entries, EntryStatus::Active);
    let mut keys = executable_duration_map.keys();
    let sorted_keys = get_map_keys_sorted(&mut keys);

    let mut lines_start = Vec::new();
    let mut lines_end = Vec::new();

    for key in &sorted_keys {
        match executable_duration_map.get(key) {
            Some(value) => {
                let duration_text = format_duration(*value, duration_format);
                let line_start = format!("{}- {}", line_prefix, key).to_string();
                let line_end = duration_text.clone();

                lines_start.push(line_start);
                lines_end.push(line_end);
            }

            // This branch should logically never be run
            // because the key must so we can iterate over the
            // map.
            None => (),
        }
    }
    // Print unknown "other" durations, when the show/shot/task
    // could not be found.
    let empty_key = String::new();
    match executable_duration_map.get(&empty_key) {
        Some(value) => {
            let duration_text = format_duration(*value, duration_format);
            let line_start = format!("{}- other", line_prefix).to_string();
            let line_end = duration_text.clone();

            lines_start.push(line_start);
            lines_end.push(line_end);
        }
        None => (),
    }

    let middle_string = " ".to_string();
    combine_start_end_lines(lines, &lines_start, &lines_end, &middle_string);
}

fn generate_week_software(
    storage: &mut Storage,
    lines: &mut Vec<String>,
    line_prefix: &str,
    week_datetime_pair: DateTimeLocalPair,
    datetime_format: DateTimeFormat,
    duration_format: DurationFormat,
) -> Result<()> {
    let (week_start_datetime, week_end_datetime) = week_datetime_pair;

    let week_start_of_time = week_start_datetime.timestamp() as u64;
    let week_end_of_time = week_end_datetime.timestamp() as u64;
    let week_entries = storage.read_entries(week_start_of_time, week_end_of_time)?;

    // Group entries by shot/shot/task name and print details.
    generate_entry_software_lines(
        &week_entries,
        lines,
        line_prefix,
        datetime_format,
        duration_format,
    );

    Ok(())
}

/// Get the week-number to print, taking the relative number given by
/// the user into account.
pub fn get_relative_week_start_end(relative_week_index: i32) -> DateTimeLocalPair {
    let today_local_timezone = chrono::Local::now();
    let today_iso_week = today_local_timezone.iso_week();
    let today_week_num: u32 = (today_iso_week.week() as i64 + relative_week_index as i64)
        .clamp(u32::MIN.into(), u32::MAX.into())
        .try_into()
        .unwrap();
    let today_year = today_local_timezone.year();
    let week_datetime_pair = get_week_datetime_local(today_year, today_week_num);
    week_datetime_pair
}

/// Prints the time entries with the various settings given.
///
/// 'relative_week_index' is added to the week number to find. A value of '-1'
/// will get the previous week, a value of '0' will get the current
/// week, and a value of '1' will get the next week (which shouldn't
/// really give any results, so it's probably pointless).
pub fn print_entries(
    storage: &mut Storage,
    relative_week_index: i32,
    datetime_format: DateTimeFormat,
    duration_format: DurationFormat,
    display_week: bool,
    display_weekday: bool,
    display_week_task: bool,
    display_weekday_task: bool,
    display_week_software: bool,
) -> Result<()> {
    let mut lines = Vec::new();
    let line_indent = " ";

    let week_datetime_pair = get_relative_week_start_end(relative_week_index);

    if display_week {
        lines.push("Week:".to_string());
        generate_week(
            storage,
            &mut lines,
            line_indent,
            week_datetime_pair,
            datetime_format,
            duration_format,
        )?;
        lines.push("".to_string());
    }

    if display_weekday {
        lines.push("Weekdays:".to_string());
        generate_weekday(
            storage,
            &mut lines,
            line_indent,
            week_datetime_pair,
            datetime_format,
            duration_format,
        )?;
        lines.push("".to_string());
    }

    if display_week_task {
        lines.push("Week Tasks:".to_string());
        generate_week_task(
            storage,
            &mut lines,
            line_indent,
            week_datetime_pair,
            datetime_format,
            duration_format,
        )?;
        lines.push("".to_string());
    }

    if display_weekday_task {
        lines.push("Weekday Tasks:".to_string());
        generate_weekday_task(
            storage,
            &mut lines,
            line_indent,
            week_datetime_pair,
            datetime_format,
            duration_format,
        )?;
        lines.push("".to_string());
    }

    if display_week_software {
        lines.push("Week Software:".to_string());
        generate_week_software(
            storage,
            &mut lines,
            line_indent,
            week_datetime_pair,
            datetime_format,
            duration_format,
        )?;
        lines.push("".to_string());
    }

    storage.close();

    for line in &lines {
        println!("{}", line);
    }

    Ok(())
}

/// Prints the time entries with the various settings given.
pub fn print_preset(
    _storage: &mut Storage,
    _week_datetime_pair: DateTimeLocalPair,
    // filter_executable: bool,
    // filter_env_vars: Vec<String>,
    _time_duration: TimeDuration,
    _datetime_format: DateTimeFormat,
    _duration_format: DurationFormat,
    _first_day_of_week: FirstDayOfWeek,
    // output_stream: MyOutputStream
) -> Result<()> {
    // let mut lines = Vec::new();
    // for line in &lines {
    //     println!("{}", line);
    // }
    Ok(())
}
