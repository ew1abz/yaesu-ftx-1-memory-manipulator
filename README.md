# Yaesu FTX-1 Memory Manipulator

A cross-platform command-line utility for backing up, restoring, and managing
memory channels on Yaesu FTX-1 amateur radio transceivers. Channels are stored
as CSV files, making them easy to edit in any spreadsheet application.

## Features

- Read all memory channels from the radio to a CSV file
- Edit channels in any spreadsheet software and write them back
- Print channels as a formatted table
- Validate a CSV file without connecting to the radio
- Supported radio: Yaesu FTX-1

## Installation

**Download a pre-built binary** from the [Releases](https://github.com/ew1abz/yaesu-ftx-1-memory-manipulator/releases) page.
Available for:

- Linux x86\_64 and ARM64 (`.tar.gz`)
- macOS Intel and Apple Silicon (`.tar.gz`)
- Windows x86\_64 (`.zip`)

Extract and place `ftx1-mm` (or `ftx1-mm.exe`) somewhere on your `PATH`.

**Or build from source** with Rust:

```bash
cargo install --path .
```

## Usage

```bash
# Read memory channels from radio to CSV (auto-named ftx1_YYYYMMDD_HHMMSS.csv)
ftx1-mm --read-radio --port /dev/ttyUSB0

# Read to a specific file
ftx1-mm --read-radio --port /dev/ttyUSB0 --file channels.csv

# Edit channels.csv in your spreadsheet app, then write back
ftx1-mm --write-radio --port /dev/ttyUSB0 --file channels.csv

# Validate a CSV file without touching the radio
ftx1-mm --check-data --file channels.csv

# Print channels as a table
ftx1-mm --print --file channels.csv
```

Default port: `/dev/ttyUSB0`. Default speed: 38400 baud. Run `ftx1-mm --help`
for all options.
