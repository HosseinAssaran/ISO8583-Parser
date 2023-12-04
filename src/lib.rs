//! # String Manipulation Module
//!
//! This module provides utilities for string manipulation.
//!
//! ## Examples
//!
//! ```
//! use emv_parser::string_manipulation::StringManipulation;
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
//! // Process a field based on field number, length, and name
//! s.process_field(1, 12, "test");
//!
//! use emv_parser::string_manipulation::positions_of_set_bits;
//!
//! let bitmap: Vec<u32> = positions_of_set_bits(u64::from_str_radix("3038058020C19201", 16).unwrap());
//! assert_eq!(bitmap, vec![3, 4, 11, 12, 13, 22, 24, 25, 35, 41, 42, 48, 49, 52, 55, 64]);
//! 
//! let mut s = String::from("1101303830303539313535301002322E362E31352E3332020330022231021532"); // LTV format in hex
//!
//! // Parse LTV (Length, Tag, Value) format
//! let ltvs = s.parse_ltv().unwrap();
//!
//! for ltv in ltvs {
//!     println!("{}", ltv);
//! }
//! ```

pub mod string_manipulation {
    use emv_tlv_parser::parse_tlv;
    use crate::ComplexError;

    #[derive(Debug)]
    pub struct  LTV {
        pub length: usize,
        pub tag: u8,
        pub value: String,
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
        fn process_field(&mut self, field_number: u32, length: u32, name: &str);

        /// Parse LTV (Length, Tag, Value) format.
        fn parse_ltv(&mut self) -> Result<Vec<LTV>, ComplexError>;
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
        fn process_field(&mut self, field_number: u32, length: u32, name: &str) {
            let mut field_value = if field_number == 35 {
                self.get_slice_until(38 as usize)
            } else {
                self.get_slice_until(length as usize)
            };

            let value_to_print = if matches!(field_number, 41 | 42 | 49 | 62) {
                field_value.hex_to_ascii().unwrap()
            } else {
                field_value.to_string()
            };

            println!("Field {:3} | {:12} | Length: {:3} | {}", field_number, name, length, value_to_print);
            
            if field_number == 55 {
                match parse_tlv(value_to_print) {
                    Ok(tags) => tags.iter().for_each(|tag| println!("{}", tag)),
                    Err(e) => eprintln!("Error parsing TLV: {}", e),
                }
            }
            else if field_number == 48 {
                let mut ltv_value = value_to_print;
                match ltv_value.parse_ltv() {
                    Ok(ltvs) => ltvs.iter().for_each(|ltv| println!("{}", ltv)),
                    Err(e) => eprintln!("Error parsing LTV: {:?}", e),
                }
            }
        }
    
        fn parse_ltv(&mut self) -> Result<Vec<LTV>, ComplexError> {
            let mut ltvs = Vec::new();
                while self.len() > 0 {
                    let length =  self.drain(..2).collect::<String>().parse::<usize>()?;
                    let tag =  self.drain(..2).collect::<String>().parse::<u8>()?;
                    let byte_length  = (length - 1) * 2;
                    let value = self.drain(..byte_length).collect::<String>().hex_to_ascii()?;
                    let ltv = LTV { length, tag, value};
                    ltvs.push(ltv);
                }
            Ok(ltvs)
        }
    }
}

use std::{fmt, num::ParseIntError};
use hex::FromHexError;
impl fmt::Display for string_manipulation::LTV {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "\tlength: {:3} | tag: {:3} | value: {}",
            self.length,
            self.tag,
            self.value,
        )
    }
}

#[derive(Debug, PartialEq)]
pub enum ComplexError {
    ParseInt(ParseIntError),
    FromHex(FromHexError),
}

impl From<ParseIntError> for ComplexError {
    fn from(err: ParseIntError) -> Self {
        ComplexError::ParseInt(err)
    }
}

impl From<FromHexError> for ComplexError {
    fn from(err: FromHexError) -> Self {
        ComplexError::FromHex(err)
    }
}

impl fmt::Display for ComplexError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ComplexError::FromHex(..) =>
                write!(f, "please use a vector with right hex string"),
            ComplexError::ParseInt(..) =>
                write!(f, "the provided string could not be parsed as int"),
        }
    }
}

use std::error;

impl error::Error for ComplexError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            ComplexError::FromHex(ref e) => Some(e),
            ComplexError::ParseInt(ref e) => Some(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::string_manipulation::StringManipulation;
    #[test]
    fn test_parse_ltv_single() {
        let mut s = String::from("061148656C6C6F");
        let ltvs = s.parse_ltv().unwrap();

        assert_eq!(ltvs.len(), 1);

        let ltv = &ltvs[0];
        assert_eq!(ltv.length, 6);
        assert_eq!(ltv.tag, 11);
        assert_eq!(ltv.value, "Hello".to_string());
    }

    #[test]
    fn test_parse_ltv_multiple() {
        let mut s = String::from("031148690622576F726C64");
        let ltvs = s.parse_ltv().unwrap();

        assert_eq!(ltvs.len(), 2);

        let ltv1 = &ltvs[0];
        assert_eq!(ltv1.length, 3);
        assert_eq!(ltv1.tag, 11);
        assert_eq!(ltv1.value, "Hi".to_string());

        let ltv2 = &ltvs[1];
        assert_eq!(ltv2.length, 6);
        assert_eq!(ltv2.tag, 22);
        assert_eq!(ltv2.value, "World".to_string());
    }

    #[test]
    fn test_parse_ltv_empty() {
        let mut s = String::new();
        let ltvs = s.parse_ltv();

        assert!(ltvs.is_ok());
        assert!(ltvs.unwrap().is_empty());
    }

    use crate::ComplexError;
    use hex::FromHexError;
    use std::error::Error;
    #[test]
    fn error_test() {
        let mut s = String::from("T31148690622576F726C64");
        let ltvs = s.parse_ltv();
        match ltvs {
            Err(e) => {
                println!("Error: {}", e);
                if let Some(source) = e.source() {
                    println!("  Caused by: {}", source);
                }
                  assert!(matches!(e, ComplexError::ParseInt(..)));
            }
            _ => panic!("Expected an error but got a result"),
        }
        let mut s = String::from("03114Y690622576F726C64");
        let ltvs = s.parse_ltv();
        match ltvs {
            Err(e) => {
                println!("Error: {}", e);
                if let Some(source) = e.source() {
                    println!("  Caused by: {}", source);
                }
                assert_eq!(e, ComplexError::FromHex(FromHexError::InvalidHexCharacter{c:'Y', index:1}));
            }
            _ => panic!("Expected an error but got a result"),
        }
    }

    // Add more test cases as needed
}
