# Yaesu FTX-1 Memory Manipulator

A cross-platform command-line utility for backing up, restoring, and managing
memory channels on Yaesu FTX-1 amateur radio transceivers. Channels are stored
as CSV files, making them easy to edit in any spreadsheet application.

## Features

- Read all memory channels from the radio to a CSV file
- Edit channels in any spreadsheet software and write them back
- Validate a CSV file without connecting to the radio
- Supported radio: Yaesu FTX-1

## Installation

```bash
cargo install --path .
```

## Usage

```bash
# Read memory channels from radio to CSV
ftx1-mm --read-radio --port /dev/ttyUSB0 --file channels.csv

# Edit channels.csv in your spreadsheet app, then write back
ftx1-mm --write-radio --port /dev/ttyUSB0 --file channels.csv

# Validate a CSV file without touching the radio
ftx1-mm --check-data --file channels.csv
```

Default port: `/dev/ttyUSB0`. Default speed: 38400 baud. Run `ftx1-mm --help`
for all options.
