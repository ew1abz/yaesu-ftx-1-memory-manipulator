use clap::Parser;
use ftdx_1chm::ftx1::MemoryChannel;
use indicatif::ProgressBar;
use log::{debug, error};
use serde::{Deserialize, Serialize};
use std::io;
use std::time::Duration;

mod ftx1;

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

    /// File to save/read memory data
    #[arg(short, long, default_value = "output.csv")]
    file: String,

    /// Read from radio
    #[arg(long, group = "action")]
    read_radio: bool,

    /// Write to radio
    #[arg(long, group = "action")]
    write_radio: bool,

    /// Check data in the file
    #[arg(long, group = "action")]
    check_data: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct CsvRecord {
    #[serde(rename = "Channel Number")]
    channel: String,
    #[serde(rename = "Frequency (Hz)")]
    freq: u32, // ftx1::FrequencyHz,
    #[serde(rename = "Memory Tag")]
    tag: Option<String>,
    #[serde(rename = "Mode")]
    mode: String,
    #[serde(rename = "Channel Type")]
    ch_type: ftx1::ChType,
    #[serde(rename = "Squelch Type")]
    tone: ftx1::SqlType,
    #[serde(rename = "Shift (Hz)")]
    shift: ftx1::Shift,
    #[serde(rename = "Clarifier Offset (Hz)")]
    clarifier_offset_hz: i16,
    #[serde(rename = "Rx Clarifier Enabled")]
    rx_clarifier_enabled: ftx1::RxClarifierOnOff,
    #[serde(rename = "Tx Clarifier Enabled")]
    tx_clarifier_enabled: ftx1::TxClarifierOnOff,
}

fn main() -> Result<(), ()> {
    let cli = Cli::parse();
    env_logger::init();

    if cli.read_radio {
        read_radio_data(&cli)?;
    } else if cli.write_radio {
        println!("Writing to radio is not implemented yet.");
    } else if cli.check_data {
        check_data(&cli.file)?;
    } else {
        println!("No action specified. Use --help for options.");
    }

    Ok(())
}

fn check_data(file_path: &str) -> Result<(), ()> {
    println!("Checking data in file: {}", file_path);
    let mut rdr = csv::Reader::from_path(file_path).unwrap();
    let mut valid_records = 0;
    let mut invalid_records = 0;

    for (i, result) in rdr.deserialize().enumerate() {
        let record: CsvRecord = match result {
            Ok(r) => r,
            Err(e) => {
                println!("Error deserializing record {}: {}", i + 1, e);
                invalid_records += 1;
                continue;
            }
        };

        match validate_record(&record) {
            Ok(_) => {
                valid_records += 1;
            }
            Err(errors) => {
                invalid_records += 1;
                println!("Record {} is invalid:", i + 1);
                for error in errors {
                    println!("  - {}", error);
                }
            }
        }
    }

    println!("\n----- Validation Summary -----");
    println!("Total records processed: {}", valid_records + invalid_records);
    println!("Valid records: {}", valid_records);
    println!("Invalid records: {}", invalid_records);

    if invalid_records == 0 {
        println!("\nData looks good!");
    } else {
        println!("\nData has issues and may not be processable.");
    }

    Ok(())
}

fn validate_record(record: &CsvRecord) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    // Validate channel
    if record.channel.len() != 5 {
        errors.push(format!("Channel '{}' has invalid length. Expected 5.", record.channel));
    } else {
        let chars: Vec<char> = record.channel.chars().collect();
        let ch_array: [char; 5] = [chars[0], chars[1], chars[2], chars[3], chars[4]];
        if ftx1::MemoryChannel::try_from(&ch_array).is_err() {
            errors.push(format!("Channel '{}' is not a valid memory channel.", record.channel));
        }
    }

    // Validate frequency
    if ftx1::FrequencyHz::try_from(record.freq).is_err() {
        errors.push(format!("Frequency '{}' is not valid.", record.freq));
    }

    // Validate clarifier offset: 0000 - 9990 (Hz)
    if ftx1::ClarifierOffsetHz::try_from(record.clarifier_offset_hz).is_err() {
        errors.push(format!("Clarifier offset '{}' is not a valid number.", record.clarifier_offset_hz));
    }

    // Validate mode
    const VALID_MODES: &[&str] = &["LSB", "USB", "CW-U", "FM", "AM", "RTTY-L", "CW-L", "DATA-L", "RTTY-U", "DATA-FM", "FM-N", "DATA-U", "AM-N", "PSK", "DATA-FM-N"];
    if !VALID_MODES.contains(&record.mode.as_str()) {
        errors.push(format!("Mode '{}' is not a valid mode.", record.mode));
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn read_radio_data(cli: &Cli) -> Result<(), ()> {
    println!("Reading from radio...");
    let mut port = serialport::new(&cli.port, cli.speed)
        .timeout(Duration::from_millis(200))
        .open()
        .expect("Failed to open port");

    read_validate_id(&mut *port)?;
    let mut wtr = csv::Writer::from_path(&cli.file).unwrap();
    let bar = ProgressBar::new(CHANNELS as u64);

    for ch in 1..=CHANNELS {
        bar.inc(1);
        let mem = read_mem(&mut *port, ch);
        match mem {
            Ok(m) => {
                let tag = read_tag(&mut *port, ch);
                let csv_record: CsvRecord = CsvRecord {
                    channel: MemoryChannel::to_chars(&MemoryChannel::Mem(ch))
                        .unwrap()
                        .into_iter()
                        .collect::<String>(),
                    tag: tag,
                    freq: m.frequency_hz.to_u32(),
                    clarifier_offset_hz: m.clarifier_offset_hz.to_i16(),
                    rx_clarifier_enabled: m.rx_clarifier_enabled,
                    tx_clarifier_enabled: m.tx_clarifier_enabled,
                    mode: m.mode.to_string(),
                    ch_type: m.ch_type,
                    tone: m.sql_type,
                    shift: m.shift,
                };
                wtr.serialize(&csv_record).unwrap();
            }
            Err(_) => (),
        }
    }
    bar.finish();
    wtr.flush().unwrap();
    println!("Data saved to {}", cli.file);
    Ok(())
}

fn read_validate_id(port: &mut dyn serialport::SerialPort) -> Result<(), ()> {
    let rx = cat_send(port, &ftx1::CMD_ID.read())?;
    let id = ftx1::CMD_ID.decode(&rx)?;
    match ftx1::CMD_ID.validate(id) {
        Ok(_) => println!("Yaesu FTX-1 found (radio ID: {:04})", &id),
        Err(e) => println!("Can't connect to Yaesu FTX-1: {:?}", e),
    }
    Ok(())
}

fn read_mem(port: &mut dyn serialport::SerialPort, ch: u16) -> Result<ftx1::MemoryRead, ()> {
    let rx = cat_send(port, &ftx1::CMD_MR.read(ftx1::MemoryChannel::Mem(ch)))?;
    ftx1::CMD_MR.decode(&rx)
}

fn read_tag(port: &mut dyn serialport::SerialPort, ch: u16) -> Option<String> {
    debug!("Reading tag for channel: {:?}", ch);
    let rx = cat_send(port, &ftx1::CMD_MT.read(ftx1::MemoryChannel::Mem(ch))).unwrap();
    let d = ftx1::CMD_MT.decode(&rx);
    match &d {
        Ok(tag) => debug!("Tag: {:}", &tag),
        Err(e) => error!("Error: {:?}", e),
    }
    d.ok()
}

fn cat_send(port: &mut dyn serialport::SerialPort, data: &[u8]) -> Result<Vec<u8>, ()> {
    port.write(data).expect("failed to write message");
    let mut buffer: Vec<u8> = vec![0; RX_BUFFER_SIZE];
    loop {
        match port.read(buffer.as_mut_slice()) {
            Ok(n) => {
                buffer.truncate(n);
                break;
            }
            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => {
                break;
            }
            Err(e) => eprintln!("{:?}", e),
        }
    }
    Ok(buffer)
}

// fn print_buffer(header: &str, v: Option<&Vec<u8>>) {
//     print!("\n{}: ", header);
//     match v {
//         Some(data) => {
//             // Print hex
//             for x in data {
//                 print!("{:02X} ", x);
//             }
//             print!("| ");
//             // Print ASCII
//             for &x in data {
//                 if x >= 32 && x < 127 {
//                     print!("{}", x as char);
//                 } else {
//                     print!(".");
//                 }
//             }
//         }
//         None => print!("[]"),
//     }
//     print!("\n");
// }
