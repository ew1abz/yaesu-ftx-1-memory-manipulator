/// Small parsing helpers for fixed-width ASCII numeric fields used by the FTX1 protocol.
pub fn buf4_to_u16(buffer: &[u8]) -> Result<u16, ()> {
    if buffer.len() != 4 {
        return Err(());
    }
    let mut result: u16 = 0;
    for (i, item) in buffer.iter().enumerate().take(4) {
        if let Some(n) = (*item as char).to_digit(10) {
            result += n as u16 * (10u16.pow(3 - i as u32));
        } else {
            return Err(());
        }
    }
    Ok(result)
}

pub fn buf9_to_u32(buffer: &[u8]) -> Result<u32, ()> {
    if buffer.len() != 9 {
        return Err(());
    }
    let mut result: u32 = 0;
    for (i, item) in buffer.iter().enumerate().take(9) {
        if let Some(n) = (*item as char).to_digit(10) {
            result += n as u32 * (10u32.pow(8 - i as u32));
        } else {
            return Err(());
        }
    }
    Ok(result)
}

pub fn buf4_to_i16(buffer: &[u8]) -> Result<i16, ()> {
    // expected format: sign ("+" or "-") followed by 4 digits => total length 5
    if buffer.len() != 5 {
        return Err(());
    }
    let mut result: i16 = 0;
    let sign: i16 = if buffer[0] == b'-' { -1 } else { 1 };
    for (i, item) in buffer[1..].iter().enumerate().take(4) {
        if let Some(n) = (*item as char).to_digit(10) {
            result += n as i16 * (10i16.pow(3 - i as u32));
        } else {
            return Err(());
        }
    }
    Ok(result * sign)
}

pub fn buf5_to_i16(buffer: &[u8]) -> Result<i16, ()> {
    // expected format: sign ("+" or "-") followed by 5 digits => total length 6
    if buffer.len() != 6 {
        return Err(());
    }
    let mut result: i32 = 0;
    let sign: i32 = if buffer[0] == b'-' { -1 } else { 1 };
    for (i, item) in buffer[1..].iter().enumerate().take(5) {
        if let Some(n) = (*item as char).to_digit(10) {
            result += n as i32 * (10i32.pow(4 - i as u32));
        } else {
            return Err(());
        }
    }
    let signed = result * sign;
    if signed < i16::MIN as i32 || signed > i16::MAX as i32 {
        return Err(());
    }
    Ok(signed as i16)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buf5_parses_positive_and_negative() {
        assert_eq!(buf5_to_i16(b"+00015").unwrap(), 15);
        assert_eq!(buf5_to_i16(b"-00123").unwrap(), -123);
    }

    #[test]
    fn buf5_bounds_and_invalid() {
        assert_eq!(buf5_to_i16(b"+32767").unwrap(), 32767);
        assert_eq!(buf5_to_i16(b"-32768").unwrap(), -32768);
        assert!(buf5_to_i16(b"+00a15").is_err());
    }

    #[test]
    fn buf5_rejects_wrong_length() {
        // missing sign or digits
        assert!(buf5_to_i16(b"00015").is_err());
        assert!(buf5_to_i16(b"+0015").is_err());
    }

    #[test]
    fn buf9_parses_and_invalid() {
        assert_eq!(buf9_to_u32(b"000000000").unwrap(), 0);
        assert_eq!(buf9_to_u32(b"000000123").unwrap(), 123);
        assert_eq!(buf9_to_u32(b"999999999").unwrap(), 999_999_999u32);
        assert!(buf9_to_u32(b"00000a123").is_err());
    }

    #[test]
    fn buf9_rejects_wrong_length() {
        assert!(buf9_to_u32(b"0000123").is_err()); // 7 bytes
        assert!(buf9_to_u32(b"00001234567").is_err()); // 11 bytes
    }

    #[test]
    fn buf4_parses_and_length() {
        assert_eq!(buf4_to_u16(b"1234").unwrap(), 1234);
        assert!(buf4_to_u16(b"123").is_err());
        assert!(buf4_to_u16(b"12345").is_err());
    }

    #[test]
    fn buf4_to_i16_parses_first_four_digits() {
        // function currently expects 5 bytes with the first byte being a sign
        assert_eq!(buf4_to_i16(b"+1234").unwrap(), 1234);
        assert_eq!(buf4_to_i16(b"+9999").unwrap(), 9999);
    }

    #[test]
    fn buf4_to_i16_parses_sign() {
        // function currently expects 5 bytes with the first byte being a sign
        assert_eq!(buf4_to_i16(b"-1234").unwrap(), -1234);
        assert_eq!(buf4_to_i16(b"-9999").unwrap(), -9999);
    }

    #[test]
    fn buf4_to_i16_invalid_and_length() {
        // invalid digit present
        assert!(buf4_to_i16(b"12a45").is_err());
        // wrong lengths
        assert!(buf4_to_i16(b"1234").is_err()); // 4 bytes -> rejected
        assert!(buf4_to_i16(b"123456").is_err()); // 6 bytes -> rejected
    }
}
