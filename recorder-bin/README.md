# Recorder (binary)

This directory contains the Rust crate for the main Recorder program -
known as Recorder.

The Recorder is used to gather information from the user's active
windows, including up to 5 environment variables, the executable name
running the window, and the amount of time the window is active for.

## Configuration

The Recorder uses the following configuration options in the
`.timetracker.toml` file:
```
[core]
# Directory to find the database file.
database_dir = "${HOME}/.config"

# File name of the database storage.
database_file_name = "name_of_database_file."  # ".timetracker.sqlite3"

[core.environment_variables]
# The environment variables gathered into the database by the recorder.
names = ["PWD", "USER", "SHOT", "PROJECT"]
```

## How Recorder Works

The Recorder only works on Linux and is responsible for gathering
activity data for the current user from the X11 library. The
'build.rs' file is responsible for linking to the installed system's
X11 libraries.

The Recorder works by polling user data every 1 second, saving that
data in memory, then every 10 seconds the data is flushed to the
(database) storage. Polling data and writing data is performed with
different threads, so that writing data cannot slow down the capture
of user data. Communication between threads is synchronized with
shared Mutex.

If the Recorder experiences a segmentation fault (e.g. panic), the
data currently stored in memory will be flushed to the storage
(database) before the program ends - if possible. If a crash happens,
at most 10 seconds of user data is lost.
