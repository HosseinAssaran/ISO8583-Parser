use iso8583_parser::{StringManipulation, positions_of_set_bits, Mode};
use clap::Parser;

fn read_data_from_stdin()-> String {
    use std::io::{stdin,stdout,Write};
    let mut data_raw =String::new();
    print!("Please enter a message to parse: ");
    let _=stdout().flush();
    stdin().read_line(&mut data_raw).expect("Did not enter a correct string");
    if let Some('\n')=data_raw.chars().next_back() {
        data_raw.pop();
    }
    if let Some('\r')=data_raw.chars().next_back() {
        data_raw.pop();
    }
    data_raw
}

/// Arguments
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// message to get
    #[arg(short, long, required = false)]
    message: Option<String>,

    #[arg(short, long)]
    including_header_length: bool,

    #[arg(short, long)]
    tlv_private: bool,

    #[arg(short, long)]
    ltv_private: bool,
}

fn main() {
    // Get command-line arguments
    let args = Args::parse();

    // Check if message argument is provided unless read data from stdin
    let mut s = match args.message {
        Some(m) => m,
        None => read_data_from_stdin(), 
    };
    s = s.replace("\"", "").replace(" ", "");
    if args.including_header_length {
        let message_len = u32::from_str_radix(&s.get_slice_until(4), 16).expect("Unable to get the length") * 2;
        println!("Lentgh Of Message: {}", message_len );
        if s.len() != message_len as usize {
            panic!("Error: Incorrect message len. The expected length is {} but The actual is {}", message_len,  s.len());
        }
        println!("Header: {}", s.get_slice_until(10));
    }
    println!("MTI: {}", s.get_slice_until(4));
    let mut bitmap: Vec<u32> = positions_of_set_bits(u64::from_str_radix(&s.get_slice_until(16),16).expect("Unable to get the process code"));
    if bitmap.contains(&1) {
        let mut positions = positions_of_set_bits(u64::from_str_radix(&s.get_slice_until(16), 16).expect("Unable to get the process code"));
        positions.iter_mut().for_each(|num| *num += 64);
        bitmap.append(&mut positions);
        bitmap.retain(|&x| x != 1);
    }
    println!("First Bit Map: {:?}", bitmap);
    let mode  = Mode { 
        enabled_private_tlv: args.tlv_private,
        enabled_private_ltv: args.ltv_private,
     };
        
     for &bit in &bitmap {
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
            122 => {
                let field122_len = s.get_slice_until(4).parse::<u32>().unwrap() * 2;
                s.process_field(122, field122_len, "Additional Data", &mode);
            }
            128 => s.process_field(128, 16, "MAC", &mode),
            num => {println!("The number {} is not defined yet.", num); return;},
        }
    }
    if !s.is_empty() {
        println!("Not parsed Part: {}",s);
    }
}

