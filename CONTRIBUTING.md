# Contributing

## Building

```bash
cargo build
cargo clippy
```

## Running

```bash
# Read memory channels from radio to CSV
cargo run -- --read-radio -p /dev/ttyUSB0

# Write memory channels from CSV to radio
cargo run -- --write-radio -p /dev/ttyUSB0 --file input.csv

# Validate a CSV file without touching the radio
cargo run -- --check-data --file data.csv

# Enable debug logging for any of the above
RUST_LOG=debug cargo run -- --read-radio -p /dev/ttyUSB0
```

## Tests

### Unit tests

Inline in `src/ftx1.rs` and `src/parsers.rs`. Run with:

```bash
cargo test
```

### Integration tests

Integration tests live in `tests/integration.rs` and are split into three groups:

**No radio required** — run as part of `cargo test`:

- `--check-data` validation (valid file, each invalid field type, multiple errors, empty file, missing file)
- CLI argument handling (no action, `--help`, mutually exclusive flags)

**Radio required** — skipped by default, opt in with `--ignored`:

```bash
RADIO_PORT=/dev/ttyUSB0 cargo test -- --ignored```

Covers `--read-radio`: CSV output, header/data row presence, output passes `--check-data`, wrong port error.

**Radio required + destructive** — write tests that overwrite radio memory, require a second opt-in:

```bash
RADIO_PORT=/dev/ttyUSB0 RADIO_DESTRUCTIVE=1 cargo test -- --ignored```

Covers `--write-radio`: read→write→read roundtrip (CSVs must match), wrong port error, invalid CSV rejection.

### Run a single test

```bash
cargo test test_name
# or for integration tests:
cargo test --test integration check_data_valid_file
```

## Releasing

Releases are built and published automatically by GitHub Actions when a version
tag is pushed. Binaries are produced for Linux x86\_64, Linux ARM64, macOS x86\_64, macOS
ARM64, and Windows x86\_64.

```bash
git tag v1.2.3
git push origin v1.2.3
```

The release is created at the tag with auto-generated notes and attached
archives.

## TODO

- Print memory channels as a formatted table
- Add animated GIF to README
