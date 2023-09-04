# Developer Documentation

Timetracker is written with the [Rust Programming
Language](www.rust-lang.org), and Rust must be installed to build the
project. Rust does not need to be installed to run Timetracker.

# Build Dependencies

You will need:

* C++ Compilier (GCC - GNU Compiler Collection)
* GNOME Toolkit v3 (GTK3)
** GLIB2
** ATK
** Cairo
* X11 libraries.
* X11 XScreenSaver library..

The easiest way to set up your build environment is to use the
provided Docker files, like this:
```bash
# Go to the unzipped projecy directory.
$ cd /path/to/project/directory/timetracker

# Build Docker image (assumes the Docker Engine is already started)
$ sudo docker build --file Dockerfile_centos7 -t timetracker-linux-centos7-build .

# Go into the Docker container.
$ sudo docker run --rm --interactive --volume "${PWD}:/timetracker" --tty timetracker-linux-centos7-build
```

The following sections assume you have all dependencies installed in
the current shell.

# Building

To build Timetracker (in debug mode), use the Rust package manager
Cargo:
```bash
$ cd /path/to/timetracker
$ cargo build
```

To run and test with the maximum performance of Timetracker, make sure
to run in release mode with the '--release' flag:
```
$ cd /path/to/timetracker
$ cargo build --release
```

Timetracker (currently) only works on Linux and requires the X11 and
XScreenSaver (Xss) libraries to be installed on the running computer.

## Testing

Run the Timetracker test suite with 'cargo test':
```bash
$ cd /path/to/timetracker
$ cargo test
```

It is highly recommended to add tests before making changes, if the
existing tests do not already cover the the behaviour of the system
you are changing. Once tests are added, make the change to ensure that
the new feature does not break existing behaviour.

## Debugging

To get the best debugging experience use the built binaries in 'debug'
mode - which is the default when running 'cargo build'

You can print more information when running the Timetracker binaries
by setting the `TIMETRACKER_LOG`, `TIMETRACKER_LOG_STYLE` and
`RUST_BACKTRACE` environment variables.

For example on most Linux shells you can run:
```bash
$ TIMETRACKER_LOG=debug timetracker-recorder  # Print debugging messages.
# Or..
$ TIMETRACKER_LOG=trace timetracker-recorder  # All log messages are printed.
# Or..
$ RUST_BACKTRACE=1 timetracker-recorder  # Print backtraces when panicing.
# Or..
$ RUST_BACKTRACE=full timetracker-recorder  # Print full backtraces when panicing.
```

The logging system used by Timetracker is
[env_logger](https://docs.rs/env_logger/latest/env_logger/).
