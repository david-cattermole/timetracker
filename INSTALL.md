# Timetracker Installation

Timetracker can be build and installed via the
[Rust](www.rust-lang.org) package manager command 'cargo'.

Timetracker can be installed directly from Git(Hub) or from a .zip
file downloaded from the GitHub project page:
https://github.com/david-cattermole/timetracker/

## How to Build and Install From Git?

Timetracker is written with the [Rust programming
language](www.rust-lang.org), and must be installed to build the
project. Go to the Rust
[install](https://www.rust-lang.org/tools/install) page for more
information.

Once Rust (and the package manager Cargo) is installed, run the
following commands:
```bash
# Install timetracker binaries into Rust's default binaries directory (${HOME}/.cargo/bin).
$ cargo install --git https://github.com/david-cattermole/timetracker.git --tag v0.1.0
```
Done! You can now type 'timetracker-recorder start' to start
Timetracker.

You can copy the 'timetracker-*' files from '${HOME}/.cargo/bin/' to
any directory accessible via your '${PATH}' environment variable.

## How to Build and Install From Zip File?

Alternatively if you want more control you can download the .zip file
from GitHub, unzip the files, then run:
```bash
# Go to the unzipped projecy directory.
$ cd /path/to/project/directory/timetracker

# Compile the project.
$ cargo build --release --verbose
...
   Fresh timetracker-configure v0.1.0 (/path/to/project/directory/timetracker/configure-bin)
   Fresh timetracker-recorder v0.1.0 (/path/to/project/directory/timetracker/recorder-bin)
   Fresh timetracker-print v0.1.0 (/path/to/project/directory/timetracker/print-bin)
Finished release [optimized] target(s) in 0.12s

# By default the files will be stored in the
# "<project directory>/target/release/" directory.

$ cargo run --release --bin timetracker-recorder
Finished release [optimized] target(s) in 0.11s
 Running `/path/to/project/directory/timetracker/target/release/timetracker-recorder`

# Copy the binary files to your preferred directory.
$ cd /path/to/project/directory/timetracker/target/release/
$ cp timetracker-configure timetracker-print timetracker-recorder /path/to/install/directory/
```
Make sure the example "/path/to/install/directory/" directory is on
your ${PATH} environment variable.

Rust does *not* need to be installed to run Timetracker, only to build
the executable binary files.
