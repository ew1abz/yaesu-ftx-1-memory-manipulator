use indicatif::ProgressBar;
use log::{debug, error};
use std::io;
use std::time::Duration;

mod ftx1;

const RX_BUFFER_SIZE: usize = 255;
struct Record {
    mem: ftx1::MemoryRead,
    tag: Option<String>,
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
    let bar = ProgressBar::new(999);
    // ---------
    for ch in 1..=999 {
        bar.inc(1);
        let mem = read_mem(&mut *port, ch);
        match mem {
            Ok(m) => {
                let tag = read_tag(&mut *port, ch);
                db.push(Record { mem: m, tag: tag });
            }
            Err(_) => (), //println!("Error: {:?}", e),
        }
    }
    bar.finish();
    for r in &db {
        print!("Memory: {:}", &r.mem);
        if r.tag.is_some() {
            println!(" Tag: {:}", &r.tag.clone().unwrap());
        }
    }

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
