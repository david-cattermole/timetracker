use crate::aggregate::get_map_keys_sorted_general;
use crate::aggregate::get_map_keys_sorted_strings;
use crate::aggregate::sum_entry_activity_duration;
use crate::aggregate::sum_entry_duration;
use crate::aggregate::sum_entry_executable_duration;
use crate::aggregate::sum_entry_variables_duration;
use crate::datetime::get_week_datetime_local;
use crate::datetime::get_weekdays_datetime_local;
use crate::datetime::DateTimeLocalPair;
use crate::variable::combine_variable_names;
use crate::variable::Variable;

use anyhow::Result;
use chrono::Datelike;
use colored::Colorize;
use log::debug;
use timetracker_core::entries::Entry;
use timetracker_core::entries::EntryStatus;
use timetracker_core::format::format_date;
use timetracker_core::format::format_duration;
use timetracker_core::format::format_naive_time_no_seconds;
use timetracker_core::format::DateTimeFormat;
use timetracker_core::format::DurationFormat;
use timetracker_core::format::PrintType;
use timetracker_core::format::TimeBlockUnit;
use timetracker_core::format::TimeScale;
use timetracker_core::storage::Entries;

const HEADING_TOTAL_TEXT_START: &str = "[total ";
const HEADING_TOTAL_TEXT_END: &str = "]";

fn combine_start_end_lines(
    lines: &mut Vec<String>,
    lines_start: &[String],
    lines_end: &[String],
    middle_string: &str,
) {
    let mut line_start_max_width = 0;
    for line_start in lines_start.iter() {
        line_start_max_width = std::cmp::max(line_start_max_width, line_start.len());
    }

    for (line_start, line_end) in lines_start.iter().zip(lines_end.iter()) {
        let extra_size = line_start_max_width - line_start.len();
        let mut extra = middle_string.to_string();
        for _i in 0..extra_size {
            extra = format!(" {}", extra);
        }
        let line = format!("{line_start}{extra}{line_end}");
        lines.push(line);
    }
}

fn get_longest_string(values: &[String]) -> usize {
    let mut max_width = 0;
    for value in values.iter() {
        max_width = std::cmp::max(max_width, value.len());
    }
    max_width
}

// TODO: Eliminate the generated spaces when a line_mid* value is empty.
fn combine_start_mid_end_lines(
    lines: &mut Vec<String>,
    lines_start: &[String],
    lines_mid1: &[String],
    lines_mid2: &[String],
    lines_mid3: &[String],
    lines_mid4: &[String],
    lines_mid5: &[String],
    lines_end: &[String],
    middle_string: &str,
    end_string: &str,
) {
    let line_start_max_width = get_longest_string(lines_start);
    let line_mid1_max_width = get_longest_string(lines_mid1);
    let line_mid2_max_width = get_longest_string(lines_mid2);
    let line_mid3_max_width = get_longest_string(lines_mid3);
    let line_mid4_max_width = get_longest_string(lines_mid4);
    let line_mid5_max_width = get_longest_string(lines_mid5);

    let mut lines_parts = Vec::<_>::new();
    for i in 0..lines_start.len() {
        let value = (
            lines_start[i].clone(),
            lines_mid1[i].clone(),
            lines_mid2[i].clone(),
            lines_mid3[i].clone(),
            lines_mid4[i].clone(),
            lines_mid5[i].clone(),
            lines_end[i].clone(),
        );
        lines_parts.push(value);
    }

    for (line_start, line_mid1, line_mid2, line_mid3, line_mid4, line_mid5, line_end) in lines_parts
    {
        let start_extra_size = line_start_max_width - line_start.len();
        let mid1_extra_size = line_mid1_max_width - line_mid1.len();
        let mid2_extra_size = line_mid2_max_width - line_mid2.len();
        let mid3_extra_size = line_mid3_max_width - line_mid3.len();
        let mid4_extra_size = line_mid4_max_width - line_mid4.len();
        let mid5_extra_size = line_mid5_max_width - line_mid5.len();

        let mut start_extra = middle_string.to_string();
        let mut mid1_extra = middle_string.to_string();
        let mut mid2_extra = middle_string.to_string();
        let mut mid3_extra = middle_string.to_string();
        let mut mid4_extra = middle_string.to_string();
        let mut mid5_extra = end_string.to_string();

        for _i in 0..start_extra_size {
            start_extra = format!(" {}", start_extra);
        }
        for _i in 0..mid1_extra_size {
            mid1_extra = format!(" {}", mid1_extra);
        }
        for _i in 0..mid2_extra_size {
            mid2_extra = format!(" {}", mid2_extra);
        }
        for _i in 0..mid3_extra_size {
            mid3_extra = format!(" {}", mid3_extra);
        }
        for _i in 0..mid4_extra_size {
            mid4_extra = format!(" {}", mid4_extra);
        }
        for _i in 0..mid5_extra_size {
            mid5_extra = format!(" {}", mid5_extra);
        }

        let line = format!("{line_start}{start_extra}{line_mid1}{mid1_extra}{line_mid2}{mid2_extra}{line_mid3}{mid3_extra}{line_mid4}{mid4_extra}{line_mid5}{mid5_extra}{line_end}");
        lines.push(line);
    }
}

fn generate_summary_week(
    entries: &Entries,
    lines: &mut Vec<String>,
    line_prefix: &str,
    week_datetime_pair: DateTimeLocalPair,
    datetime_format: DateTimeFormat,
    duration_format: DurationFormat,
) -> Result<()> {
    let (week_start_datetime, week_end_datetime) = week_datetime_pair;
    let week_entries = entries.datetime_range_entries(week_start_datetime, week_end_datetime);

    let week_total_duration = sum_entry_duration(&week_entries, EntryStatus::Active);
    let week_start_date_text = format_date(week_start_datetime, datetime_format);
    let week_end_date_text = format_date(week_end_datetime, datetime_format);
    let week_total_duration_text = format_duration(week_total_duration, duration_format);

    let line = format!(
        "{}{} to {} | total {}",
        line_prefix, week_start_date_text, week_end_date_text, week_total_duration_text
    );
    lines.push(line);
    Ok(())
}

fn generate_summary_weekday(
    entries: &Entries,
    lines: &mut Vec<String>,
    line_prefix: &str,
    line_heading: &str,
    week_datetime_pair: DateTimeLocalPair,
    datetime_format: DateTimeFormat,
    duration_format: DurationFormat,
) -> Result<()> {
    let (week_start_datetime, week_end_datetime) = week_datetime_pair;

    let mut lines_start = Vec::new();
    let mut lines_end = Vec::new();

    let mut week_total_duration = chrono::Duration::zero();

    let weekdays_datetime_pairs =
        get_weekdays_datetime_local(week_start_datetime, week_end_datetime);
    for (weekday, weekdays_datetime_pair) in weekdays_datetime_pairs {
        let (weekday_start_datetime, weekday_end_datetime) = weekdays_datetime_pair;
        let weekday_entries =
            entries.datetime_range_entries(weekday_start_datetime, weekday_end_datetime);

        if weekday_entries.is_empty() {
            continue;
        }

        let total_duration = sum_entry_duration(&weekday_entries, EntryStatus::Active);
        week_total_duration = week_total_duration + total_duration;

        let total_duration_text = format_duration(total_duration, duration_format);
        let line_start = format!(
            "{}{} {}",
            line_prefix,
            weekday,
            format_date(weekday_start_datetime, datetime_format),
        )
        .to_string();
        let line_end = format!("total {}", total_duration_text).to_string();

        lines_start.push(line_start);
        lines_end.push(line_end);
    }

    let week_total_duration_text = format_duration(week_total_duration, duration_format);
    lines.push(format!(
        "{} {}{}{}:",
        line_heading, HEADING_TOTAL_TEXT_START, week_total_duration_text, HEADING_TOTAL_TEXT_END
    ));

    let middle_string = " | ".to_string();
    combine_start_end_lines(lines, &lines_start, &lines_end, &middle_string);
    Ok(())
}

fn generate_entry_variables_lines(
    entries: &[Entry],
    lines_start: &mut Vec<String>,
    lines_mid1: &mut Vec<String>,
    lines_mid2: &mut Vec<String>,
    lines_mid3: &mut Vec<String>,
    lines_mid4: &mut Vec<String>,
    lines_mid5: &mut Vec<String>,
    lines_end: &mut Vec<String>,
    line_prefix: &str,
    _datetime_format: DateTimeFormat,
    duration_format: DurationFormat,
    variables: &[Variable],
) {
    let duration_map = sum_entry_variables_duration(entries, variables, EntryStatus::Active);
    let keys = duration_map.keys();
    let sorted_keys = get_map_keys_sorted_strings(&keys);

    for key in sorted_keys {
        if let Some(value) = duration_map.get(&key) {
            let (vars, duration) = value;
            let duration_text = format_duration(*duration, duration_format);
            let line_start = format!("{}-", line_prefix).to_string();

            let line_mid1 = if !vars.is_empty() {
                vars[0].to_string()
            } else {
                "".to_string()
            };

            let line_mid2 = if vars.len() > 1 {
                vars[1].to_string()
            } else {
                "".to_string()
            };

            let line_mid3 = if vars.len() > 2 {
                vars[2].to_string()
            } else {
                "".to_string()
            };

            let line_mid4 = if vars.len() > 3 {
                vars[3].to_string()
            } else {
                "".to_string()
            };

            let line_mid5 = if vars.len() > 4 {
                vars[4].to_string()
            } else {
                "".to_string()
            };

            let line_end = duration_text.clone();

            lines_start.push(line_start);
            lines_mid1.push(line_mid1);
            lines_mid2.push(line_mid2);
            lines_mid3.push(line_mid3);
            lines_mid4.push(line_mid4);
            lines_mid5.push(line_mid5);
            lines_end.push(line_end);
        }
    }

    // Print unknown "other" durations, when the variables could
    // not be found.
    let empty_key = String::new();

    if let Some(value) = duration_map.get(&empty_key) {
        let (vars, duration) = value;
        let duration_text = format_duration(*duration, duration_format);

        let line_start = format!("{}-", line_prefix);

        let line_mid1 = if !vars.is_empty() {
            vars[0].to_string()
        } else {
            "other".to_string()
        };

        let line_mid2 = if vars.len() > 1 {
            vars[1].to_string()
        } else {
            "".to_string()
        };

        let line_mid3 = if vars.len() > 2 {
            vars[2].to_string()
        } else {
            "".to_string()
        };

        let line_mid4 = if vars.len() > 3 {
            vars[3].to_string()
        } else {
            "".to_string()
        };

        let line_mid5 = if vars.len() > 4 {
            vars[4].to_string()
        } else {
            "".to_string()
        };

        let line_end = duration_text;

        lines_start.push(line_start);
        lines_mid1.push(line_mid1);
        lines_mid2.push(line_mid2);
        lines_mid3.push(line_mid3);
        lines_mid4.push(line_mid4);
        lines_mid5.push(line_mid5);
        lines_end.push(line_end);
    }
}

fn generate_variables_week(
    entries: &Entries,
    lines: &mut Vec<String>,
    line_prefix: &str,
    line_heading: &str,
    week_datetime_pair: DateTimeLocalPair,
    datetime_format: DateTimeFormat,
    duration_format: DurationFormat,
    variables: &[Variable],
) -> Result<()> {
    let (week_start_datetime, week_end_datetime) = week_datetime_pair;
    let week_entries = entries.datetime_range_entries(week_start_datetime, week_end_datetime);
    let week_total_duration = sum_entry_duration(&week_entries, EntryStatus::Active);

    let mut lines_start = Vec::new();
    let mut lines_mid1 = Vec::new();
    let mut lines_mid2 = Vec::new();
    let mut lines_mid3 = Vec::new();
    let mut lines_mid4 = Vec::new();
    let mut lines_mid5 = Vec::new();
    let mut lines_end = Vec::new();

    // Group entries by variable name and print details.
    generate_entry_variables_lines(
        &week_entries,
        &mut lines_start,
        &mut lines_mid1,
        &mut lines_mid2,
        &mut lines_mid3,
        &mut lines_mid4,
        &mut lines_mid5,
        &mut lines_end,
        line_prefix,
        datetime_format,
        duration_format,
        variables,
    );

    let week_total_duration_text = format_duration(week_total_duration, duration_format);
    lines.push(format!(
        "{} {}{}{}:",
        line_heading, HEADING_TOTAL_TEXT_START, week_total_duration_text, HEADING_TOTAL_TEXT_END
    ));
    let middle_string = " ".to_string();
    let end_string = " | ".to_string();
    combine_start_mid_end_lines(
        lines,
        &lines_start,
        &lines_mid1,
        &lines_mid2,
        &lines_mid3,
        &lines_mid4,
        &lines_mid5,
        &lines_end,
        &middle_string,
        &end_string,
    );
    Ok(())
}

fn generate_variables_weekday(
    entries: &Entries,
    lines: &mut Vec<String>,
    line_prefix: &str,
    week_datetime_pair: DateTimeLocalPair,
    datetime_format: DateTimeFormat,
    duration_format: DurationFormat,
    variables: &[Variable],
) -> Result<()> {
    let (week_start_datetime, week_end_datetime) = week_datetime_pair;

    let weekdays_datetime_pairs =
        get_weekdays_datetime_local(week_start_datetime, week_end_datetime);
    for (weekday, weekdays_datetime_pair) in weekdays_datetime_pairs {
        let (weekday_start_datetime, weekday_end_datetime) = weekdays_datetime_pair;
        let weekday_entries =
            entries.datetime_range_entries(weekday_start_datetime, weekday_end_datetime);

        if weekday_entries.is_empty() {
            continue;
        }

        let total_duration = sum_entry_duration(&weekday_entries, EntryStatus::Active);
        let total_duration_text = format_duration(total_duration, duration_format);
        let line = format!(
            "{}{} {} {}{}{}",
            line_prefix,
            weekday,
            format_date(weekday_start_datetime, datetime_format),
            HEADING_TOTAL_TEXT_START,
            total_duration_text,
            HEADING_TOTAL_TEXT_END
        )
        .to_string();
        lines.push(line);

        let mut lines_start = Vec::new();
        let mut lines_mid1 = Vec::new();
        let mut lines_mid2 = Vec::new();
        let mut lines_mid3 = Vec::new();
        let mut lines_mid4 = Vec::new();
        let mut lines_mid5 = Vec::new();
        let mut lines_end = Vec::new();

        let line_indent2 = format!("{} ", line_prefix);
        generate_entry_variables_lines(
            &weekday_entries,
            &mut lines_start,
            &mut lines_mid1,
            &mut lines_mid2,
            &mut lines_mid3,
            &mut lines_mid4,
            &mut lines_mid5,
            &mut lines_end,
            &line_indent2,
            datetime_format,
            duration_format,
            variables,
        );

        let middle_string = " ".to_string();
        let end_string = " | ".to_string();
        combine_start_mid_end_lines(
            lines,
            &lines_start,
            &lines_mid1,
            &lines_mid2,
            &lines_mid3,
            &lines_mid4,
            &lines_mid5,
            &lines_end,
            &middle_string,
            &end_string,
        );
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
    let executable_duration_map = sum_entry_executable_duration(entries, EntryStatus::Active);
    let keys = executable_duration_map.keys();
    // TODO: Allow sorting by value, so we can show how much the
    // software was used, starting at the top of the print out (rather
    // than alphabetical).
    let sorted_keys = get_map_keys_sorted_strings(&keys);

    let mut lines_start = Vec::new();
    let mut lines_end = Vec::new();

    for key in &sorted_keys {
        if let Some(value) = executable_duration_map.get(key) {
            let (_vars, duration) = value;
            let duration_text = format_duration(*duration, duration_format);

            let line_start = format!("{}- {}", line_prefix, key);
            let line_end = format!("| {}", duration_text);

            lines_start.push(line_start);
            lines_end.push(line_end);
        }
    }

    // Print unknown "other" durations, when the variables
    // could not be found.
    let empty_key = String::new();
    if let Some(value) = executable_duration_map.get(&empty_key) {
        let (_vars, duration) = value;
        let duration_text = format_duration(*duration, duration_format);
        let line_start = format!("{}- other", line_prefix);
        let line_end = format!("| {}", duration_text);

        lines_start.push(line_start);
        lines_end.push(line_end);
    }

    let middle_string = " ".to_string();
    combine_start_end_lines(lines, &lines_start, &lines_end, &middle_string);
}

fn generate_software_week(
    entries: &Entries,
    lines: &mut Vec<String>,
    line_prefix: &str,
    line_heading: &str,
    week_datetime_pair: DateTimeLocalPair,
    datetime_format: DateTimeFormat,
    duration_format: DurationFormat,
) -> Result<()> {
    let (week_start_datetime, week_end_datetime) = week_datetime_pair;
    let week_entries = entries.datetime_range_entries(week_start_datetime, week_end_datetime);

    let week_total_duration = sum_entry_duration(&week_entries, EntryStatus::Active);
    let week_total_duration_text = format_duration(week_total_duration, duration_format);
    lines.push(format!(
        "{} {}{}{}:",
        line_heading, HEADING_TOTAL_TEXT_START, week_total_duration_text, HEADING_TOTAL_TEXT_END
    ));

    // Group entries by name and print details.
    generate_entry_software_lines(
        &week_entries,
        lines,
        line_prefix,
        datetime_format,
        duration_format,
    );

    Ok(())
}

fn generate_software_weekday(
    entries: &Entries,
    lines: &mut Vec<String>,
    line_prefix: &str,
    week_datetime_pair: DateTimeLocalPair,
    datetime_format: DateTimeFormat,
    duration_format: DurationFormat,
) -> Result<()> {
    let (week_start_datetime, week_end_datetime) = week_datetime_pair;

    let weekday_datetime_pairs =
        get_weekdays_datetime_local(week_start_datetime, week_end_datetime);

    for (weekday, weekday_datetime_pair) in weekday_datetime_pairs {
        let (weekday_start_datetime, weekday_end_datetime) = weekday_datetime_pair;
        let weekday_entries =
            entries.datetime_range_entries(weekday_start_datetime, weekday_end_datetime);

        if weekday_entries.is_empty() {
            continue;
        }

        let date_string = format_date(week_start_datetime, datetime_format);

        let weekday_total_duration = sum_entry_duration(&weekday_entries, EntryStatus::Active);
        let weekday_total_duration_text = format_duration(weekday_total_duration, duration_format);
        lines.push(format!(
            "{} {} {}{}{}:",
            weekday,
            date_string,
            HEADING_TOTAL_TEXT_START,
            weekday_total_duration_text,
            HEADING_TOTAL_TEXT_END
        ));

        // Group entries by name and print details.
        generate_entry_software_lines(
            &weekday_entries,
            lines,
            line_prefix,
            datetime_format,
            duration_format,
        );
    }

    Ok(())
}

fn generate_entry_activity_lines(
    entries: &[Entry],
    lines: &mut Vec<String>,
    line_prefix: &str,
    datetime_format: DateTimeFormat,
    _duration_format: DurationFormat,
    bar_graph_character_num_width: u8,
    weekday_datetime_pair: DateTimeLocalPair,
    time_block_unit: TimeBlockUnit,
    color: Option<colored::Color>,
) {
    let add_fringe_datetimes = false;
    let fill_datetimes_gaps = true;
    let duration_map = sum_entry_activity_duration(
        entries,
        weekday_datetime_pair,
        add_fringe_datetimes,
        fill_datetimes_gaps,
        time_block_unit,
        EntryStatus::Active,
    );
    let sorted_keys = get_map_keys_sorted_general(&duration_map.keys());

    let mut lines_start = Vec::new();
    let mut lines_end = Vec::new();

    for key in &sorted_keys {
        if let Some(value) = duration_map.get(key) {
            let increment_minutes = time_block_unit.as_minutes();
            let mut num_minutes: u64 = value.num_minutes().try_into().unwrap();
            if num_minutes > increment_minutes {
                // This should not be possible - how can it be
                // possible that we've recorded more active time
                // in the time slot than physically possible?
                num_minutes = increment_minutes;
            }
            let duration_ratio = (num_minutes as f32) / (increment_minutes as f32);
            let duration_ratio_scaled = duration_ratio * (bar_graph_character_num_width as f32);
            let duration_ratio_round = duration_ratio_scaled.round() as u32;

            let mut duration_text = String::new();

            for num in 0..bar_graph_character_num_width {
                let check = (num as u32) < duration_ratio_round;
                let character = match check {
                    true => "-",
                    false => " ",
                };
                let character_string = match color {
                    Some(c) => character.color(c).to_string(),
                    None => character.to_string(),
                };
                duration_text.push_str(&character_string);
            }
            duration_text.push_str(&format!(" | {:2}m", num_minutes).to_string());

            let key_string = format_naive_time_no_seconds(*key, datetime_format);
            let line_start = format!("{}- {}", line_prefix, key_string).to_string();
            let line_end = duration_text.clone();

            lines_start.push(line_start);
            lines_end.push(line_end);
        }
    }

    let middle_string = " ".to_string();
    combine_start_end_lines(lines, &lines_start, &lines_end, &middle_string);
}

fn generate_activity_weekday(
    entries: &Entries,
    lines: &mut Vec<String>,
    line_prefix: &str,
    week_datetime_pair: DateTimeLocalPair,
    datetime_format: DateTimeFormat,
    duration_format: DurationFormat,
    time_block_unit: TimeBlockUnit,
    bar_graph_character_num_width: u8,
    color: Option<colored::Color>,
) -> Result<()> {
    let (week_start_datetime, week_end_datetime) = week_datetime_pair;

    let weekday_datetime_pairs =
        get_weekdays_datetime_local(week_start_datetime, week_end_datetime);

    for (weekday, weekday_datetime_pair) in weekday_datetime_pairs {
        let (weekday_start_datetime, weekday_end_datetime) = weekday_datetime_pair;
        let weekday_entries =
            entries.datetime_range_entries(weekday_start_datetime, weekday_end_datetime);

        if weekday_entries.is_empty() {
            continue;
        }

        let date_string = format_date(week_start_datetime, datetime_format);

        let weekday_total_duration = sum_entry_duration(&weekday_entries, EntryStatus::Active);
        let weekday_total_duration_text = format_duration(weekday_total_duration, duration_format);
        lines.push(format!(
            "{} {} {}{}{}",
            weekday,
            date_string,
            HEADING_TOTAL_TEXT_START,
            weekday_total_duration_text,
            HEADING_TOTAL_TEXT_END
        ));

        // Group entries by name and print details.
        generate_entry_activity_lines(
            &weekday_entries,
            lines,
            line_prefix,
            datetime_format,
            duration_format,
            bar_graph_character_num_width,
            weekday_datetime_pair,
            time_block_unit,
            color,
        );
    }

    Ok(())
}

fn generate_duration_bins_text(
    duration_bins_normalized: &Vec<f32>,
    use_unicode_blocks: bool,
    color: Option<colored::Color>,
) -> String {
    let mut duration_text = String::new();
    duration_text.push('[');

    for duration_ratio in duration_bins_normalized {
        let duration_ratio = *duration_ratio;
        let text;
        if duration_ratio < 0.05 {
            text = " ".to_string();
        } else if duration_ratio <= 0.2 {
            if !use_unicode_blocks {
                text = ".".to_string();
            } else {
                text = "\u{2591}".to_string();
            }
        } else if duration_ratio <= 0.5 {
            if !use_unicode_blocks {
                text = "-".to_string();
            } else {
                text = "\u{2592}".to_string();
            }
        } else if duration_ratio <= 0.8 {
            if !use_unicode_blocks {
                text = "x".to_string();
            } else {
                text = "\u{2593}".to_string();
            }
        } else {
            if !use_unicode_blocks {
                text = "X".to_string();
            } else {
                text = "\u{2588}".to_string();
            }
        }

        let text = match color {
            Some(c) => text.color(c).to_string(),
            None => text.into(),
        };

        duration_text.push_str(&text)
    }

    duration_text.push(']');

    duration_text
}

fn generate_entry_day_activity_lines(
    entries: &[Entry],
    lines: &mut Vec<String>,
    line_prefix: &str,
    datetime_format: DateTimeFormat,
    duration_format: DurationFormat,
    bar_graph_character_num_width: u8,
    color: Option<colored::Color>,
    weekday: chrono::Weekday,
    weekday_datetime_pair: DateTimeLocalPair,
    time_block_unit: TimeBlockUnit,
) {
    let add_fringe_datetimes = false;
    let fill_datetimes_gaps = true;

    let duration_map = sum_entry_activity_duration(
        entries,
        weekday_datetime_pair,
        add_fringe_datetimes,
        fill_datetimes_gaps,
        time_block_unit,
        EntryStatus::Active,
    );
    let sorted_keys = get_map_keys_sorted_general(&duration_map.keys());
    if sorted_keys.is_empty() {
        debug!("No sorted keys found for duration map: {:#?}", duration_map);
        return;
    }

    let mut duration_bins: Vec<u64> = Vec::with_capacity(bar_graph_character_num_width as usize);
    duration_bins.resize(bar_graph_character_num_width as usize, 0);

    let mut max_duration_bin_value = 0;
    let sorted_keys_length = sorted_keys.len() as f32;
    for (i, key) in sorted_keys.iter().enumerate() {
        let key_ratio_min = (i as f32) / sorted_keys_length;
        let key_ratio_max = ((i + 1) as f32) / sorted_keys_length;
        let bin_index_min =
            (key_ratio_min * ((bar_graph_character_num_width) as f32)).round() as usize;
        let bin_index_max =
            (key_ratio_max * ((bar_graph_character_num_width) as f32)).round() as usize;

        if let Some(value) = duration_map.get(key) {
            let increment_seconds = time_block_unit.as_seconds();
            let mut num_seconds: u64 = value.num_seconds().try_into().unwrap();
            if num_seconds > increment_seconds {
                // This should not be possible - how can it be
                // possible that we've recorded more active time
                // in the time slot than physically possible?
                num_seconds = increment_seconds;
            }

            for duration_bin in duration_bins
                .iter_mut()
                .take(bin_index_max)
                .skip(bin_index_min)
            {
                *duration_bin += num_seconds;
                let current_value = *duration_bin;
                if current_value > max_duration_bin_value {
                    max_duration_bin_value = current_value;
                }
            }
        }
    }

    let inverse_max_value = 1.0 / (max_duration_bin_value as f64);
    let duration_bins_normalized: Vec<_> = duration_bins
        .iter_mut()
        .map(|x| ((*x as f64) * inverse_max_value) as f32)
        .collect();

    let key_first = &sorted_keys[0];
    let key_last = &sorted_keys[sorted_keys.len() - 1];
    let key_first_string = format_naive_time_no_seconds(*key_first, datetime_format);
    let key_last_string = format_naive_time_no_seconds(*key_last, datetime_format);

    let use_unicode_blocks = false;
    let mut duration_text =
        generate_duration_bins_text(&duration_bins_normalized, use_unicode_blocks, color);
    duration_text.push(' ');
    duration_text.push_str(&key_last_string);

    let mut lines_start = Vec::new();
    let mut lines_end = Vec::new();

    let (start_datetime_pair, _end_datetime_pair) = weekday_datetime_pair;
    let date_string = format_date(start_datetime_pair, datetime_format);
    let line_start = format!(
        "{}- {} {} {}",
        line_prefix, weekday, date_string, key_first_string
    );

    let total_duration = sum_entry_duration(&entries, EntryStatus::Active);
    let total_duration_text = format_duration(total_duration, duration_format);
    let line_end = format!(
        "{} {}{}{}",
        duration_text, HEADING_TOTAL_TEXT_START, total_duration_text, HEADING_TOTAL_TEXT_END
    );

    lines_start.push(line_start);
    lines_end.push(line_end);

    let middle_string = " ".to_string();
    combine_start_end_lines(lines, &lines_start, &lines_end, &middle_string);
}

fn generate_activity_week(
    entries: &Entries,
    lines: &mut Vec<String>,
    line_prefix: &str,
    line_heading: &str,
    week_datetime_pair: DateTimeLocalPair,
    datetime_format: DateTimeFormat,
    duration_format: DurationFormat,
    time_block_unit: TimeBlockUnit,
    bar_graph_character_num_width: u8,
    color: Option<colored::Color>,
) -> Result<()> {
    let (week_start_datetime, week_end_datetime) = week_datetime_pair;

    let mut weekday_lines = Vec::<String>::new();
    let mut week_total_duration = chrono::Duration::zero();

    let weekday_datetime_pairs =
        get_weekdays_datetime_local(week_start_datetime, week_end_datetime);

    for (weekday, weekday_datetime_pair) in weekday_datetime_pairs {
        let (weekday_start_datetime, weekday_end_datetime) = weekday_datetime_pair;
        let weekday_entries =
            entries.datetime_range_entries(weekday_start_datetime, weekday_end_datetime);

        if weekday_entries.is_empty() {
            continue;
        }

        let weekday_total_duration = sum_entry_duration(&weekday_entries, EntryStatus::Active);
        week_total_duration = week_total_duration + weekday_total_duration;

        // Group entries by name and print details.
        generate_entry_day_activity_lines(
            &weekday_entries,
            &mut weekday_lines,
            line_prefix,
            datetime_format,
            duration_format,
            bar_graph_character_num_width,
            color,
            weekday,
            weekday_datetime_pair,
            time_block_unit,
        );
    }

    let week_total_duration_text = format_duration(week_total_duration, duration_format);
    lines.push(format!(
        "{} {}{}{}:",
        line_heading, HEADING_TOTAL_TEXT_START, week_total_duration_text, HEADING_TOTAL_TEXT_END
    ));

    lines.append(&mut weekday_lines);

    Ok(())
}

/// Get the week-number to print, taking the relative number given by
/// the user into account.
//
// TODO: Write function to get relative fortnight and month.
pub fn get_relative_week_start_end(relative_week_index: i32) -> Result<DateTimeLocalPair> {
    let today_local_timezone = chrono::Local::now();
    let today_iso_week = today_local_timezone.iso_week();
    let today_week_num: u32 = (today_iso_week.week() as i64 + relative_week_index as i64)
        .clamp(u32::MIN.into(), u32::MAX.into())
        .try_into()?;
    let today_year = today_local_timezone.year();

    Ok(get_week_datetime_local(today_year, today_week_num))
}

/// Prints the time entries with the various settings given.
pub fn generate_preset_lines(
    entries: &Entries,
    output_lines: &mut Vec<String>,
    start_end_datetime_pair: DateTimeLocalPair,
    print_type: PrintType,
    variables: &[Variable],
    time_scale: TimeScale,
    datetime_format: DateTimeFormat,
    duration_format: DurationFormat,
    time_block_unit: TimeBlockUnit,
    bar_graph_character_num_width: u8,
    color: Option<colored::Color>,
) -> Result<()> {
    let line_indent = " ";

    match print_type {
        PrintType::Summary => match time_scale {
            TimeScale::Week => {
                output_lines.push("Week Summary:".to_string());
                generate_summary_week(
                    entries,
                    output_lines,
                    line_indent,
                    start_end_datetime_pair,
                    datetime_format,
                    duration_format,
                )?;
                output_lines.push("".to_string());
            }
            TimeScale::Weekday => {
                let heading_text = "Weekdays Summary";
                generate_summary_weekday(
                    entries,
                    output_lines,
                    line_indent,
                    heading_text,
                    start_end_datetime_pair,
                    datetime_format,
                    duration_format,
                )?;
                output_lines.push("".to_string());
            }
        },

        PrintType::Activity => {
            match time_scale {
                TimeScale::Week => {
                    // Duration of user for the week.
                    let heading_text = "Week Activity";
                    generate_activity_week(
                        entries,
                        output_lines,
                        line_indent,
                        &heading_text,
                        start_end_datetime_pair,
                        datetime_format,
                        duration_format,
                        TimeBlockUnit::FiveMinutes,
                        bar_graph_character_num_width,
                        color,
                    )?;
                    output_lines.push("".to_string());
                }

                TimeScale::Weekday => {
                    output_lines.push("Weekday Activity:".to_string());
                    generate_activity_weekday(
                        entries,
                        output_lines,
                        line_indent,
                        start_end_datetime_pair,
                        datetime_format,
                        duration_format,
                        time_block_unit,
                        bar_graph_character_num_width,
                        color,
                    )?;
                    output_lines.push("".to_string());
                }
            }
        }

        PrintType::Variables => match time_scale {
            TimeScale::Week => {
                let names = combine_variable_names(variables);
                let heading_text = format!("Week Variables ({})", names).to_string();

                generate_variables_week(
                    entries,
                    output_lines,
                    line_indent,
                    &heading_text,
                    start_end_datetime_pair,
                    datetime_format,
                    duration_format,
                    variables,
                )?;
                output_lines.push("".to_string());
            }
            TimeScale::Weekday => {
                let names = combine_variable_names(variables);
                output_lines.push(format!("Weekday Variables ({}):", names));

                generate_variables_weekday(
                    entries,
                    output_lines,
                    line_indent,
                    start_end_datetime_pair,
                    datetime_format,
                    duration_format,
                    variables,
                )?;
                output_lines.push("".to_string());
            }
        },

        PrintType::Software => match time_scale {
            TimeScale::Week => {
                let names = combine_variable_names(variables);
                let heading_text = format!("Week Software ({})", names).to_string();

                generate_software_week(
                    entries,
                    output_lines,
                    line_indent,
                    &heading_text,
                    start_end_datetime_pair,
                    datetime_format,
                    duration_format,
                )?;
                output_lines.push("".to_string());
            }
            TimeScale::Weekday => {
                let names = combine_variable_names(variables);
                output_lines.push(format!("Weekday Software ({}):", names));

                generate_software_weekday(
                    entries,
                    output_lines,
                    line_indent,
                    start_end_datetime_pair,
                    datetime_format,
                    duration_format,
                )?;
                output_lines.push("".to_string());
            }
        },
    }

    Ok(())
}
