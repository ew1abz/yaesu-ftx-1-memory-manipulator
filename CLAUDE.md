# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`ftx1-mm` is a CLI tool for managing memory channels on Yaesu FTX-1 amateur radio transceivers via the CAT (Computer Aided Transceiver) serial protocol. It reads/writes memory channel data between the radio and CSV files.

## Commands

```bash
cargo build
cargo test
cargo clippy
RUST_LOG=debug cargo run -- --read-radio -p /dev/ttyUSB0
cargo run -- --help
```

Run a single test by name:

```bash
cargo test test_name
```

## Architecture

```text
src/
├── main.rs     - CLI (clap), serial port management, CSV I/O, 3 top-level ops
├── ftx1.rs     - CAT protocol: commands, packet codecs, all domain types
├── parsers.rs  - ASCII→integer converters for fixed-width binary fields
├── ftdx10.rs   - Reference stub for FTDX10 variant (different radio ID)
└── lib.rs      - Re-exports ftx1 module for library use
```

### Data Flow

**Read:** serial port → `CMD_MR`/`CMD_MT`/`CMD_CN` per channel → `MemoryReadWrite` structs → CSV

**Write:** CSV → `CsvRecord` → `TryFrom` → `MemoryReadWrite` → `CMD_MW`/`CMD_MT` per channel

### CAT Protocol

Commands are ASCII strings ending with `;`. All serial I/O goes through `serialport` crate at 38400 baud by default, 200 ms timeout. Each command type is a struct wrapping `Cmd<'a>` with a fixed expected response length; `is_reply_ok()` validates response code and byte count.

Key commands: `CMD_ID` (validate radio, ID=840), `CMD_MR` (read 27-byte memory), `CMD_MW` (write memory), `CMD_MT` (write 12-byte tag), `CMD_MC` (select channel), `CMD_CN` (CTCSS/DCS tone).

### Core Domain Types (`ftx1.rs`)

- `MemoryReadWrite` — complete 26-byte channel state
- `MemoryChannel` — 5-char channel ID (e.g. `"00001"`, `"P-01L"`, `"EMGCH"`)
- `FrequencyHz` — validated 30 kHz–174 MHz or 400–470 MHz
- `ClarifierOffsetHz` — –9990 to +9990 Hz
- `Mode` — 16 variants (LSB, USB, FM, AM, CW, RTTY, PSK, DATA variants)
- `SqlType` — CTCSS OFF/ENC/ENC-DEC, DCS, PR FREQ, REV TONE
- `Shift` — Simplex/Plus/Minus

CTCSS lookup table: 50 standard tones. DCS lookup table: 104 codes. Both are const arrays in `ftx1.rs`.

### CSV Format

Columns: Channel Number, Frequency (Hz), Memory Tag, Mode, Channel Type, Squelch Type, Shift (Hz), Clarifier Offset (Hz), Rx Clarifier Enabled, Tx Clarifier Enabled, CTCSS Tone, DCS Tone.

### Tests

Unit tests live inline in `src/ftx1.rs` and `src/parsers.rs`. The `tests/` directory is currently empty.
