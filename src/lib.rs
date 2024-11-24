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
    fn process_field(&mut self, field_number: u32, length: u32, name: &str, mode: &Mode) -> String;

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
    fn process_field(&mut self, field_number: u32, length: u32, name: &str, mode: &Mode) -> String {
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
    
        let mut result = format!(
            "Field {:3} | Length: {:3}| {:25} | {}\n",
            field_number,
            length,
            name,
            value_to_print.chars().take(length as usize).collect::<String>()
        );
    
        if field_number == 55 {
            match parse_tlv(value_to_print) {
                Ok(tags) => {
                    for tag in tags {
                        result.push_str(&format!("{}\n", tag));
                    }
                }
                Err(e) => result.push_str(&format!("Error parsing TLV: {}\n", e)),
            }
        } else if field_number == 48 || field_number == 121 {
            if mode.enabled_private_tlv {
                let mut tlv_private_value = value_to_print;
                match tlv_private_value.parse_private_tlv() {
                    Ok(tlvs_p) => {
                        for tlv_p in tlvs_p {
                            result.push_str(&format!("{}\n", tlv_p));
                        }
                    }
                    Err(e) => result.push_str(&format!("Error parsing private tlv: {:?}\n", e)),
                }
            } else if mode.enabled_private_ltv {
                let mut ltv_value = value_to_print;
                match ltv_value.parse_private_ltv() {
                    Ok(ltvs) => {
                        for ltv in ltvs {
                            result.push_str(&format!("{}\n", ltv));
                        }
                    }
                    Err(e) => result.push_str(&format!("Error parsing LTV: {:?}\n", e)),
                }
            }
        }
    
        result
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
    pub fields: Vec<String>,  // Added this field to store the processed fields
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
        let field_info = match bit {
            2 => {
                let pan_len: u32 = s.get_slice_until(2).parse::<u32>().unwrap();
                Some((bit, pan_len, "PAN"))
            }
            3 => Some((bit, 6, "Process Code")),
            4 => Some((bit, 12, "Transaction Amount")),
            5 => Some((bit, 12, "Settlement Amount")),
            6 => Some((bit, 12, "Cardholder Billing Amount")),
            7 => Some((bit, 10, "Transaction Date and Time")),
            9 => Some((bit, 8, "Conversion rate, settlement")),
            10 => Some((bit, 8, "Conversion rate, cardholder billing")),
            11 => Some((bit, 6, "Trace")),
            12 => Some((bit, 6, "Time")),
            13 => Some((bit, 4, "Date")),
            14 => Some((bit, 4, "Card EXpiration Date")),
            15 => Some((bit, 4, "Settlement Date")),
            18 => Some((bit, 4, "Merchant Category Code")),
            19 => Some((bit, 3, "Acquirer Country Code")),
            22 => Some((bit, 4, "POS Entry Mode")),
            23 => Some((bit, 3, "Card Sequence Number")),
            24 => Some((bit, 4, "")),
            25 => Some((bit, 2, "")),
            32 => {
                let field32_len: u32 = s.get_slice_until(2).parse::<u32>().unwrap();
                Some((bit, field32_len, "Institution Identification Code Acquiring"))
            }
            35 => {
                let track2_len: u32 = s.get_slice_until(2).parse::<u32>().unwrap() * 2;
                Some((bit, track2_len, "Track2"))
            }
            37 => Some((bit, 24, "Retrieval Ref #")),
            38 => Some((bit, 12, "Authorization Code")),
            39 => Some((bit, 4, "Response Code")),
            41 => Some((bit, 16, "Terminal")),
            42 => Some((bit, 30, "Acceptor")),
            43 => Some((bit, 40, "Card Acceptor Name/Location")),
            44 => {
                let field44_len: u32 = s.get_slice_until(2).parse::<u32>().unwrap() * 2;
                Some((bit, field44_len, "Additional response data"))
            }
            45 => {
                let track1_len: u32 = s.get_slice_until(2).parse::<u32>().unwrap();
                Some((bit, track1_len, "Track 1 Data"))
            }
            48 => {
                let field48_len = s.get_slice_until(4).parse::<u32>().unwrap() * 2;
                Some((bit, field48_len, "Aditional Data"))
            }
            49 => Some((bit, 6, "Transaction Currency Code")),
            50 => Some((bit, 6, "Settlement Currency Code")),
            51 => Some((bit, 6, "Billing Currency Code")),
            52 => Some((bit, 16, "PinBlock")),
            54 => {
                let field54_len = s.get_slice_until(4).parse::<u32>().unwrap() * 2;
                Some((bit, field54_len, "Amount"))
            }
            55 => {
                let field55_len = s.get_slice_until(4).parse::<u32>().unwrap() * 2;
                Some((bit, field55_len, ""))
            }
            60 => {
                let field60_len = s.get_slice_until(4).parse::<u32>().unwrap() * 2;
                Some((bit, field60_len, ""))
            }
            62 => {
                let field62_len = s.get_slice_until(4).parse::<u32>().unwrap() * 2;
                Some((bit, field62_len, "Private"))
            }
            64 => Some((bit, 16, "MAC")),
            70 => Some((bit, 4, "")),
            116 => {
                let field116_len = s.get_slice_until(4).parse::<u32>().unwrap() * 2;
                Some((bit, field116_len, ""))
            }
            121 => {
                let field121_len = s.get_slice_until(4).parse::<u32>().unwrap() * 2;
                Some((bit, field121_len, "Additional Data"))
            }
            122 => {
                let field122_len = s.get_slice_until(4).parse::<u32>().unwrap() * 2;
                Some((bit, field122_len, "Additional Data"))
            }
            128 => Some((bit, 16, "MAC")),
            _ => return Err(format!("Field {} is not implemented", bit).into()),
        };

        if let Some((field_number, length, name)) = field_info {
            let field_data = s.process_field(field_number, length, name, &mode);
            result.fields.push(field_data);
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
