//! # String Manipulation Module
//!
//! This module provides utilities for string manipulation.
//!
//! ## Examples
//!
//! ```
//! use iso8583_parser::{StringManipulation, Mode};
//!
//! let mut s = String::from("48656C6C6F2C576F726C64"); // "Hello, World" in hex
//!
//! // Convert hex to ASCII
//! let ascii_result = s.hex_to_ascii();
//! assert_eq!(ascii_result.unwrap(), "Hello,World");
//! 
//! // Get a slice of the string until a specified length
//! let slice = s.get_slice_until(5);
//! assert_eq!(slice, "48656");
//! 
//! // Get another slice of the string until a specified length
//! let slice = s.get_slice_until(5);
//! assert_eq!(slice, "C6C6F");
//! 
//!let mode_instance = Mode { enabled_private_tlv: false, enabled_private_ltv: false };
//! // Process a field based on field number, length, and name
//! s.process_field(1, 12, "test", &mode_instance);
//!
//! use iso8583_parser::positions_of_set_bits;
//!
//! let bitmap: Vec<u32> = positions_of_set_bits(u64::from_str_radix("3038058020C19201", 16).unwrap());
//! assert_eq!(bitmap, vec![3, 4, 11, 12, 13, 22, 24, 25, 35, 41, 42, 48, 49, 52, 55, 64]);
//! 
//! let mut s = String::from("1101303830303539313535301002322E362E31352E3332020330022231021532"); // LTV format in hex
//!
//! // Parse LTV (Length, Tag, Value) format
//! let ltvs = s.parse_private_ltv().unwrap();
//!
//! for ltv in ltvs {
//!     println!("{}", ltv);
//! }
//! ```

use emv_tlv_parser::parse_tlv;
use std::error;
pub mod gui; // Make the gui module public

#[derive(Debug)]
pub struct  LTV {
    pub length: usize,
    pub tag: u8,
    pub value: String,
}
pub struct  PrivateTlv {
    pub tag: String,
    pub length: usize,
    pub value: String,
}

pub struct Mode {
    pub enabled_private_tlv: bool,
    pub enabled_private_ltv: bool,
}

/// Returns the positions of set bits in a binary number.
pub fn positions_of_set_bits(n: u64) -> Vec<u32> {
    (0..64).filter(|&bit| 1 & (n >> (63 - bit)) != 0).map(|bit| bit + 1).collect()
}

/// Trait for string manipulation operations.
pub trait StringManipulation {
    /// Get a slice of the string until a specified length.
    fn get_slice_until(&mut self, length: usize) -> String;

    /// Convert a hex string to ASCII.
    fn hex_to_ascii(&mut self) -> Result<String, hex::FromHexError>;

    /// Process a field based on field number, length, and name.
    fn process_field(&mut self, field_number: u32, length: u32, name: &str, mode: &Mode);

    /// Parse LTV (Length, Tag, Value) format.
    fn parse_private_ltv(&mut self) -> Result<Vec<LTV>, Box<dyn error::Error>>;

    /// Parse Private TLV format
    fn parse_private_tlv(&mut self) -> Result<Vec<PrivateTlv>, Box<dyn error::Error>>;
}

impl StringManipulation for String {
    /// Get a slice of the string until a specified length.
    fn get_slice_until(&mut self, length: usize) -> String {
        self.drain(..length).collect::<String>()
    }

    /// Convert a hex string to ASCII.
    fn hex_to_ascii(&mut self) -> Result<String, hex::FromHexError> {
        let hex_bytes = hex::decode(self)?;
        let ascii_chars: String = hex_bytes.iter().map(|&byte| byte as char).collect();
        Ok(ascii_chars)
    }

    /// Process a field based on field number, length, and name.
    fn process_field(&mut self, field_number: u32,length: u32,name: &str, mode: &Mode) {
        let padded_length = if length % 2 == 1 {
            length + 1
        } else {
            length
        };
        let mut field_value = if field_number == 35 {
            self.get_slice_until(38 as usize)
        } else {
            self.get_slice_until(padded_length as usize)
        };
        let value_to_print = if matches!(field_number, 37 | 38 | 41 | 42 | 44 | 49 | 50 | 51 | 62 | 116 | 122) {
            field_value.hex_to_ascii().unwrap()
        } else {
            field_value.to_string()
        };

        println!("Field {:3} | Length: {:3}| {:25} | {}", field_number, length, name, value_to_print.chars().take(length as usize).collect::<String>());
        
        if field_number == 55 {
            match parse_tlv(value_to_print) {
                Ok(tags) => tags.iter().for_each(|tag| println!("{}", tag)),
                Err(e) => eprintln!("Error parsing TLV: {}", e),
            }
        }
        else if field_number == 48 || field_number == 121  {
            if mode.enabled_private_tlv {
                let mut tlv_private_value = value_to_print;
                match tlv_private_value.parse_private_tlv() {
                    Ok(tlvs_p) => tlvs_p.iter().for_each(|tlv_p| println!("{}", tlv_p)),
                    Err(e) => eprintln!("Error parsing private tlv: {:?}", e),
                }
            }
            else if mode.enabled_private_ltv {
                let mut ltv_value = value_to_print;
                match ltv_value.parse_private_ltv() {
                    Ok(ltvs) => ltvs.iter().for_each(|ltv| println!("{}", ltv)),
                    Err(e) => eprintln!("Error parsing LTV: {:?}", e),
                }
            }
        }
    }


    fn parse_private_ltv(&mut self) -> Result<Vec<LTV>, Box<dyn error::Error>> {
    let mut ltvs = Vec::new();
        while self.len() > 0 {
            let length =  self.drain(..2).collect::<String>().parse::<usize>()?;
            let tag =  self.drain(..2).collect::<String>().parse::<u8>()?;
            let byte_length  = (length - 1) * 2;
            let value = self.drain(..byte_length).collect::<String>();
            let ltv = LTV { length, tag, value};
            ltvs.push(ltv);
        }
    Ok(ltvs)
    }

    fn parse_private_tlv(&mut self) -> Result<Vec<PrivateTlv>, Box<dyn error::Error>> {
        let mut private_tlvs = Vec::new();
            while self.len() > 0 {
                let tag =  self.drain(..4).collect::<String>().hex_to_ascii().unwrap();
                let length_hex_string =  self.drain(..4).collect::<String>().hex_to_ascii().unwrap();
                let length = usize::from_str_radix(length_hex_string.as_str(), 16)?;
                let byte_length  = length * 2;
                let value = self.drain(..byte_length).collect::<String>().hex_to_ascii().unwrap();
                let private_tlv = PrivateTlv { tag, length, value};
                private_tlvs.push(private_tlv);
            }
        Ok(private_tlvs)
    }

}

use std::fmt;
impl fmt::Display for LTV {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value_string = match self.value.clone().hex_to_ascii() {
            Ok(ascii) => format!("-> {}", ascii),
            Err(e) => e.to_string(), // Handle the error case, you might want to log or handle it differently
        };
        write!(
            f,
            "\tLen: {:3} | Tag: {:3} | Val: {} {}",
            self.length,
            self.tag,
            self.value,
            value_string,
        )
    }
}

impl fmt::Display for PrivateTlv {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "\tTag: {:3} | Len: {:3} | Val: {}",
            self.tag,
            self.length,
            self.value,
        )
    }
}

#[derive(Debug)]
pub struct ParserResult {
    pub message_length: Option<u32>,
    pub header: Option<String>,
    pub mti: String,
    pub bitmap: Vec<u32>,
    pub fields: Vec<(u32, String, String)>, // (field number, name, value)
    pub unparsed: String,
}

pub fn parse_iso8583(message: &str, including_header_length: bool, tlv_private: bool, ltv_private: bool) -> Result<ParserResult, Box<dyn error::Error>> {
    let mut result = ParserResult {
        message_length: None,
        header: None,
        mti: String::new(),
        bitmap: Vec::new(),
        fields: Vec::new(),
        unparsed: String::new(),
    };

    let mut s = message.replace("\"", "").replace(" ", "");
    
    if including_header_length {
        let message_len = u32::from_str_radix(&s.get_slice_until(4), 16)? * 2;
        result.message_length = Some(message_len);
        
        if s.len() != message_len as usize {
            return Err(format!("Error: Incorrect message len. The expected length is {} but The actual is {}", message_len, s.len()).into());
        }
        result.header = Some(s.get_slice_until(10).to_string());
    }

    result.mti = s.get_slice_until(4).to_string();
    
    let mut bitmap: Vec<u32> = positions_of_set_bits(u64::from_str_radix(&s.get_slice_until(16), 16)?);
    if bitmap.contains(&1) {
        let mut positions = positions_of_set_bits(u64::from_str_radix(&s.get_slice_until(16), 16)?);
        positions.iter_mut().for_each(|num| *num += 64);
        bitmap.append(&mut positions);
        bitmap.retain(|&x| x != 1);
    }
    result.bitmap = bitmap;

    let mode = Mode {
        enabled_private_tlv: tlv_private,
        enabled_private_ltv: ltv_private,
    };

    for &bit in &result.bitmap {
        match bit {
            2 => {
                let pan_len: u32 =  s.get_slice_until(2).parse::<u32>().unwrap();
                s.process_field(2, pan_len, "PAN", &mode);
            }
            3 => s.process_field(3, 6, "Process Code", &mode),
            4 => s.process_field(4, 12, "Transaction Amount", &mode),
            5 => s.process_field(5, 12, "Settlement Amount", &mode),
            6 => s.process_field(6, 12, "Cardholder Billing Amount", &mode),
            7 => s.process_field(7, 10, "Transaction Date and Time", &mode),
            9 => s.process_field(9, 8, "Conversion rate, settlement", &mode),
            10 => s.process_field(10, 8, "Conversion rate, cardholder billing", &mode),
            11 => s.process_field(11, 6, "Trace", &mode),
            12 => s.process_field(12, 6, "Time", &mode),
            13 => s.process_field(13, 4, "Date", &mode),
            14 => s.process_field(14, 4, "Card EXpiration Date", &mode),
            15 => s.process_field(15, 4, "Settlement Date", &mode),
            18 => s.process_field(18, 4, "Merchant Category Code", &mode),
            19 => s.process_field(19, 3, "Acquirer Country Code", &mode),
            22 => s.process_field(22, 4, "POS Entry Mode", &mode),
            23 => s.process_field(23, 3, "Card Sequence Number", &mode),
            24 => s.process_field(24, 4, "", &mode),
            25 => s.process_field(25, 2, "", &mode),
            32 => {
                let field32_len: u32 = s.get_slice_until(2).parse::<u32>().unwrap();
                s.process_field(32, field32_len, "Institution Identification Code Acquiring", &mode);
            }
            35 => {
                let track2_len: u32 = s.get_slice_until(2).parse::<u32>().unwrap() *2;
                s.process_field(35, track2_len, "Track2", &mode);
            }
            37 => s.process_field(37, 24, "Retrieval Ref #", &mode),
            38 => s.process_field(38, 12, "Authorization Code", &mode),
            39 => s.process_field(39, 4, "Response Code", &mode),
            41 => s.process_field(41, 16, "Terminal", &mode),
            42 => s.process_field(42, 30, "Acceptor", &mode),
            43 => s.process_field(43, 40, "Card Acceptor Name/Location", &mode),
            44 => {
                let field44_len: u32 = s.get_slice_until(2).parse::<u32>().unwrap() * 2;
                s.process_field(44, field44_len, "Additional response data", &mode);
            }
            45 => {
                let track1_len: u32 = s.get_slice_until(2).parse::<u32>().unwrap();
                s.process_field(45, track1_len, "Track 1 Data", &mode);
            }
            48 => {
                let field48_len = s.get_slice_until(4).parse::<u32>().unwrap() * 2;
                s.process_field(48, field48_len, "Aditional Data", &mode);
            }
            49 => s.process_field(49, 6, "Transaction Currency Code", &mode),
            50 => s.process_field(50, 6, "Settlement Currency Code", &mode),
            51 => s.process_field(51, 6, "Billing Currency Code", &mode),
            52 => s.process_field(52, 16, "PinBlock", &mode),
            54 => {
                let field54_len = s.get_slice_until(4).parse::<u32>().unwrap() * 2;
                s.process_field(54, field54_len, "Amount", &mode);
            }
            55 => {
                let field55_len = s.get_slice_until(4).parse::<u32>().unwrap() * 2;
                s.process_field(55, field55_len, "", &mode);
            }
            60 => {
                let field60_len = s.get_slice_until(4).parse::<u32>().unwrap() * 2;
                s.process_field(60, field60_len, "", &mode);              
            }
            62 => {
                let field62_len = s.get_slice_until(4).parse::<u32>().unwrap() * 2;
                s.process_field(62, field62_len, "Private", &mode);
            }
            64 => s.process_field(64, 16, "MAC", &mode),
            70 => s.process_field(70, 4, "", &mode),
            116 => {
                let field116_len = s.get_slice_until(4).parse::<u32>().unwrap() * 2;
                s.process_field(116, field116_len, "", &mode);
            }
            121 => {
                let field121_len = s.get_slice_until(4).parse::<u32>().unwrap() * 2;
                s.process_field(121, field121_len, "Additional Data", &mode);
            }
            122 => {
                let field122_len = s.get_slice_until(4).parse::<u32>().unwrap() * 2;
                s.process_field(122, field122_len, "Additional Data", &mode);
            }
            128 => s.process_field(128, 16, "MAC", &mode),
            _ => return Err(format!("Field {} is not implemented", bit).into()),
        }
     }

    result.unparsed = s;
    Ok(result)
}

#[cfg(test)]
mod tests {
      use crate::StringManipulation;
    #[test]
    fn test_parse_ltv_single() {
        let mut s = String::from("061148656C6C6F");
        let mut ltvs = s.parse_private_ltv().unwrap();

        assert_eq!(ltvs.len(), 1);

        let ltv = &mut ltvs[0];
        assert_eq!(ltv.length, 6);
        assert_eq!(ltv.tag, 11);
        assert_eq!(ltv.value.hex_to_ascii().unwrap(), "Hello");
    }

    #[test]
    fn test_parse_ltv_multiple() {
        let mut s = String::from("031148690622576F726C64");
        let mut ltvs = s.parse_private_ltv().unwrap();

        assert_eq!(ltvs.len(), 2);

        let ltv1 = &mut ltvs[0];
        assert_eq!(ltv1.length, 3);
        assert_eq!(ltv1.tag, 11);
        assert_eq!(ltv1.value.hex_to_ascii().unwrap(), "Hi");

        let ltv2 = &mut ltvs[1];
        assert_eq!(ltv2.length, 6);
        assert_eq!(ltv2.tag, 22);
        assert_eq!(ltv2.value.hex_to_ascii().unwrap(), "World");
    }

    #[test]
    fn test_parse_ltv_empty() {
        let mut s = String::new();
        let ltvs = s.parse_private_ltv();

        assert!(ltvs.is_ok());
        assert!(ltvs.unwrap().is_empty());
    }

    #[test]
    fn error_test() {
        let mut s = String::from("T31148690622576F726C64");
        let ltvs = s.parse_private_ltv();
        assert!(ltvs.is_err());
        assert_eq!(ltvs.err().unwrap().to_string().as_str(), "invalid digit found in string");
    }

}
