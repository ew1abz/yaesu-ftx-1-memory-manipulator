#![allow(dead_code)]
use core::fmt;
use log::debug;
use serde::{Deserialize, Serialize};

// include parsing helpers from a separate file so both the binary module and the library
// can use the same implementation. The file `src/parsers.rs` lives next to this file.
#[path = "parsers.rs"]
pub mod parsers;
use parsers::{buf4_to_i16, buf4_to_u16, buf9_to_u32};

//------------------------------------
// RX Clarifier
//------------------------------------
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum RxClarifierOnOff {
    RxClarifierOff = 0x00,
    RxClarifierOn = 0x01,
}

impl fmt::Display for RxClarifierOnOff {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                RxClarifierOnOff::RxClarifierOff => "RxClarifierOff",
                RxClarifierOnOff::RxClarifierOn => "RxClarifierOn",
            },
        )
    }
}

impl TryFrom<char> for RxClarifierOnOff {
    type Error = ();

    fn try_from(item: char) -> Result<Self, Self::Error> {
        match item {
            '0' => Ok(RxClarifierOnOff::RxClarifierOff),
            '1' => Ok(RxClarifierOnOff::RxClarifierOn),
            _ => Err(()),
        }
    }
}

//------------------------------------
// TX Clarifier
//------------------------------------
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TxClarifierOnOff {
    TxClarifierOff = 0x00,
    TxClarifierOn = 0x01,
}

impl fmt::Display for TxClarifierOnOff {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                TxClarifierOnOff::TxClarifierOff => "TxClarifierOff",
                TxClarifierOnOff::TxClarifierOn => "TxClarifierOn",
            },
        )
    }
}

impl TryFrom<char> for TxClarifierOnOff {
    type Error = ();

    fn try_from(item: char) -> Result<Self, Self::Error> {
        match item {
            '0' => Ok(TxClarifierOnOff::TxClarifierOff),
            '1' => Ok(TxClarifierOnOff::TxClarifierOn),
            _ => Err(()),
        }
    }
}

//------------------------------------
// Memory Channel
//------------------------------------
// 00000: VFO or MT or QMB (5 Bytes)
// 00001 - 00999: (Memory Channel)
// P-01L - P-50U: (PMS)
// 50001 - 50020: (5MHz BAND)
// EMGCH: (EMERGENCY CH)

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PmsLowerUpper {
    Lower = 0x00,
    Upper = 0x01,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PmsChannel {
    pub slot: u8, // 01-50
    pub lower_upper: PmsLowerUpper,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum MemoryChannel {
    VfoMtQmb,
    Mem(u16),
    Pms(PmsChannel),
    FiveMHzBand(u8),
    EmergencyChannel,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ChType {
    Vfo = 0x00,
    MemoryChannel = 0x01,
    MemoryTune = 0x02,
    Qmb = 0x03,
    Reserved4 = 0x04,
    Pms = 0x05,
}

impl TryFrom<char> for ChType {
    type Error = ();

    fn try_from(item: char) -> Result<Self, Self::Error> {
        match item {
            '0' => Ok(ChType::Vfo),
            '1' => Ok(ChType::MemoryChannel),
            '2' => Ok(ChType::MemoryTune),
            '3' => Ok(ChType::Qmb),
            '4' => Ok(ChType::Reserved4),
            '5' => Ok(ChType::Pms),
            _ => Err(()),
        }
    }
}

impl fmt::Display for ChType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChType::Vfo => write!(f, "VFO"),
            ChType::MemoryChannel => write!(f, "Memory"),
            ChType::MemoryTune => write!(f, "MemoryTune"),
            ChType::Qmb => write!(f, "QMB"),
            ChType::Reserved4 => write!(f, "Reserved"),
            ChType::Pms => write!(f, "PMS"),
        }
    }
}

impl TryFrom<&[char; 5]> for MemoryChannel {
    type Error = ();

    fn try_from(item: &[char; 5]) -> Result<Self, Self::Error> {
        // diagnostic: show the incoming 5-char channel identifier
        // debug!("DEBUG: MemoryChannel::try_from input: {:?}", item);
        match item {
            ['0', '0', '0', '0', '0'] => Ok(Self::VfoMtQmb),
            ['0', _, _, _, _] => {
                // Memory channel: parse as u16 (00001 - 00999)
                let ch =
                    buf4_to_u16(&[item[1] as u8, item[2] as u8, item[3] as u8, item[4] as u8])?;
                Ok(Self::Mem(ch))
            }
            ['P', _, _, _, _] => {
                // PMS channel: e.g., P-01L, P-50U
                // Parse slot (positions 2-3) and L/U suffix (position 4)
                let slot_str = format!("{}{}", item[2], item[3]);
                let slot = slot_str.parse::<u8>().map_err(|_| ())?;
                let lower_upper = match item[4] {
                    'L' => PmsLowerUpper::Lower,
                    'U' => PmsLowerUpper::Upper,
                    _ => return Err(()),
                };
                Ok(Self::Pms(PmsChannel { slot, lower_upper }))
            }
            ['5', _, _, _, _] => {
                // 5MHz band: parse as u16 (50001 - 50020)
                let band =
                    buf4_to_u16(&[item[1] as u8, item[2] as u8, item[3] as u8, item[4] as u8])?;
                Ok(Self::FiveMHzBand(band as u8))
            }
            ['E', 'M', 'G', 'C', 'H'] => Ok(Self::EmergencyChannel),
            _ => Err(()),
        }
    }
}

impl MemoryChannel {
    pub fn to_chars(&self) -> Result<[char; 5], ()> {
        match self {
            MemoryChannel::VfoMtQmb => Ok(['0', '0', '0', '0', '0']),
            MemoryChannel::Mem(ch) => {
                let s = format!("{:05}", ch);
                let chars: Vec<char> = s.chars().collect();
                Ok([chars[0], chars[1], chars[2], chars[3], chars[4]])
            }
            MemoryChannel::Pms(pms) => {
                let lu = match pms.lower_upper {
                    PmsLowerUpper::Lower => 'L',
                    PmsLowerUpper::Upper => 'U',
                };
                let s = format!("P-{:02}{}", pms.slot, lu);
                let chars: Vec<char> = s.chars().collect();
                Ok([chars[0], chars[1], chars[2], chars[3], chars[4]])
            }
            MemoryChannel::FiveMHzBand(band) => {
                let h = (band / 10) as char;
                let l = (band % 10) as char;
                Ok(['5', '0', '0', h, l])
            }
            MemoryChannel::EmergencyChannel => Ok(['E', 'M', 'G', 'C', 'H']),
        }
    }
}

impl fmt::Display for MemoryChannel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MemoryChannel::VfoMtQmb => write!(f, "VFO/MT/QMB"),
            MemoryChannel::Mem(ch) => write!(f, "Mem({})", ch),
            MemoryChannel::Pms(pms) => {
                let lu = match pms.lower_upper {
                    PmsLowerUpper::Lower => "L",
                    PmsLowerUpper::Upper => "U",
                };
                write!(f, "PMS-{:02}{}", pms.slot, lu)
            }
            MemoryChannel::FiveMHzBand(band) => write!(f, "5MHz Band({})", band),
            MemoryChannel::EmergencyChannel => write!(f, "EMGCH"),
        }
    }
}

//------------------------------------
// Shift
//------------------------------------
// [0: Simplex 1: Plus Shift 2: Minus Shift]

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Shift {
    Simplex = 0x00,
    PlusShift = 0x01,
    MinusShift = 0x02,
}

impl TryFrom<char> for Shift {
    type Error = ();

    fn try_from(item: char) -> Result<Self, Self::Error> {
        match item {
            '0' => Ok(Self::Simplex),
            '1' => Ok(Self::PlusShift),
            '2' => Ok(Self::MinusShift),
            _ => Err(()),
        }
    }
}

impl fmt::Display for Shift {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Shift::Simplex => write!(f, "SIMPLEX"),
            Shift::PlusShift => write!(f, "PLUS SHIFT"),
            Shift::MinusShift => write!(f, "MINUS SHIFT"),
        }
    }
}

//------------------------------------
// Tone
//------------------------------------

// [0: CTCSS “OFF” 1: CTCSS ENC/DEC 2: CTCSS ENC]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Tone {
    CtcssOff = 0x00,
    CtcssEncDec = 0x01,
    CtcssEnc = 0x02,
}

impl TryFrom<char> for Tone {
    type Error = ();

    fn try_from(item: char) -> Result<Self, Self::Error> {
        match item {
            '0' => Ok(Self::CtcssOff),
            '1' => Ok(Self::CtcssEncDec),
            '2' => Ok(Self::CtcssEnc),
            _ => Err(()),
        }
    }
}

impl fmt::Display for Tone {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Tone::CtcssOff => write!(f, "CTCSS_OFF"),
            Tone::CtcssEncDec => write!(f, "CTCSS_ENCDEC"),
            Tone::CtcssEnc => write!(f, "CTCSS_ENC"),
        }
    }
}

//------------------------------------
// Mode
//------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
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

//------------------------------------
// Cmd
//------------------------------------

type CmdError = ();
pub struct Cmd<'a> {
    code: &'a [char; 2],
    read_params: usize,
}

impl Cmd<'_> {
    /// Constructs tx buffer, includes the params and the terminator into it.
    pub fn tx_buffer(&self, params: Option<Vec<char>>) -> Vec<u8> {
        let mut tx_vec = Vec::<u8>::new();
        tx_vec.extend([self.code[0] as u8, self.code[1] as u8].iter().cloned());
        if let Some(p) = params {
            p.iter().for_each(|b| tx_vec.push(*b as u8));
        }
        tx_vec.push(b';');
        tx_vec
    }

    /// Validate received packet from a transceiver.
    /// Returns Ok() if the answer is valid, Error() otherwise.
    fn is_reply_ok(&self, rx_buffer: &Vec<u8>) -> Result<(), CmdError> {
        if rx_buffer.len() < 3 {
            return Err(());
        }
        let code0 = rx_buffer.contains(&(self.code[0] as u8));
        let code1 = rx_buffer.contains(&(self.code[1] as u8));
        let params = rx_buffer.len() - 3 == self.read_params;
        let terminator = rx_buffer.contains(&b';');
        debug!("{} {} {} {} {}", &code0, &code1, &params, rx_buffer.len() - 3, &terminator);
        (terminator & code0 & code1 & params).then_some(()).ok_or(())
    }
}

//------------------------------------
// CmdId
//------------------------------------

pub struct CmdId<'a> {
    cmd: Cmd<'a>,
}

/// Identification
pub const CMD_ID: CmdId<'static> = CmdId { cmd: Cmd { code: &['I', 'D'], read_params: 4 } };
pub const FTX1_ID: u16 = 840;
pub const FTDX5000: u16 = 362;
pub const FT991A: u16 = 362;
pub const FTDX101D: u16 = 362;
pub const FTDX101MP: u16 = 362;
pub const FTDX10: u16 = 362;

impl CmdId<'_> {
    pub fn read(&self) -> Vec<u8> {
        Cmd::tx_buffer(&self.cmd, None)
    }

    pub fn decode(&self, buffer: &Vec<u8>) -> Result<u16, ()> {
        Cmd::is_reply_ok(&self.cmd, buffer)?;
        let id = buf4_to_u16(&buffer[2..6])?;
        Ok(id)
    }

    pub fn validate(&self, id: u16) -> Result<(), ()> {
        if id == FTX1_ID {
            Ok(())
        } else {
            Err(())
        }
    }
}

//------------------------------------
// CmdMemoryRead
//------------------------------------
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryRead {
    pub channel: MemoryChannel,                 // 5 positions [00001]
    pub frequency_hz: u32,                      // 9 positions [432100000]
    pub clarifier_offset_hz: i16,               // 5 positions [+0015]
    pub rx_clarifier_enabled: RxClarifierOnOff, // 1 position [0: OFF, 1: ON]
    pub tx_clarifier_enabled: TxClarifierOnOff, // 1 position [0: OFF, 1: ON]
    pub mode: Mode,                             // 1 positions
    pub ch_type: ChType, // 1 position [0: VFO 1: Memory Channel 2: Memory Tune 3: Quick Memory Bank (QMB) 4: - 5: PMS]
    pub tone: Tone,      // 1 position [0: CTCSS “OFF” 1: CTCSS ENC/DEC 2: CTCSS ENC]
    pub shift: Shift,    // 1 position [0: Simplex 1: Plus Shift 2: Minus Shift]
}

impl Default for MemoryRead {
    fn default() -> Self {
        Self {
            channel: MemoryChannel::VfoMtQmb,
            frequency_hz: 0,
            clarifier_offset_hz: 0,
            rx_clarifier_enabled: RxClarifierOnOff::RxClarifierOff,
            tx_clarifier_enabled: TxClarifierOnOff::TxClarifierOff,
            mode: Mode::Lsb,
            ch_type: ChType::Vfo,
            tone: Tone::CtcssOff,
            shift: Shift::Simplex,
        }
    }
}

pub struct CmdMr<'a> {
    cmd: Cmd<'a>,
}

pub const CMD_MR: CmdMr<'static> = CmdMr { cmd: Cmd { code: &['M', 'R'], read_params: 27 } };

impl CmdMr<'_> {
    pub fn read(&self, ch: MemoryChannel) -> Vec<u8> {
        let s = ch.to_chars().unwrap();
        debug!("DEBUG: CMD_MT::read input: {:?}", s);
        Cmd::tx_buffer(&self.cmd, Some(s.to_vec()))
    }

    pub fn decode(&self, buffer: &Vec<u8>) -> Result<MemoryRead, ()> {
        // MR00001007000000+000000110000;
        let mut mr = MemoryRead::default();
        Cmd::is_reply_ok(&self.cmd, buffer)?;
        let ch_chars: [char; 5] = [
            buffer[2] as char,
            buffer[3] as char,
            buffer[4] as char,
            buffer[5] as char,
            buffer[6] as char,
        ];
        mr.channel = MemoryChannel::try_from(&ch_chars)?;
        mr.frequency_hz = buf9_to_u32(&buffer[7..16])?;
        mr.clarifier_offset_hz = buf4_to_i16(&buffer[16..21])?;
        mr.rx_clarifier_enabled = RxClarifierOnOff::try_from(buffer[21] as char)?;
        mr.tx_clarifier_enabled = TxClarifierOnOff::try_from(buffer[22] as char)?;
        mr.mode = Mode::try_from(buffer[23] as char)?;
        mr.ch_type = ChType::try_from(buffer[24] as char)?;
        mr.tone = Tone::try_from(buffer[25] as char)?;
        let _dummy = buffer[26] | buffer[27];
        mr.shift = Shift::try_from(buffer[28] as char)?;

        Ok(mr)
    }
}

impl fmt::Display for MemoryRead {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "#{}({}), Frequency: {} Hz, Mode: {}, Tone: {}, Shift: {}, Clarifier: {:+} Hz, RX: {}, TX: {}",
            self.channel,
            self.ch_type,
            self.frequency_hz,
            self.mode,
            self.tone,
            self.shift,
            self.clarifier_offset_hz,
            self.rx_clarifier_enabled,
            self.tx_clarifier_enabled,
        )
    }
}

//------------------------------------
// MT - MEMORY CHANNEL TAG WRITE
//------------------------------------
pub struct CmdMt<'a> {
    cmd: Cmd<'a>,
}

pub const CMD_MT: CmdMt<'static> = CmdMt { cmd: Cmd { code: &['M', 'T'], read_params: 17 } };

impl CmdMt<'_> {
    pub fn read(&self, ch: MemoryChannel) -> Vec<u8> {
        let s = ch.to_chars().unwrap();
        debug!("CMD_MT::read input: {:?}", s);
        Cmd::tx_buffer(&self.cmd, Some(s.to_vec()))
    }

    pub fn decode(&self, buffer: &Vec<u8>) -> Result<String, ()> {
        debug!("CMD_MT::decode input: {:?}", buffer);
        Cmd::is_reply_ok(&self.cmd, buffer)?;
        let _channel = &buffer[2..6];
        let tag = buffer[7..19].iter().map(|&b| b as char).collect();
        Ok(tag)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_channel_to_chars() {
        // VFO
        assert_eq!(MemoryChannel::VfoMtQmb.to_chars().unwrap(), ['0', '0', '0', '0', '0']);

        // Memory Channel
        assert_eq!(MemoryChannel::Mem(1).to_chars().unwrap(), ['0', '0', '0', '0', '1']);
        assert_eq!(MemoryChannel::Mem(123).to_chars().unwrap(), ['0', '0', '1', '2', '3']);

        // PMS
        let pms_l = PmsChannel { slot: 1, lower_upper: PmsLowerUpper::Lower };
        assert_eq!(MemoryChannel::Pms(pms_l).to_chars().unwrap(), ['P', '-', '0', '1', 'L']);

        let pms_u = PmsChannel { slot: 50, lower_upper: PmsLowerUpper::Upper };
        assert_eq!(MemoryChannel::Pms(pms_u).to_chars().unwrap(), ['P', '-', '5', '0', 'U']);

        // Emergency
        assert_eq!(MemoryChannel::EmergencyChannel.to_chars().unwrap(), ['E', 'M', 'G', 'C', 'H']);
    }
}
