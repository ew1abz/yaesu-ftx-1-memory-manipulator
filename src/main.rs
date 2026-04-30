use clap::Parser;
use comfy_table::presets::{ASCII_FULL_CONDENSED, UTF8_FULL_CONDENSED};
use comfy_table::{Attribute, Cell, Color, ContentArrangement, Table};
use indicatif::ProgressBar;
use log::{debug, error, trace};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::io;
use std::iter::zip;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

mod ftx1;
use ftx1::*;

const RX_BUFFER_SIZE: usize = 255;
const CHANNELS: u16 = 100;

/// A simple program to interact with Yaesu FT-DX1 series radios
#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "Yaesu FTX-1 series radio memory manager",
    long_about = "This program allows you to read and write memory channels to your Yaesu FTX-1 series radio.

Usage:
  ftdx-1chm --read-radio --port /dev/ttyUSB0 --speed 38400 --file output.csv
  ftdx-1chm --write-radio --port /dev/ttyUSB0 --speed 38400 --file input.csv
  ftdx-1chm --check-data --file data.csv"
)]
struct Cli {
    /// Port to connect to the radio
    #[arg(short, long, default_value = "/dev/ttyUSB0")]
    port: String,

    /// Speed for the serial port
    #[arg(short, long, default_value_t = 38_400)]
    speed: u32,

    /// File to save/read memory data (default for --read-radio: ftx1_YYYYMMDD_HHMMSS.csv)
    #[arg(short, long)]
    file: Option<String>,

    /// Read from radio
    #[arg(short = 'r', long, group = "action")]
    read_radio: bool,

    /// Write to radio
    #[arg(short = 'w', long, group = "action")]
    write_radio: bool,

    /// Check data in the file
    #[arg(long, group = "action")]
    check_data: bool,

    /// Print memory channels from file as a table
    #[arg(long, group = "action")]
    print: bool,

    /// Use plain ASCII table style without colors
    #[arg(long)]
    plain: bool,

    /// Suppress all output (progress bars, status messages, table)
    #[arg(short, long)]
    quiet: bool,

    /// Disable non-blocking validation warnings (currently: duplicate-frequency detection)
    #[arg(long)]
    no_warnings: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
struct CsvRecord {
    #[serde(rename = "Channel Number")]
    channel: String,
    #[serde(rename = "Frequency (Hz)")]
    freq: u32, // FrequencyHz,
    #[serde(rename = "Memory Tag")]
    tag: Option<String>,
    #[serde(rename = "Mode")]
    mode: String,
    #[serde(rename = "Channel Type")]
    ch_type: ChType,
    #[serde(rename = "Squelch Type")]
    tone: SqlType,
    #[serde(rename = "Shift (Hz)")]
    shift: Shift,
    #[serde(rename = "Clarifier Offset (Hz)")]
    clarifier_offset_hz: i16,
    #[serde(rename = "Rx Clarifier Enabled")]
    rx_clarifier_enabled: RxClarifierOnOff,
    #[serde(rename = "Tx Clarifier Enabled")]
    tx_clarifier_enabled: TxClarifierOnOff,
    #[serde(rename = "CTCSS Tone")]
    ctcss_tone: String,
    #[serde(rename = "DCS Tone")]
    dcs_tone: String,
}

impl TryFrom<CsvRecord> for MemoryReadWrite {
    type Error = ();

    fn try_from(item: CsvRecord) -> Result<Self, Self::Error> {
        let channel = MemoryChannel::try_from(item.channel)?;
        let mem = MemoryReadWrite {
            channel,
            frequency_hz: FrequencyHz::try_from(item.freq)?,
            clarifier_offset_hz: ClarifierOffsetHz::try_from(item.clarifier_offset_hz)?,
            rx_clarifier_enabled: item.rx_clarifier_enabled,
            tx_clarifier_enabled: item.tx_clarifier_enabled,
            mode: Mode::try_from(item.mode)?,
            ch_type: item.ch_type,
            sql_type: item.tone,
            shift: item.shift,
        };
        Ok(mem)
    }
}

fn default_filename() -> String {
    let secs = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    let (y, mo, d, h, mi, s) = secs_to_datetime(secs);
    format!("ftx1_{:04}{:02}{:02}_{:02}{:02}{:02}.csv", y, mo, d, h, mi, s)
}

fn secs_to_datetime(secs: u64) -> (u64, u64, u64, u64, u64, u64) {
    let s = secs % 60;
    let mins = secs / 60;
    let mi = mins % 60;
    let hours = mins / 60;
    let h = hours % 24;
    let days = hours / 24;
    // Days since 1970-01-01
    let (y, mo, d) = days_to_ymd(days);
    (y, mo, d, h, mi, s)
}

fn days_to_ymd(mut days: u64) -> (u64, u64, u64) {
    let mut year = 1970u64;
    loop {
        let leap = year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400));
        let days_in_year = if leap { 366 } else { 365 };
        if days < days_in_year { break; }
        days -= days_in_year;
        year += 1;
    }
    let leap = year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400));
    let days_in_month = [31u64, if leap { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut month = 1u64;
    for dim in &days_in_month {
        if days < *dim { break; }
        days -= dim;
        month += 1;
    }
    (year, month, days + 1)
}

fn require_file(file: &Option<String>, flag: &str) -> Result<String, ()> {
    file.clone().ok_or_else(|| println!("Error: --file is required for {}", flag))
}

fn main() -> Result<(), ()> {
    let cli = Cli::parse();
    env_logger::init();

    if cli.read_radio {
        read_radio_data(&cli)?;
    } else if cli.write_radio {
        let file = require_file(&cli.file, "--write-radio")?;
        write_radio_data(&cli, &file)?;
    } else if cli.check_data {
        let file = require_file(&cli.file, "--check-data")?;
        check_data(&file, cli.quiet, true, !cli.no_warnings)?;
    } else if cli.print {
        let file = require_file(&cli.file, "--print")?;
        print_table(&file, cli.plain, cli.quiet)?;
    } else {
        println!("No action specified. Use --help for options.");
    }

    Ok(())
}

fn check_data(file_path: &str, quiet: bool, verbose: bool, warnings_enabled: bool) -> Result<(), ()> {
    let mut rdr = csv::ReaderBuilder::new()
        .comment(Some(b'#'))
        .from_path(file_path)
        .map_err(|e| {
            if !quiet { println!("Error opening file '{}': {}", file_path, e); }
        })?;
    let mut valid_records = 0;
    let mut invalid_records = 0;
    let mut warnings_count: u32 = 0;
    let mut seen_channels: HashSet<String> = HashSet::new();
    let mut seen_frequencies: HashMap<u32, (String, Option<String>)> = HashMap::new();
    let mut duplicates_found = false;

    for (i, result) in rdr.deserialize().enumerate() {
        let mut record: CsvRecord = match result {
            Ok(r) => r,
            Err(e) => {
                if !quiet { println!("Error deserializing record {}: {}", i + 1, e); }
                invalid_records += 1;
                continue;
            }
        };
        normalize_record(&mut record);

        let mut errors = match validate_record(&record) {
            Ok(_) => Vec::new(),
            Err(e) => e,
        };
        if !seen_channels.insert(record.channel.clone()) {
            errors.push(format!("Channel '{}' appears more than once.", record.channel));
            duplicates_found = true;
        }

        let mut warnings: Vec<String> = Vec::new();
        if warnings_enabled {
            match seen_frequencies.get(&record.freq) {
                Some((prev_ch, prev_tag)) => {
                    let prev_label = match prev_tag {
                        Some(t) if !t.trim().is_empty() => format!("'{}' ({})", prev_ch, t.trim()),
                        _ => format!("'{}'", prev_ch),
                    };
                    let cur_label = match &record.tag {
                        Some(t) if !t.trim().is_empty() => format!(" ({})", t.trim()),
                        _ => String::new(),
                    };
                    warnings.push(format!(
                        "Frequency {} Hz{} is also used by channel {}.",
                        record.freq, cur_label, prev_label
                    ));
                }
                None => {
                    seen_frequencies.insert(record.freq, (record.channel.clone(), record.tag.clone()));
                }
            }
        }

        if errors.is_empty() {
            valid_records += 1;
        } else {
            invalid_records += 1;
        }
        warnings_count += warnings.len() as u32;

        if !quiet && (!errors.is_empty() || !warnings.is_empty()) {
            let label = if errors.is_empty() { "has warnings" } else { "is invalid" };
            println!("Record {} {}:", i + 1, label);
            for error in errors {
                println!("  - {}", error);
            }
            for warning in &warnings {
                println!("  ! {}", warning);
            }
        }
    }

    if verbose && !quiet {
        println!("\n----- Validation Summary -----");
        println!("Total records processed: {}", valid_records + invalid_records);
        println!("Valid records: {}", valid_records);
        println!("Invalid records: {}", invalid_records);
        if warnings_count > 0 {
            println!("Warnings: {}", warnings_count);
        }
    }

    if invalid_records == 0 {
        if verbose && !quiet {
            if warnings_count > 0 {
                println!("\nData is valid ({} warning(s) — see above).", warnings_count);
            } else {
                println!("\nData looks good!");
            }
        }
        Ok(())
    } else {
        if verbose && !quiet { println!("\nData has issues and may not be processable."); }
        if duplicates_found && !quiet {
            println!(
                "\nTip: to renumber duplicate channels sequentially, run\n     python3 renumber_channels.py {} > fixed.csv",
                file_path
            );
        }
        Err(())
    }
}

// Normalise fields a spreadsheet (LibreOffice, Excel) is likely to have
// mangled on a save round-trip. Only the leading-zero numeric memory channel
// format (00001–00999) is affected: spreadsheets open it as Number and strip
// the zeros. PMS (`P-01L`), 5MHz band (`50001`), and EMGCH already survive
// because they contain non-digits or no leading zero.
fn normalize_record(record: &mut CsvRecord) {
    let ch = &record.channel;
    if (1..5).contains(&ch.len()) && ch.chars().all(|c| c.is_ascii_digit()) {
        record.channel = format!("{:0>5}", ch);
    }
}

fn validate_record(record: &CsvRecord) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    // Validate channel
    if record.channel.len() != 5 {
        errors.push(format!("Channel '{}' has invalid length. Expected 5.", record.channel));
    } else {
        let chars: Vec<char> = record.channel.chars().collect();
        let ch_array: [char; 5] = [chars[0], chars[1], chars[2], chars[3], chars[4]];
        if MemoryChannel::try_from(&ch_array).is_err() {
            errors.push(format!("Channel '{}' is not a valid memory channel.", record.channel));
        }
    }

    // Validate frequency
    if FrequencyHz::try_from(record.freq).is_err() {
        errors.push(format!("Frequency '{}' is not valid.", record.freq));
    }

    // Validate clarifier offset: 0000 - 9990 (Hz)
    if ClarifierOffsetHz::try_from(record.clarifier_offset_hz).is_err() {
        errors.push(format!(
            "Clarifier offset '{}' is not a valid number.",
            record.clarifier_offset_hz
        ));
    }

    // Validate mode
    const VALID_MODES: &[&str] = &[
        "LSB",
        "USB",
        "CW-U",
        "FM",
        "AM",
        "RTTY-L",
        "CW-L",
        "DATA-L",
        "RTTY-U",
        "DATA-FM",
        "FM-N",
        "DATA-U",
        "AM-N",
        "PSK",
        "DATA-FM-N",
    ];
    if !VALID_MODES.contains(&record.mode.as_str()) {
        errors.push(format!("Mode '{}' is not a valid mode.", record.mode));
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn print_table(file_path: &str, plain: bool, quiet: bool) -> Result<(), ()> {
    if quiet { return Ok(()); }
    let mut rdr = csv::ReaderBuilder::new()
        .comment(Some(b'#'))
        .from_path(file_path)
        .map_err(|_| ())?;
    let mut table = Table::new();

    if plain {
        table.load_preset(ASCII_FULL_CONDENSED);
    } else {
        table
            .load_preset(UTF8_FULL_CONDENSED)
            .set_content_arrangement(ContentArrangement::Dynamic);
    }

    let headers = ["Ch", "Frequency", "Tag", "Mode", "Type", "Squelch", "Shift", "Clar (Hz)", "RX Clar", "TX Clar", "CTCSS", "DCS"];
    table.set_header(headers.iter().map(|h| {
        if plain {
            Cell::new(h)
        } else {
            Cell::new(h)
                .add_attribute(Attribute::Bold)
                .fg(Color::Cyan)
        }
    }));

    for result in rdr.deserialize::<CsvRecord>() {
        let r = result.map_err(|_| ())?;
        let freq = format!("{:.3} MHz", r.freq as f64 / 1_000_000.0);
        let tag = r.tag.as_deref().unwrap_or("").to_string();
        let squelch = r.tone.to_string();

        let squelch_color = match r.tone {
            SqlType::CtcssOff => Color::DarkGrey,
            SqlType::CtcssEnc => Color::Yellow,
            SqlType::CtcssEncDec => Color::Green,
            SqlType::Dcs => Color::Magenta,
            _ => Color::White,
        };

        let rx_clar_on = r.rx_clarifier_enabled == RxClarifierOnOff::RxClarifierOn;
        let tx_clar_on = r.tx_clarifier_enabled == TxClarifierOnOff::TxClarifierOn;

        let make = |s: String, color: Color| -> Cell {
            if plain { Cell::new(s) } else { Cell::new(s).fg(color) }
        };

        table.add_row(vec![
            make(r.channel,                                                          Color::White),
            make(freq,                                                               Color::Green),
            make(tag,                                                                Color::White),
            make(r.mode,                                                             Color::Yellow),
            make(r.ch_type.to_string(),                                              Color::DarkGrey),
            make(squelch,                                                            squelch_color),
            make(r.shift.to_string(),                                                Color::DarkGrey),
            make(r.clarifier_offset_hz.to_string(),                                  Color::DarkGrey),
            make(r.rx_clarifier_enabled.to_string(), if rx_clar_on { Color::Green } else { Color::DarkGrey }),
            make(r.tx_clarifier_enabled.to_string(), if tx_clar_on { Color::Green } else { Color::DarkGrey }),
            make(r.ctcss_tone,                                                       Color::DarkGrey),
            make(r.dcs_tone,                                                         Color::DarkGrey),
        ]);
    }
    println!("{table}");
    Ok(())
}

fn read_radio_data(cli: &Cli) -> Result<(), ()> {
    let quiet = cli.quiet;
    let file = cli.file.clone().unwrap_or_else(default_filename);
    let mut port = open_radio(&cli.port, cli.speed, quiet)?;
    let mut wtr = csv::Writer::from_path(&file).map_err(|_| ())?;

    if !quiet { println!("Reading memory channels..."); }
    let bar = if quiet { ProgressBar::hidden() } else { ProgressBar::new(CHANNELS as u64) };
    let mut memory_list: Vec<MemoryReadWrite> = Vec::new();
    for ch in 1..=CHANNELS {
        bar.inc(1);
        let mem = read_mem(&mut *port, ch);
        if let Ok(m) = mem {
            memory_list.push(m);
        }
    }
    bar.finish();

    if !quiet { println!("Reading memory tags..."); }
    let bar = if quiet { ProgressBar::hidden() } else { ProgressBar::new(memory_list.len() as u64) };
    let mut tag_list: Vec<Option<String>> = Vec::new();
    for ch in 1..=memory_list.len() as u16 {
        bar.inc(1);
        let tag = read_tag(&mut *port, ch);
        tag_list.push(tag);
    }
    bar.finish();

    if !quiet { println!("Reading tone info..."); }
    let bar = if quiet { ProgressBar::hidden() } else { ProgressBar::new(memory_list.len() as u64) };
    let mut tone_list: Vec<(ToneCode, ToneCode)> = Vec::new();
    for ch in 1..=memory_list.len() as u16 {
        bar.inc(1);
        // There is no answer for this command, so we ignore the result
        let _ = cat_send(&mut *port, &CMD_MC.set(Side::Sub, MemoryChannel::Mem(ch)))?;
        let ctcss_tone_reply = cat_send(&mut *port, &CMD_CN.read(Side::Sub, ToneType::Ctcss))?;
        let ctcss_tone_decoded = CMD_CN.decode(&ctcss_tone_reply)?;
        let dcs_tone_reply = cat_send(&mut *port, &CMD_CN.read(Side::Sub, ToneType::Dcs))?;
        let dcs_tone_decoded = CMD_CN.decode(&dcs_tone_reply)?;
        tone_list.push((ctcss_tone_decoded.tone_code, dcs_tone_decoded.tone_code));
    }

    // Combine memory data, tags and tones into CSV records
    for (m, (tag, tone)) in zip(memory_list, zip(tag_list, tone_list)) {
        let rec = CsvRecord {
            channel: m.channel.to_string()?,
            tag,
            freq: m.frequency_hz.to_u32(),
            clarifier_offset_hz: m.clarifier_offset_hz.to_i16(),
            rx_clarifier_enabled: m.rx_clarifier_enabled,
            tx_clarifier_enabled: m.tx_clarifier_enabled,
            mode: m.mode.to_string(),
            ch_type: m.ch_type,
            tone: m.sql_type,
            shift: m.shift,
            ctcss_tone: CmdCn::tone_code_to_string(ToneType::Ctcss, tone.0)?,
            dcs_tone: CmdCn::tone_code_to_string(ToneType::Dcs, tone.1)?,
        };
        // println!("{:?}", rec);
        wtr.serialize(&rec).unwrap();
    }
    wtr.flush().unwrap();
    if !quiet { println!("Memory data saved to CSV file: {}", file); }
    print_table(&file, cli.plain, quiet)
}

fn read_validate_id(port: &mut dyn serialport::SerialPort, quiet: bool) -> Result<(), ()> {
    let rx = cat_send(port, &CMD_ID.read())?;
    let id = CMD_ID.decode(&rx)?;
    match CMD_ID.validate(id) {
        Ok(_) => { if !quiet { println!("Yaesu FTX-1 found (radio ID: {:04})", &id); } }
        Err(e) => { if !quiet { println!("Can't connect to Yaesu FTX-1: {:?}", e); } }
    }
    Ok(())
}

fn read_mem(port: &mut dyn serialport::SerialPort, ch: u16) -> Result<MemoryReadWrite, ()> {
    let rx = cat_send(port, &CMD_MR.read(MemoryChannel::Mem(ch)))?;
    CMD_MR.decode(&rx)
}

fn read_tag(port: &mut dyn serialport::SerialPort, ch: u16) -> Option<String> {
    debug!("Reading tag for channel: {:?}", ch);
    let rx = cat_send(port, &CMD_MT.read(MemoryChannel::Mem(ch))).ok()?;
    let d = CMD_MT.decode(&rx);
    match &d {
        Ok(tag) => debug!("Tag: {:}", &tag),
        Err(e) => error!("Error: {:?}", e),
    }
    d.ok()
}

fn cat_send(port: &mut dyn serialport::SerialPort, data: &[u8]) -> Result<Vec<u8>, ()> {
    port.write_all(data).map_err(|_| ())?;
    trace!("Sent: {:?} {:?}", String::from_utf8_lossy(data), data);

    // CAT replies end with ';'. On Linux the kernel usually delivers the whole
    // reply in one read; on Windows the driver hands it back byte by byte, so
    // we must accumulate until we see the terminator or actually time out.
    let mut buffer: Vec<u8> = Vec::with_capacity(RX_BUFFER_SIZE);
    let mut chunk: Vec<u8> = vec![0; RX_BUFFER_SIZE];
    loop {
        match port.read(chunk.as_mut_slice()) {
            Ok(n) => {
                buffer.extend_from_slice(&chunk[..n]);
                if buffer.last() == Some(&b';') || buffer.len() >= RX_BUFFER_SIZE {
                    break;
                }
            }
            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => break,
            Err(e) => eprintln!("{:?}", e),
        }
    }
    trace!("Received: {:?} {:?}", String::from_utf8_lossy(&buffer), buffer);
    Ok(buffer)
}

fn open_radio(port_name: &String, port_peed: u32, quiet: bool) -> Result<Box<dyn serialport::SerialPort>, ()> {
    match serialport::new(port_name, port_peed).timeout(Duration::from_millis(200)).open() {
        Ok(mut port) => {
            if let Err(e) = read_validate_id(&mut *port, quiet) {
                if !quiet { println!("Error validating radio ID: {:?}", e); }
                return Err(());
            }
            Ok(port)
        }
        Err(e) => {
            if !quiet { println!("Failed to open port '{}': {:?}", port_name, e); }
            Err(())
        }
    }
}

fn write_radio_data(cli: &Cli, file: &str) -> Result<(), ()> {
    let quiet = cli.quiet;
    check_data(file, quiet, false, !cli.no_warnings)?;
    let mut port = open_radio(&cli.port, cli.speed, quiet)?;

    let mut rdr = csv::ReaderBuilder::new()
        .comment(Some(b'#'))
        .from_path(file)
        .map_err(|_| ())?;
    let mut records: Vec<CsvRecord> = rdr.deserialize::<CsvRecord>().filter_map(|r| r.ok()).collect();
    for r in &mut records {
        normalize_record(r);
    }
    if !quiet { println!("Writing memory data from CSV file: {} ({} records)... ", file, records.len()); }
    let bar = if quiet { ProgressBar::hidden() } else { ProgressBar::new(records.len() as u64) };
    for rec in records {
        bar.inc(1);
        let mem = MemoryReadWrite::try_from(rec.clone())?;
        debug!("Writing memory data for channel: {:?}", mem);
        // MW first to ensure the channel slot exists. AM-only fails to create
        // new (empty) channels because MC can't reliably select an empty slot.
        // MW resets tones, but the AM step below re-commits them from VFO state.
        let _ = cat_send(&mut *port, &CMD_MW.set(mem.clone())?)?;
        // Put main in Memory mode and select the channel so AM later writes
        // back to the correct memory slot; switch to VFO to build up state.
        let _ = cat_send(&mut *port, &CMD_VM.set(Side::Main, VmMode::Memory))?;
        let _ = cat_send(&mut *port, &CMD_MC.set(Side::Main, mem.channel.clone()))?;
        let _ = cat_send(&mut *port, &CMD_VM.set(Side::Main, VmMode::Vfo))?;
        // Set shift while in FM mode (OS is only accepted in FM), then flip to
        // the target mode. This also clears stale shift state on non-FM channels.
        let _ = cat_send(&mut *port, &CMD_MD.set(Side::Main, Mode::Fm))?;
        let _ = cat_send(&mut *port, &CMD_OS.set(Side::Main, mem.shift.clone()))?;
        let _ = cat_send(&mut *port, &CMD_MD.set(Side::Main, mem.mode.clone()))?;
        let _ = cat_send(&mut *port, &CMD_FA.set(mem.frequency_hz))?;
        let _ = cat_send(&mut *port, &CMD_CT.set(Side::Main, mem.sql_type.clone()))?;
        let ctcss_code = CmdCn::tone_code_from_string(ToneType::Ctcss, &rec.ctcss_tone)?;
        let _ = cat_send(&mut *port, &CMD_CN.set(Side::Main, ToneType::Ctcss, ctcss_code))?;
        let dcs_code = CmdCn::tone_code_from_string(ToneType::Dcs, &rec.dcs_tone)?;
        let _ = cat_send(&mut *port, &CMD_CN.set(Side::Main, ToneType::Dcs, dcs_code))?;
        // Commit the full VFO state to the selected memory channel.
        let _ = cat_send(&mut *port, &CMD_AM.save())?;
        if let Some(tag) = rec.tag {
            debug!("Writing tag for channel: {:?}, tag: {:?}", mem.channel, tag);
            let _ = cat_send(&mut *port, &CMD_MT.set(mem.channel, tag)?)?;
        }
    }
    bar.finish();
    if !quiet { println!("Memory data written to radio."); }

    Ok(())
}
