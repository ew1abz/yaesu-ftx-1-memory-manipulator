# Yaesu FTX-1 Memory Manipulator

A cross-platform command-line utility for backing up, restoring, and managing
memory channels on Yaesu FTX-1 amateur radio transceivers. Channels are stored
as CSV files, making them easy to edit in any spreadsheet application.

![Reading memory channels from the radio](doc/demo-read.gif)

## Features

- Read all memory channels from the radio to a CSV file
- Write memory channels from a CSV file to the radio
- Print channels as a formatted table
- Validate a CSV file without connecting to the radio

Supported radio: Yaesu FTX-1

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

## Editing the CSV

**Channel numbering.** Channels don't have to be contiguous. Skip any
numbers you want to leave open for future use — the validator ignores
the gap, the radio leaves untouched slots untouched on write. Lines
starting with `#` are also skipped, so you can use them as section
dividers in the spreadsheet view:

```text
# --- 2 m repeaters ---
00050,...
00051,...

# --- 70 cm repeaters ---
00100,...
```

**Squelch Type names.** The CSV uses the internal enum names rather
than the radio's front-panel labels. Quick reference:

| Radio UI | CSV value     | Meaning                            |
| :------: | :------------ | :--------------------------------- |
|   OFF    | `CtcssOff`    | No tone squelch                    |
|   TONE   | `CtcssEnc`    | Encode only (TX tone, no RX gate)  |
|   TSQ    | `CtcssEncDec` | Encode + decode (tone squelch)     |
|   DCS    | `Dcs`         | Digital code squelch               |

**Shift values.** `Simplex`, `PlusShift`, `MinusShift` use the per-band
offset menu setting. `Ars` lets the radio pick direction and offset
from its built-in band plan — useful where the per-band default
doesn't match local convention. For a fully custom TX frequency,
set `Split TX (Hz)` to the exact transmit frequency instead.

## Spreadsheet caveats

Editing the CSV in Excel or LibreOffice is fully supported, but be aware
that on save the spreadsheet will reformat numeric columns. The two
visible changes:

- Channel Number `00001` → `1` (leading zeros stripped)
- CTCSS Tone `100.0` → `100` (trailing `.0` stripped)

`ftx1-mm` accepts both forms — channel numbers are auto-padded to 5
chars on read, and CTCSS tones are matched by numeric value rather than
exact string. Just keep in mind that the file *looks* different after a
spreadsheet round-trip; nothing is actually lost.

If you want the original formatting back, re-export from the radio with
`--read-radio` after writing your edits.

## Limitations

- **Not all memory channel fields are supported.** The per-channel fields
  currently round-tripped are listed in
  [doc/memory-channel-fields.md](doc/memory-channel-fields.md). Notably
  unsupported: Memory Group (M-GRP) and the per-band repeater-offset Hz
  (a menu setting, not per-channel — use `Split TX (Hz)` instead when you
  need a non-standard offset).
- **Radio settings are not touched.** This tool only reads and writes
  memory channels. Global/per-band/per-side settings — IPO/pre-amp, DNR,
  DNF, narrow filter, RF attenuator, noise blanker, AGC, band repeater
  offsets, menu (`EX`) parameters — are out of scope.
- **Speech EQ / Compressor are not per-channel.** The CAT spec exposes
  them as radio-global settings, not per memory slot, so a CSV can't
  store them. Set them once on the radio and they apply across channels.
- **No CAT command to delete a channel.** The radio doesn't expose
  channel clearing over CAT. Writing a CSV only programs the channels
  it contains; existing channels not in the CSV are left untouched. To
  clear a slot, use the radio's front panel.

---

⭐ If this program is useful to you, a star goes a long way — thank you!

Made with ❤️ for terminal enthusiasts and spreadsheet power users.
