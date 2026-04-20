use std::path::PathBuf;
use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_ftx1-mm"))
}

fn radio_port() -> String {
    std::env::var("RADIO_PORT").expect("RADIO_PORT env var must be set to run radio tests")
}

fn require_destructive() {
    if std::env::var("RADIO_DESTRUCTIVE").is_err() {
        panic!("set RADIO_DESTRUCTIVE=1 to run write tests (they overwrite radio memory)");
    }
}

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn temp_csv(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("ftx1_test_{name}.csv"))
}

/// Parse a CSV file into sorted, whitespace-normalised data rows (header excluded).
fn normalise_csv(path: &PathBuf) -> Vec<String> {
    let content = std::fs::read_to_string(path).unwrap();
    let mut rows: Vec<String> = content
        .lines()
        .skip(1)
        .map(|line| {
            line.split(',')
                .map(|field| field.trim().to_string())
                .collect::<Vec<_>>()
                .join(",")
        })
        .filter(|line| !line.is_empty())
        .collect();
    rows.sort();
    rows
}

// ---------------------------------------------------------------------------
// Group 1: --check-data (no radio required)
// ---------------------------------------------------------------------------

#[test]
fn check_data_valid_file() {
    let out = bin()
        .args(["--check-data", "--file", fixture("valid.csv").to_str().unwrap()])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout).contains("Data looks good!"));
}

#[test]
fn check_data_invalid_channel() {
    let out = bin()
        .args(["--check-data", "--file", fixture("invalid_channel.csv").to_str().unwrap()])
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout).contains("not a valid memory channel"));
}

#[test]
fn check_data_invalid_frequency() {
    let out = bin()
        .args(["--check-data", "--file", fixture("invalid_frequency.csv").to_str().unwrap()])
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout).contains("not valid"));
}

#[test]
fn check_data_invalid_mode() {
    let out = bin()
        .args(["--check-data", "--file", fixture("invalid_mode.csv").to_str().unwrap()])
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout).contains("not a valid mode"));
}

#[test]
fn check_data_multiple_errors() {
    let out = bin()
        .args(["--check-data", "--file", fixture("multiple_errors.csv").to_str().unwrap()])
        .output()
        .unwrap();
    assert!(!out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("not a valid memory channel"));
    assert!(stdout.contains("not valid"));
    assert!(stdout.contains("not a valid mode"));
}

#[test]
fn check_data_empty_file() {
    let out = bin()
        .args(["--check-data", "--file", fixture("empty.csv").to_str().unwrap()])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout).contains("Total records processed: 0"));
}

#[test]
fn check_data_missing_file() {
    let out = bin()
        .args(["--check-data", "--file", "nonexistent_file.csv"])
        .output()
        .unwrap();
    assert!(!out.status.success());
}

// ---------------------------------------------------------------------------
// Group 2: CLI argument handling (no radio required)
// ---------------------------------------------------------------------------

#[test]
fn no_args_prints_help_hint() {
    let out = bin().output().unwrap();
    assert!(String::from_utf8_lossy(&out.stdout).contains("No action specified"));
}

#[test]
fn help_flag() {
    let out = bin().arg("--help").output().unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("--read-radio"));
    assert!(stdout.contains("--write-radio"));
    assert!(stdout.contains("--check-data"));
}

#[test]
fn mutually_exclusive_actions() {
    let out = bin().args(["--read-radio", "--write-radio"]).output().unwrap();
    assert!(!out.status.success());
}

// ---------------------------------------------------------------------------
// Group 3: --read-radio (real radio required)
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires physical radio on RADIO_PORT"]
fn read_radio_produces_csv() {
    let out_file = temp_csv("read_produces");
    let out = bin()
        .args(["--read-radio", "--port", &radio_port(), "--file", out_file.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "binary failed:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let content = std::fs::read_to_string(&out_file).unwrap();
    assert!(content.contains("Channel Number"), "CSV header missing");
    assert!(content.lines().count() > 1, "CSV has no data rows");
    let _ = std::fs::remove_file(&out_file);
}

#[test]
#[ignore = "requires physical radio on RADIO_PORT"]
fn read_radio_csv_passes_check_data() {
    let out_file = temp_csv("read_check");
    let read_status = bin()
        .args(["--read-radio", "--port", &radio_port(), "--file", out_file.to_str().unwrap()])
        .status()
        .unwrap();
    assert!(read_status.success());

    let check = bin()
        .args(["--check-data", "--file", out_file.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(check.status.success(), "stdout: {}", String::from_utf8_lossy(&check.stdout));
    let _ = std::fs::remove_file(&out_file);
}

#[test]
#[ignore = "requires physical radio on RADIO_PORT"]
fn read_radio_wrong_port() {
    let out_file = temp_csv("read_bad_port");
    let out = bin()
        .args(["--read-radio", "--port", "/dev/nonexistent", "--file", out_file.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(!out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Failed to open port"));
    let _ = std::fs::remove_file(&out_file);
}

// ---------------------------------------------------------------------------
// Group 4: --write-radio (real radio required)
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires physical radio on RADIO_PORT"]
fn write_radio_roundtrip() {
    require_destructive();
    let before = temp_csv("roundtrip_before");
    let after = temp_csv("roundtrip_after");
    let port = radio_port();

    let s = bin()
        .args(["--read-radio", "--port", &port, "--file", before.to_str().unwrap()])
        .status()
        .unwrap();
    assert!(s.success(), "initial read failed");

    let s = bin()
        .args(["--write-radio", "--port", &port, "--file", before.to_str().unwrap()])
        .status()
        .unwrap();
    assert!(s.success(), "write failed");

    let s = bin()
        .args(["--read-radio", "--port", &port, "--file", after.to_str().unwrap()])
        .status()
        .unwrap();
    assert!(s.success(), "second read failed");

    assert_eq!(normalise_csv(&before), normalise_csv(&after), "CSV mismatch after roundtrip");

    let _ = std::fs::remove_file(&before);
    let _ = std::fs::remove_file(&after);
}

#[test]
#[ignore = "requires physical radio on RADIO_PORT"]
fn write_radio_wrong_port() {
    require_destructive();
    let out = bin()
        .args(["--write-radio", "--port", "/dev/nonexistent", "--file", fixture("valid.csv").to_str().unwrap()])
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout).contains("Failed to open port"));
}

#[test]
#[ignore = "requires physical radio on RADIO_PORT"]
fn write_radio_invalid_csv() {
    require_destructive();
    let out = bin()
        .args(["--write-radio", "--port", &radio_port(), "--file", fixture("invalid_channel.csv").to_str().unwrap()])
        .output()
        .unwrap();
    assert!(!out.status.success());
}
