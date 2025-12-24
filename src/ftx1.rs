#![allow(dead_code)]
use core::fmt;
use log::debug;
use serde::{Deserialize, Serialize};

// include parsing helpers from a separate file so both the binary module and the library
// can use the same implementation. The file `src/parsers.rs` lives next to this file.
#[path = "parsers.rs"]
pub mod parsers;
use parsers::{buf3_to_u8, buf4_to_i16, buf4_to_u16, buf9_to_u32};

//------------------------------------
// Frequency
//------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct FrequencyHz {
    value: u32,
}

impl FrequencyHz {
    pub fn to_u32(&self) -> u32 {
        self.value
    }
}

impl TryFrom<u32> for FrequencyHz {
    type Error = ();

    fn try_from(item: u32) -> Result<Self, Self::Error> {
        // 30kHz - 174MHz, 400MHz - 470MHz
        if item >= 30_000 && item < 174_000_000 || item >= 400_000_000 && item < 470_000_000 {
            Ok(FrequencyHz { value: item })
        } else {
            Err(())
        }
    }
}

impl TryFrom<&[u8]> for FrequencyHz {
    type Error = ();

    fn try_from(item: &[u8]) -> Result<Self, Self::Error> {
        if item.len() != 9 {
            return Err(());
        }
        let value = buf9_to_u32(item)?;
        FrequencyHz::try_from(value)
    }
}

impl TryFrom<String> for FrequencyHz {
    type Error = ();

    fn try_from(item: String) -> Result<Self, Self::Error> {
        if item.len() > 9 {
            return Err(());
        }
        FrequencyHz::try_from(item.as_bytes())
    }
}

impl fmt::Display for FrequencyHz {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:09}", self.value)
    }
}

//------------------------------------
// Clarifier offset
//------------------------------------
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ClarifierOffsetHz {
    value: i16,
}

impl ClarifierOffsetHz {
    pub fn to_i16(&self) -> i16 {
        self.value
    }
}

impl TryFrom<i16> for ClarifierOffsetHz {
    type Error = ();

    fn try_from(item: i16) -> Result<Self, Self::Error> {
        if item.abs() > 9_990 {
            Err(())
        } else {
            Ok(ClarifierOffsetHz { value: item })
        }
    }
}

impl TryFrom<&[u8]> for ClarifierOffsetHz {
    type Error = ();

    fn try_from(item: &[u8]) -> Result<Self, Self::Error> {
        if item.len() != 5 {
            return Err(());
        }
        let value = buf4_to_i16(item)?;
        ClarifierOffsetHz::try_from(value)
    }
}

impl fmt::Display for ClarifierOffsetHz {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:+05}", self.value,)
    }
}

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
// SqlType
//------------------------------------
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SqlType {
    CtcssOff = 0x00,
    CtcssEncDec = 0x01,
    CtcssEnc = 0x02,
    Dcs = 0x03,
    PrFreq = 0x04,
    RevTone = 0x05,
}

impl TryFrom<char> for SqlType {
    type Error = ();

    fn try_from(item: char) -> Result<Self, Self::Error> {
        match item {
            '0' => Ok(Self::CtcssOff),
            '1' => Ok(Self::CtcssEncDec),
            '2' => Ok(Self::CtcssEnc),
            '3' => Ok(Self::Dcs),
            '4' => Ok(Self::PrFreq),
            '5' => Ok(Self::RevTone),
            _ => Err(()),
        }
    }
}

impl fmt::Display for SqlType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SqlType::CtcssOff => write!(f, "CTCSS_OFF"),
            SqlType::CtcssEncDec => write!(f, "CTCSS_ENCDEC"),
            SqlType::CtcssEnc => write!(f, "CTCSS_ENC"),
            SqlType::Dcs => write!(f, "DCS"),
            SqlType::PrFreq => write!(f, "PR FREQ"),
            SqlType::RevTone => write!(f, "REV TONE"),
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
    pub frequency_hz: FrequencyHz,              // 9 positions [432100000]
    pub clarifier_offset_hz: ClarifierOffsetHz, // 5 positions [+0015]
    pub rx_clarifier_enabled: RxClarifierOnOff, // 1 position [0: OFF, 1: ON]
    pub tx_clarifier_enabled: TxClarifierOnOff, // 1 position [0: OFF, 1: ON]
    pub mode: Mode,                             // 1 positions
    pub ch_type: ChType, // 1 position [0: VFO 1: Memory Channel 2: Memory Tune 3: Quick Memory Bank (QMB) 4: - 5: PMS]
    pub sql_type: SqlType,      // 1 position [0: CTCSS “OFF” 1: CTCSS ENC/DEC 2: CTCSS ENC]
    pub shift: Shift,    // 1 position [0: Simplex 1: Plus Shift 2: Minus Shift]
}

impl Default for MemoryRead {
    fn default() -> Self {
        Self {
            channel: MemoryChannel::VfoMtQmb,
            frequency_hz: FrequencyHz { value: 0 },
            clarifier_offset_hz: ClarifierOffsetHz { value: 0 },
            rx_clarifier_enabled: RxClarifierOnOff::RxClarifierOff,
            tx_clarifier_enabled: TxClarifierOnOff::TxClarifierOff,
            mode: Mode::Lsb,
            ch_type: ChType::Vfo,
            sql_type: SqlType::CtcssOff,
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
        mr.frequency_hz = FrequencyHz::try_from(&buffer[7..16])?;
        mr.clarifier_offset_hz = ClarifierOffsetHz::try_from(&buffer[16..21])?;
        mr.rx_clarifier_enabled = RxClarifierOnOff::try_from(buffer[21] as char)?;
        mr.tx_clarifier_enabled = TxClarifierOnOff::try_from(buffer[22] as char)?;
        mr.mode = Mode::try_from(buffer[23] as char)?;
        mr.ch_type = ChType::try_from(buffer[24] as char)?;
        mr.sql_type = SqlType::try_from(buffer[25] as char)?;
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
            self.sql_type,
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

//------------------------------------
// MC - MEMORY CHANNEL
//------------------------------------
pub enum Side {
    Main = 0,
    Sub = 1,
}

impl TryFrom<&u8> for Side {
    type Error = ();

    fn try_from(item: &u8) -> Result<Self, Self::Error> {
        match item {
            0 => Ok(Side::Main),
            1 => Ok(Side::Sub),
            _ => Err(()),
        }
    }
}

pub struct McReply {
    pub side: Side,
    pub channel: MemoryChannel,
}

pub struct CmdMc<'a> {
    cmd: Cmd<'a>,
}

pub const CMD_MC: CmdMc<'static> = CmdMc { cmd: Cmd { code: &['M', 'C'], read_params: 6 } };

impl CmdMc<'_> {
    pub fn read(&self) -> Vec<u8> {
        Cmd::tx_buffer(&self.cmd, None)
    }

    pub fn set(&self, ch: MemoryChannel) -> Vec<u8> {
        let s = ch.to_chars().unwrap();
        debug!("CMD_MC::set input: {:?}", s);
        Cmd::tx_buffer(&self.cmd, Some(s.to_vec()))
    }

    pub fn decode(&self, buffer: &Vec<u8>) -> Result<McReply, ()> {
        debug!("CMD_MC::decode input: {:?}", buffer);
        Cmd::is_reply_ok(&self.cmd, buffer)?;
        let side = Side::try_from(&buffer[2]).unwrap();
        let ch: [char; 5] = [
            buffer[3] as char,
            buffer[4] as char,
            buffer[5] as char,
            buffer[6] as char,
            buffer[7] as char,
        ];
        let channel = MemoryChannel::try_from(&ch).unwrap();
        Ok(McReply { side, channel })
    }
}

//------------------------------------
// CN CTCSS TONE FREQUENCY / DCS CODE
//------------------------------------
pub enum ToneType {
    Ctcss = 0,
    Dcs = 1,
}

impl TryFrom<&u8> for ToneType {
    type Error = ();

    fn try_from(item: &u8) -> Result<Self, Self::Error> {
        match item {
            0 => Ok(ToneType::Ctcss),
            1 => Ok(ToneType::Dcs),
            _ => Err(()),
        }
    }
}

type CtcssFreq = f32;
type DcsCode = u16;
type ToneCode = u8;

const CTCSS_CODES: [CtcssFreq; 50] = [
    67.0, 69.3, 71.9, 74.4, 77.0, 79.7, 82.5, 85.4, 88.5,
    91.5, 94.8, 97.4, 100.0, 103.5, 107.2, 110.9, 114.8, 118.8,
    123.0, 127.3, 131.8, 136.5, 141.3, 146.2, 151.4, 156.7, 159.8, // 150.0
    162.2, 165.5, 167.9, 171.3, 173.8, 177.3, 179.9, 183.5, 186.2,
    189.9, 192.8, 196.6, 199.5, 203.5, 206.5, 210.7, 218.1, 225.7,
    229.1, 233.6, 241.8, 250.3, 254.1
];

const DCS_CODES: [DcsCode; 104] = [
    23, 25, 26, 31, 32, 36, 43, 47, 51, 53, 54, 65, 71, 72, 73,
    74, 114, 115, 116, 122, 125, 131, 132, 134, 143, 145, 152,
    155, 156, 162, 165, 172, 174, 205, 212, 223, 225, 226, 243,
    244, 245, 246, 251, 252, 255, 261, 263, 265, 266, 271, 274,
    306, 311, 315, 325, 331, 332, 343, 346, 351, 356, 364, 365,
    371, 411, 412, 413, 423, 431, 432, 445, 446, 452, 454, 455,
    462, 464, 465, 466, 503, 506, 516, 523, 565, 532, 546, 565,
    606, 612, 624, 627, 631, 632, 654, 662, 664, 703, 712, 723,
    731, 732, 734, 743, 754
];

pub struct CmdCn<'a> {
    cmd: Cmd<'a>,
}

pub const CMD_CN: CmdCn<'static> = CmdCn { cmd: Cmd { code: &['C', 'N'], read_params: 5 } };

pub struct CnReply {
    side: Side,
    tone_type: ToneType,
    tone_code: ToneCode,
}

impl CmdCn<'_> {
    pub fn read(&self) -> Vec<u8> {
        Cmd::tx_buffer(&self.cmd, None)
    }

    pub fn set(&self, sd: Side, tt: ToneType, cd: ToneCode) -> Vec<u8> {
        let sd = sd as u8 as char;
        let tt = tt as u8 as char;
        let s = format!("{}{}{:03}", sd, tt, cd);
        debug!("CMD_CN::set input: {:?}", s);
        Cmd::tx_buffer(&self.cmd, Some(s.chars().map(|c| c as char).collect::<Vec<char>>()))
    }

    pub fn decode(&self, buffer: &Vec<u8>) -> Result<CnReply, ()> {
        debug!("CMD_CN::decode input: {:?}", buffer);
        Cmd::is_reply_ok(&self.cmd, buffer)?;
        let side = Side::try_from(&buffer[2]).unwrap();
        let tone_type = ToneType::try_from(&buffer[3]).unwrap();
        let tone_code = buf3_to_u8(&buffer[4..7]).unwrap();
        Ok(CnReply { side, tone_type, tone_code })
    }
}

//------------------------------------
// TESTS
//------------------------------------

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

    #[test]
    fn test_frequency_hz_from_u32_valid() {
        assert!(FrequencyHz::try_from(30_000).is_ok());
        assert!(FrequencyHz::try_from(173_999_999).is_ok());
        assert!(FrequencyHz::try_from(400_000_000).is_ok());
        assert!(FrequencyHz::try_from(469_999_999).is_ok());
    }

    #[test]
    fn test_frequency_hz_from_u32_invalid() {
        assert!(FrequencyHz::try_from(29_999).is_err());
        assert!(FrequencyHz::try_from(174_000_000).is_err());
        assert!(FrequencyHz::try_from(399_999_999).is_err());
        assert!(FrequencyHz::try_from(470_000_000).is_err());
    }

    #[test]
    fn test_frequency_hz_from_bytes_valid() {
        assert!(FrequencyHz::try_from("007000000".as_bytes()).is_ok());
        assert_eq!(
            FrequencyHz::try_from("007000000".as_bytes()).unwrap().value,
            7_000_000
        );
    }

    #[test]
    fn test_frequency_hz_from_bytes_invalid() {
        assert!(FrequencyHz::try_from("00700000".as_bytes()).is_err()); // Invalid length
        assert!(FrequencyHz::try_from("000000001".as_bytes()).is_err()); // Invalid value
    }

    #[test]
    fn test_frequency_hz_display() {
        let freq = FrequencyHz { value: 7_123_456 };
        assert_eq!(format!("{}", freq), "007123456");
    }

    #[test]
    fn test_frequency_hz_from_string_valid() {
        assert!(FrequencyHz::try_from("007000000".to_string()).is_ok());
    }

    #[test]
    fn test_frequency_hz_from_string_invalid() {
        assert!(FrequencyHz::try_from("0070000000".to_string()).is_err()); // Invalid length
        assert!(FrequencyHz::try_from("invalid".to_string()).is_err()); // Not a number
    }

    #[test]
    fn test_clarifier_offset_hz_from_i16_valid() {
        assert!(ClarifierOffsetHz::try_from(0).is_ok());
        assert!(ClarifierOffsetHz::try_from(9990).is_ok());
        assert!(ClarifierOffsetHz::try_from(-9990).is_ok());
    }

    #[test]
    fn test_clarifier_offset_hz_from_i16_invalid() {
        assert!(ClarifierOffsetHz::try_from(9991).is_err());
        assert!(ClarifierOffsetHz::try_from(-9991).is_err());
    }

    #[test]
    fn test_clarifier_offset_hz_from_bytes_valid() {
        assert!(ClarifierOffsetHz::try_from("+0000".as_bytes()).is_ok());
        assert_eq!(
            ClarifierOffsetHz::try_from("+1234".as_bytes()).unwrap().value,
            1234
        );
        assert_eq!(
            ClarifierOffsetHz::try_from("-1234".as_bytes()).unwrap().value,
            -1234
        );
    }

    #[test]
    fn test_clarifier_offset_hz_from_bytes_invalid() {
        assert!(ClarifierOffsetHz::try_from("+000".as_bytes()).is_err()); // Invalid length
        assert!(ClarifierOffsetHz::try_from("-99999".as_bytes()).is_err()); // Invalid length
        assert!(ClarifierOffsetHz::try_from("?1234".as_bytes()).is_err()); // Invalid sign
    }

    #[test]
    fn test_clarifier_offset_hz_display() {
        let offset = ClarifierOffsetHz { value: 123 };
        assert_eq!(format!("{}", offset), "+0123");
        let offset = ClarifierOffsetHz { value: -123 };
        assert_eq!(format!("{}", offset), "-0123");
        let offset = ClarifierOffsetHz { value: 0 };
        assert_eq!(format!("{}", offset), "+0000");
    }
}
