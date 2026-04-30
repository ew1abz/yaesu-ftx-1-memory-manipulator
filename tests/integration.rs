use std::path::PathBuf;
use std::process::{Command, Output};

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_ftx1-mm"))
}

#[track_caller]
fn assert_success(out: &Output) {
    assert!(
        out.status.success(),
        "binary failed:\nexit: {}\nstdout: {}\nstderr: {}",
        out.status,
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );
}

#[track_caller]
fn assert_failure(out: &Output) {
    assert!(
        !out.status.success(),
        "binary unexpectedly succeeded:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );
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
    assert_success(&out);
    assert!(String::from_utf8_lossy(&out.stdout).contains("Data looks good!"));
}

#[test]
fn check_data_invalid_channel() {
    let out = bin()
        .args(["--check-data", "--file", fixture("invalid_channel.csv").to_str().unwrap()])
        .output()
        .unwrap();
    assert_failure(&out);
    assert!(String::from_utf8_lossy(&out.stdout).contains("not a valid memory channel"));
}

#[test]
fn check_data_invalid_frequency() {
    let out = bin()
        .args(["--check-data", "--file", fixture("invalid_frequency.csv").to_str().unwrap()])
        .output()
        .unwrap();
    assert_failure(&out);
    assert!(String::from_utf8_lossy(&out.stdout).contains("not valid"));
}

#[test]
fn check_data_invalid_mode() {
    let out = bin()
        .args(["--check-data", "--file", fixture("invalid_mode.csv").to_str().unwrap()])
        .output()
        .unwrap();
    assert_failure(&out);
    assert!(String::from_utf8_lossy(&out.stdout).contains("not a valid mode"));
}

#[test]
fn check_data_multiple_errors() {
    let out = bin()
        .args(["--check-data", "--file", fixture("multiple_errors.csv").to_str().unwrap()])
        .output()
        .unwrap();
    assert_failure(&out);
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
    assert_success(&out);
    assert!(String::from_utf8_lossy(&out.stdout).contains("Total records processed: 0"));
}

#[test]
fn check_data_missing_file() {
    let out = bin()
        .args(["--check-data", "--file", "nonexistent_file.csv"])
        .output()
        .unwrap();
    assert_failure(&out);
}

#[test]
fn check_data_duplicate_frequency_warns_but_passes() {
    let out = bin()
        .args(["--check-data", "--file", fixture("duplicate_frequency.csv").to_str().unwrap()])
        .output()
        .unwrap();
    assert_success(&out);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("has warnings"), "expected warning banner in stdout: {stdout}");
    assert!(stdout.contains("is also used by channel '00001'"), "expected dup-frequency warning text: {stdout}");
    assert!(stdout.contains("Warnings: 1"), "expected warning count in summary: {stdout}");
}

#[test]
fn check_data_accepts_libreoffice_mangled_channel_numbers() {
    let out = bin()
        .args(["--check-data", "--file", fixture("libreoffice_mangled.csv").to_str().unwrap()])
        .output()
        .unwrap();
    assert_success(&out);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Data looks good!"), "expected lenient acceptance: {stdout}");
    assert!(stdout.contains("Valid records: 2"), "expected both rows valid: {stdout}");
}

#[test]
fn check_data_no_warnings_flag_suppresses_dup_frequency() {
    let out = bin()
        .args([
            "--check-data",
            "--no-warnings",
            "--file",
            fixture("duplicate_frequency.csv").to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_success(&out);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(!stdout.contains("has warnings"), "warnings should be suppressed: {stdout}");
    assert!(!stdout.contains("is also used by channel"), "warnings should be suppressed: {stdout}");
    assert!(stdout.contains("Data looks good!"), "expected clean verdict: {stdout}");
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
    assert_success(&out);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("--read-radio"));
    assert!(stdout.contains("--write-radio"));
    assert!(stdout.contains("--check-data"));
}

#[test]
fn mutually_exclusive_actions() {
    let out = bin().args(["--read-radio", "--write-radio"]).output().unwrap();
    assert_failure(&out);
}

// ---------------------------------------------------------------------------
// Group 2b: --print (no radio required)
// ---------------------------------------------------------------------------

#[test]
fn print_valid_file() {
    let out = bin()
        .args(["--print", "--file", fixture("valid.csv").to_str().unwrap()])
        .output()
        .unwrap();
    assert_success(&out);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("145.000 MHz"));
    assert!(stdout.contains("HOME"));
    assert!(stdout.contains("433.500 MHz"));
    assert!(stdout.contains("REPEATER"));
}

#[test]
fn print_missing_file() {
    let out = bin()
        .args(["--print", "--file", "nonexistent.csv"])
        .output()
        .unwrap();
    assert_failure(&out);
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
    assert_success(&out);
    let content = std::fs::read_to_string(&out_file).unwrap();
    assert!(content.contains("Channel Number"), "CSV header missing");
    assert!(content.lines().count() > 1, "CSV has no data rows");
    let _ = std::fs::remove_file(&out_file);
}

#[test]
#[ignore = "requires physical radio on RADIO_PORT"]
fn read_radio_default_filename() {
    // Run in a temp dir so the generated file is easy to find and clean up
    let tmp = std::env::temp_dir();
    let out = bin()
        .args(["--read-radio", "--port", &radio_port()])
        .current_dir(&tmp)
        .output()
        .unwrap();
    assert_success(&out);
    // stdout says "Memory data saved to CSV file: ftx1_YYYYMMDD_HHMMSS.csv"
    let stdout = String::from_utf8_lossy(&out.stdout);
    let fname = stdout
        .lines()
        .find(|l| l.contains("Memory data saved"))
        .and_then(|l| l.split(": ").nth(1))
        .expect("expected 'Memory data saved to CSV file:' line in output");
    let generated = tmp.join(fname.trim());
    assert!(generated.exists(), "generated file not found: {:?}", generated);
    let content = std::fs::read_to_string(&generated).unwrap();
    assert!(content.contains("Channel Number"), "CSV header missing");
    let _ = std::fs::remove_file(&generated);
}

#[test]
#[ignore = "requires physical radio on RADIO_PORT"]
fn read_radio_csv_passes_check_data() {
    let out_file = temp_csv("read_check");
    let read = bin()
        .args(["--read-radio", "--port", &radio_port(), "--file", out_file.to_str().unwrap()])
        .output()
        .unwrap();
    assert_success(&read);

    let check = bin()
        .args(["--check-data", "--file", out_file.to_str().unwrap()])
        .output()
        .unwrap();
    assert_success(&check);
    let _ = std::fs::remove_file(&out_file);
}

#[test]
#[ignore = "requires physical radio on RADIO_PORT"]
fn read_radio_wrong_port() {
    let out = bin()
        .args(["--read-radio", "--port", "/dev/nonexistent"])
        .output()
        .unwrap();
    assert_failure(&out);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Failed to open port"));
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

    let out = bin()
        .args(["--read-radio", "--port", &port, "--file", before.to_str().unwrap()])
        .output()
        .unwrap();
    assert_success(&out);

    let out = bin()
        .args(["--write-radio", "--port", &port, "--file", before.to_str().unwrap()])
        .output()
        .unwrap();
    assert_success(&out);

    let out = bin()
        .args(["--read-radio", "--port", &port, "--file", after.to_str().unwrap()])
        .output()
        .unwrap();
    assert_success(&out);

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
    assert_failure(&out);
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
    assert_failure(&out);
}
