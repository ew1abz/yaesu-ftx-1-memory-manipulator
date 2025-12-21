#![allow(dead_code)]

use core::fmt;
use heapless::Vec;

type CmdError = ();

pub enum Band {
    Main,
    Sub,
}

impl Band {
    fn code(&self) -> char {
        match self {
            Band::Main => '0',
            Band::Sub => '1',
        }
    }
}

impl TryFrom<char> for Band {
    type Error = ();

    fn try_from(item: char) -> Result<Self, Self::Error> {
        match item {
            '0' => Ok(Self::Main),
            '1' => Ok(Self::Sub),
            _ => Err(()),
        }
    }
}

impl TryFrom<u8> for Band {
    type Error = ();

    fn try_from(item: u8) -> Result<Self, Self::Error> {
        Band::try_from(item as char)
    }
}

impl fmt::Display for Band {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Band::Main => write!(f, "Main"),
            Band::Sub => write!(f, "Sub"),
        }
    }
}

pub enum Mode {
    Lsb = 0x01,
    Usb = 0x02,
    CwU = 0x03,
    Fm = 0x04,
    Am = 0x05,
    RttyL = 0x06,
    CwL = 0x07,
    DataL = 0x08,
    RttyU = 0x09,
    DataFm = 0x0a,
    FmN = 0x0b,
    DataU = 0x0c,
    AmN = 0x0d,
    Psk = 0x0e,
    DataFmN = 0x0f,
}

impl Mode {
    fn code(&self) -> char {
        match self {
            Self::Lsb => '1',
            Self::Usb => '2',
            Self::CwU => '3',
            Self::Fm => '4',
            Self::Am => '5',
            Self::RttyL => '6',
            Self::CwL => '7',
            Self::DataL => '8',
            Self::RttyU => '9',
            Self::DataFm => 'A',
            Self::FmN => 'B',
            Self::DataU => 'C',
            Self::AmN => 'D',
            Self::Psk => 'E',
            Self::DataFmN => 'F',
        }
    }
}

impl TryFrom<char> for Mode {
    type Error = ();

    fn try_from(item: char) -> Result<Self, Self::Error> {
        match item {
            '1' => Ok(Self::Lsb),
            '2' => Ok(Self::Usb),
            '3' => Ok(Self::CwU),
            '4' => Ok(Self::Fm),
            '5' => Ok(Self::Am),
            '6' => Ok(Self::RttyL),
            '7' => Ok(Self::CwL),
            '8' => Ok(Self::DataL),
            '9' => Ok(Self::RttyU),
            'A' => Ok(Self::DataFm),
            'B' => Ok(Self::FmN),
            'C' => Ok(Self::DataU),
            'D' => Ok(Self::AmN),
            'E' => Ok(Self::Psk),
            'F' => Ok(Self::DataFmN),
            _ => Err(()),
        }
    }
}

impl TryFrom<u8> for Mode {
    type Error = ();

    fn try_from(item: u8) -> Result<Self, Self::Error> {
        Mode::try_from(item as char)
    }
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Mode::Lsb => write!(f, "LSB"),
            Mode::Usb => write!(f, "USB"),
            Mode::CwU => write!(f, "CW-U"),
            Mode::Fm => write!(f, "FM"),
            Mode::Am => write!(f, "AM"),
            Mode::RttyL => write!(f, "RTTY-L"),
            Mode::CwL => write!(f, "CW-L"),
            Mode::DataL => write!(f, "DATA-L"),
            Mode::RttyU => write!(f, "RTTY-U"),
            Mode::DataFm => write!(f, "DATA-FM"),
            Mode::FmN => write!(f, "FM-N"),
            Mode::DataU => write!(f, "DATA-U"),
            Mode::AmN => write!(f, "AM-N"),
            Mode::Psk => write!(f, "PSK"),
            Mode::DataFmN => write!(f, "DATA-FM-N"),
        }
    }
}

pub struct Cmd<'a> {
    code: &'a [char; 2],
    read_params: usize,
}

impl Cmd<'_> {
    /// Constructs tx buffer, includes the params and the terminator into it.
    pub fn tx_buffer(&self, params: Option<Vec<char, 8>>) -> Vec<u8, 8> {
        let mut tx_vec = Vec::<u8, 8>::new();
        tx_vec.extend([self.code[0] as u8, self.code[1] as u8].iter().cloned());
        if let Some(p) = params {
            p.iter().for_each(|b| tx_vec.push(*b as u8).unwrap());
        }
        tx_vec.push(b';').ok();
        tx_vec
    }

    /// Validate received packet from a transceiver.
    /// Returns Ok() if the answer is valid, Error() otherwise.
    fn is_reply_ok(&self, rx_buffer: &Vec<u8, { crate::RX_BUFFER_SIZE }>) -> Result<(), CmdError> {
        if rx_buffer.len() < 3 {
            return Err(());
        }
        let code0 = rx_buffer.contains(&(self.code[0] as u8));
        let code1 = rx_buffer.contains(&(self.code[1] as u8));
        let params = rx_buffer.len() - 3 == self.read_params;
        let terminator = rx_buffer.contains(&b';');
        (terminator & code0 & code1 & params).then_some(()).ok_or(())
    }
}

pub struct CmdId<'a> {
    cmd: Cmd<'a>,
}

pub struct CmdMd<'a> {
    pub cmd: Cmd<'a>,
}

pub struct CmdPc<'a> {
    pub cmd: Cmd<'a>,
}

pub struct CmdTx<'a> {
    cmd: Cmd<'a>,
}

/// Identification
pub const CMD_ID: CmdId<'static> = CmdId { cmd: Cmd { code: &['I', 'D'], read_params: 4 } };
/// Operating Mode
pub const CMD_MD: CmdMd<'static> = CmdMd { cmd: Cmd { code: &['M', 'D'], read_params: 2 } };
/// Power Control (005 - 100)
pub const CMD_PC: CmdPc<'static> = CmdPc { cmd: Cmd { code: &['P', 'C'], read_params: 3 } };
///  TX
pub const CMD_TX: CmdTx<'static> = CmdTx { cmd: Cmd { code: &['T', 'X'], read_params: 1 } };

impl CmdMd<'_> {
    pub fn set(&self, band: &Band, mode: &Mode) -> Vec<u8, 8> {
        let mut params = Vec::<char, 8>::new();
        params.push(band.code()).ok();
        params.push(mode.code()).ok();
        Cmd::tx_buffer(&self.cmd, Some(params))
    }

    pub fn read(&self, band: Band) -> Vec<u8, 8> {
        let mut params = Vec::<char, 8>::new();
        params.push(band.code()).ok();
        Cmd::tx_buffer(&self.cmd, Some(params))
    }

    pub fn decode(&self, buffer: &Vec<u8, { crate::RX_BUFFER_SIZE }>) -> Result<(Band, Mode), ()> {
        Cmd::is_reply_ok(&self.cmd, buffer)?;
        let band = Band::try_from(buffer[2])?;
        let mode = Mode::try_from(buffer[3])?;
        Ok((band, mode))
    }
}

impl CmdId<'_> {
    pub fn read(&self) -> Vec<u8, 8> {
        Cmd::tx_buffer(&self.cmd, None)
    }

    pub fn decode(&self, buffer: &Vec<u8, { crate::RX_BUFFER_SIZE }>) -> Result<u16, ()> {
        Cmd::is_reply_ok(&self.cmd, buffer)?;
        let id = buf4_to_u16(&buffer[2..6])?;
        Ok(id)
    }

    pub fn validate(id: u16) -> Result<(), ()> {
        // 0362: FTDX5000
        // 0670: FT-991A
        // 0681: FTDX101D
        // 0682: FTDX101MP
        // 0761: FTDX10
        if id == 761 {
            Ok(())
        } else {
            Err(())
        }
    }
}

impl CmdPc<'_> {
    pub fn set(&self, power: u8) -> Result<Vec<u8, 8>, ()> {
        if !(5..=100).contains(&power) {
            return Err(());
        }
        let mut params = Vec::<char, 8>::new();
        let chars = base_10_chars_3(power);
        params.extend_from_slice(&chars[..]).ok();
        Ok(Cmd::tx_buffer(&self.cmd, Some(params)))
    }

    pub fn read(&self) -> Vec<u8, 8> {
        Cmd::tx_buffer(&self.cmd, None)
    }

    pub fn decode(&self, buffer: &Vec<u8, { crate::RX_BUFFER_SIZE }>) -> Result<u8, ()> {
        Cmd::is_reply_ok(&self.cmd, buffer)?;
        let power = buf3_to_u8(&buffer[2..5])?;
        Ok(power)
    }
}

impl CmdTx<'_> {
    pub fn set(&self, tx: bool) -> Result<Vec<u8, 8>, ()> {
        let mut params = Vec::<char, 8>::new();
        params.push(if tx { '1' } else { '0' }).ok();
        Ok(Cmd::tx_buffer(&self.cmd, Some(params)))
    }

    pub fn read(&self) -> Vec<u8, 8> {
        Cmd::tx_buffer(&self.cmd, None)
    }

    pub fn decode(&self, buffer: &Vec<u8, { crate::RX_BUFFER_SIZE }>) -> Result<bool, ()> {
        Cmd::is_reply_ok(&self.cmd, buffer)?;
        Ok(buffer[2] != b'0')
    }
}

fn base_10_chars_3(n: u8) -> [char; 3] {
    let buf = &mut ['0'; 3];
    let mut n = n;

    buf[0] = ((n - (n % 100)) / 100 + b'0') as char;
    n %= 100;
    buf[1] = ((n - (n % 10)) / 10 + b'0') as char;
    n %= 10;
    buf[2] = (n + b'0') as char;
    *buf
}

// fn u8_to_buf3(u8) -> Vec::<char, 8> {
//     let mut vec = Vec::<char, 8>::new();
//     vec.push()
// (power);
// }

fn buf3_to_u8(buffer: &[u8]) -> Result<u8, ()> {
    let mut result = 0;
    for (i, item) in buffer.iter().enumerate().take(3) {
        if let Some(n) = (*item as char).to_digit(10) {
            result += n as u8 * (10u8.pow(2 - i as u32));
        } else {
            return Err(());
        }
    }
    Ok(result)
}

fn buf4_to_u16(buffer: &[u8]) -> Result<u16, ()> {
    let mut result = 0;
    for (i, item) in buffer.iter().enumerate().take(4) {
        if let Some(n) = (*item as char).to_digit(10) {
            result += n as u16 * (10u16.pow(3 - i as u32));
        } else {
            return Err(());
        }
    }
    Ok(result)
}

// pub trait Read {
//     fn read(&self) -> Vec<u8, 8>;
// }

// impl Read for CmdId<'_> {
//     // const CMD: Cmd<'_> =
//         //  Cmd { code: &['I', 'D'], set_params: None, read_params: None, answer_params: 4 };

//     fn read(&self) -> Vec<u8, 8> {
//         let a = self.cmd.code[0];
//         Cmd::tx_buffer(&self.cmd, None)
//     }
// }

// impl set for CMD_MD {
//     pub fn set(&self, params: Option<Vec<u8, 8>>) -> Vec<u8, 8> {
//         let mut tx_vec = Vec::<u8, 8>::new();
//         tx_vec
//         }
// }
// "MD0;"   // read MAIN band mode
// "PC005;" // set power 5W
// "MD04" // Main band mode = FM
// "FT2;"  // transmit ON

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_mode() {
//         assert_eq!(Mode::try_from('0'), Ok(Mode::Lsb));
//     }
// }
