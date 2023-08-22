# Timetracker

**Timetracker automatically records application activity to gather
personal time statistics.**

Timetracker runs in the background, keeping track of the time you
spend using different software and the specific environment you're
working in. It remembers things like the software you're using, the
context it is used, and more.

Once it has all this info, you can ask Timetracker:

 * How much time you've spent actively using your computer, and when
   you were active.
 * How much time you've spent in a particular software.
 * How much time you've spent in a certain environment (like a project
   setting).
 * How much time you've spent working within a specific working
   directory.

Features:

 * Records the current active window and (a limited set of) environment
   variables into a local file.
 * Queries the local file to display time-based activity information.

Timetracker is meant for your *personal* use. It only keeps your data
on your own computer in your home folder - if you want, you can always
delete your data.

Timetracker was created with 3D Animation, Computer Graphics (CG), and
Visual Effects (VFX) work in mind. It's especially useful for keeping
track of which projects, sequences, shots, and tasks artists are
worked on.

## Getting Started

First, create a new basic configuration file:
```bash
$ timetracker-configure > ~/.config/.timetracker.toml   # '~/.timetracker.toml' also works.
```

Edit and display the resolved configuration file:
```bash
# Edit your configuration text file as needed with a text editor
# such as 'nano', 'emacs', or 'vi'.
$ nano ~/.config/.timetracker.toml

# Display the fully resolved configuration file that will be used.
$ timetracker-configure --load-user-overrides
```
See the "Configuration File Example" below for more details of setting
up your configuration file.

Start the recorder:
```bash
# Start running timetracker in the background.
$ timetracker-recorder start &

# You can also, start running a new timetracker process,
# killing any existing processes at the same time.
$ timetracker-recorder start --terminate-existing-processes

# Or, you can stop all running timetracker processes.
$ timetracker-recorder stop
```

Printing recorded data:
```bash
# Display recorded information for the current week.
$ timetracker-print

# Display information for last week.
$ timetracker-print --relative-week=-1

# Display information using only specific presets.
$ timetracker-print -p activity_weekdays -p software_week

# List all available preset names (to be queried with '-p' flag).
$ timetracker-print --list-presets
```

All the Timetracker commands support the '-h' or '--help' flags to
print help.

## Configuration File Example

The configuration file can be edited to record and print information
that is tailored to your needs.

The key variables to edit are:
* `core.environment_variables.names`
* `print.format_datetime`
* `print.format_duration`
* `print.display_presets`

Any variable that is given in the configuration file overrides any
default values. You do not need to specify any or all variables.

```toml
[core.environment_variables]
# These are the environment variables that will be recorded. Use
# variables that are set in your applications that can identify
# important information. For example in VFX or Animation projects, you
# may want to record the project name, sequence and shot. You can give
# up to *five* environment variable names.
names = ["PROJECT", "SEQUENCE", "SHOT", "PWD"]

[print]
format_datetime = "Iso"  # Locale, Iso, or UsaMonthDayYear.
format_duration = "HoursMinutes"  # HoursMinutes, HoursMinutesSeconds, or DecimalHours.
# The list of presets that are displayed by default when
# running 'timetracker-print'.
display_presets = ["summary_week", "summary_weekdays", "working_directory_week", "software_week"]

# A custom preset named 'shot_weekdays' that will display the extra
# recorded environment variables, listed per-weekday.
[print.presets.shot_weekdays]
print_type = "Variables"
time_scale = "Weekday"
variable_names = ["PROJECT", "SEQUENCE", "SHOT"]

# A custom preset named 'shot_week' that will display the extra
# recorded environment variables, listed per-weekday.
[print.presets.shot_week]
print_type = "Variables"
time_scale = "Week"
variable_names = ["PROJECT", "SEQUENCE", "SHOT"]
```

## Installation

Follow the instructions in the
[INSTALL.md](https://github.com/david-cattermole/timetracker/blob/main/INSTALL.md)
file for more information.

## Contributions

The
[DEVELOPER.md](https://github.com/david-cattermole/timetracker/blob/main/DEVELOPER.md)
file contains information for developers and people wanting to make
changes to Timetracker.

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
