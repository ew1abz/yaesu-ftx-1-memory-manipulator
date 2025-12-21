use ftdx_1chm::ftx1::MemoryChannel;
use indicatif::ProgressBar;
use log::{debug, error};
use serde::Serialize;
use std::io;
use std::time::Duration;

mod ftx1;

const RX_BUFFER_SIZE: usize = 255;
const CHANNELS: u16 = 100;

#[derive(Debug, Serialize)]
struct Record {
    mem: ftx1::MemoryRead,
    tag: Option<String>,
}
#[derive(Debug, Serialize)]
struct CsvRecord {
    #[serde(rename = "Channel Number")]
    channel: String,
    #[serde(rename = "Frequency (Hz)")]
    freq: u32,
    #[serde(rename = "Memory Tag")]
    tag: Option<String>,
    #[serde(rename = "Clarifier Offset (Hz)")]
    clarifier_offset_hz: String,                  // 5 positions [+0015]
    #[serde(rename = "Rx Clarifier Enabled")]
    rx_clarifier_enabled: ftx1::RxClarifierOnOff, // 1 position [0: OFF, 1: ON]
    #[serde(rename = "Tx Clarifier Enabled")]
    tx_clarifier_enabled: ftx1::TxClarifierOnOff, // 1 position [0: OFF, 1: ON]
    #[serde(rename = "Mode")]
    mode: String,                                 // 1 positions
    #[serde(rename = "Channel Type")]
    ch_type: ftx1::ChType, // 1 position [0: VFO 1: Memory Channel 2: Memory Tune 3: Quick Memory Bank (QMB) 4: - 5: PMS]
    #[serde(rename = "CTCSS Tone")]
    tone: ftx1::Tone,      // 1 position [0: CTCSS “OFF” 1: CTCSS ENC/DEC 2: CTCSS ENC]
    #[serde(rename = "Shift (Hz)")]
    shift: ftx1::Shift,    // 1 position [0: Simplex 1: Plus Shift 2: Minus Shift]
}

fn main() -> Result<(), ()> {
    println!("Hello, world!");
    env_logger::init();
    let mut port = serialport::new("/dev/ttyUSB0", 38_400)
        .timeout(Duration::from_millis(200))
        .open()
        .expect("Failed to open port");

    read_validate_id(&mut *port)?;
    let mut db: Vec<Record> = Vec::new();
    let mut wtr = csv::Writer::from_writer(io::stdout());
    let mut wtrf = csv::Writer::from_path("output.csv").unwrap();
    let bar = ProgressBar::new(CHANNELS as u64);
    // ---------
    for ch in 1..=CHANNELS {
        bar.inc(1);
        let mem = read_mem(&mut *port, ch);
        match mem {
            Ok(m) => {
                let tag = read_tag(&mut *port, ch);
                db.push(Record { mem: m.clone(), tag: tag.clone() });
                let csv_record: CsvRecord = CsvRecord {
                    channel: MemoryChannel::to_chars(&MemoryChannel::Mem(ch))
                        .unwrap()
                        .into_iter()
                        .collect::<String>(),
                    tag: tag,
                    freq: m.frequency_hz,
                    clarifier_offset_hz: m.clarifier_offset_hz.to_string(), // 5 positions [+0015]
                    rx_clarifier_enabled: m.rx_clarifier_enabled, // 1 position [0: OFF, 1: ON]
                    tx_clarifier_enabled: m.tx_clarifier_enabled, // 1 position [0: OFF, 1: ON]
                    mode: m.mode.to_string(),                     // 1 positions
                    ch_type: m.ch_type, // 1 position [0: VFO 1: Memory Channel 2: Memory Tune 3: Quick Memory Bank (QMB) 4: - 5: PMS]
                    tone: m.tone,       // 1 position [0: CTCSS “OFF” 1: CTCSS ENC/DEC 2: CTCSS ENC]
                    shift: m.shift,     // 1 position [0: Simplex 1: Plus Shift 2: Minus Shift]
                };
                wtr.serialize(&csv_record).unwrap();
                wtrf.serialize(&csv_record).unwrap();
            }
            Err(_) => (), //println!("Error: {:?}", e),
        }
    }
    bar.finish();
    // for r in &db {
    //     print!("Memory: {:}", &r.mem);
    //     if r.tag.is_some() {
    //         println!(" Tag: {:}", &r.tag.clone().unwrap());
    //     }
    // }
    // ---------
    wtr.flush().unwrap();
    wtrf.flush().unwrap();

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
                // print_buffer("RX", Some(&buffer[..n]));
                buffer.truncate(n);
                break;
            }
            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => {
                print_buffer("RX", None);
                break;
            }
            Err(e) => eprintln!("{:?}", e),
        }
    }
    Ok(buffer)
}

fn print_buffer(header: &str, v: Option<&Vec<u8>>) {
    print!("\n{}: ", header);
    match v {
        Some(data) => {
            // Print hex
            for x in data {
                print!("{:02X} ", x);
            }
            print!("| ");
            // Print ASCII
            for &x in data {
                if x >= 32 && x < 127 {
                    print!("{}", x as char);
                } else {
                    print!(".");
                }
            }
        }
        None => print!("[]"),
    }
    print!("\n");
}
