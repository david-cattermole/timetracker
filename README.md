# Timetracker

**Timetracker automatically records application activity to gather personal time statistics.**

Timetracker runs in the background to gather how much time you use software, and the (environment variable) context that the software is used in.

After the data is recorded, it may be queried to find out how much time you've spent:

 * active on a computer, and when where you active?
 * working in a specific software?
 * working with a set of environment variables?
 * working with-in a working directory (with the `PWD` environment variable)?

Features:

 * Records the current active window and (a limited set of) environment variables into a local file.
 * Queries the local file to display time-based activity information.

Timetracker is a *personal* tool and is intended to only store data *locally* in the user's home directory. Users may delete their data at anytime.

Timetracker was developed for 3D Animation, Computer Graphics (CG) and Visual Effects (VFX) productions, where environment variables passed to software is used to denote the different projects, sequences, shots and tasks that are worked on by artists.

## Getting Started

Set up a basic configuration file.

```bash
$ timetracker-configure
```

Follow the prompts.

```bash
$ timetracker-recorder &
```

```bash
$ timetracker-print
```

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
