use clap::Parser;
use iso8583_parser::parse_iso8583;
use std::io::{stdin, stdout, Write};

fn read_data_from_stdin() -> String {
    let mut data_raw = String::new();
    print!("Please enter a message to parse: ");
    let _ = stdout().flush();
    stdin().read_line(&mut data_raw).expect("Did not enter a correct string");
    if let Some('\n') = data_raw.chars().next_back() {
        data_raw.pop();
    }
    if let Some('\r') = data_raw.chars().next_back() {
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
    let message = match args.message {
        Some(m) => m,
        None => read_data_from_stdin(),
    };

    // Parse the message using the shared library function
    match parse_iso8583(
        &message,
        args.including_header_length,
        args.tlv_private,
        args.ltv_private,
    ) {
        Ok(result) => {
            // Print length and header if available
            if let Some(len) = result.message_length {
                println!("Length Of Message: {}", len);
            }
            
            if let Some(header) = result.header {
                println!("Header: {}", header);
            }
            
            // Print MTI and bitmap
            println!("MTI: {}", result.mti);
            println!("First Bit Map: {:?}", result.bitmap);
            
            // Print all parsed fields
            for field in result.fields {
                println!("{}", field);
            }
            
            // Print any unparsed data
            if !result.unparsed.is_empty() {
                println!("Not parsed Part: {}", result.unparsed);
            }
        }
        Err(e) => {
            eprintln!("Error parsing message: {}", e);
            std::process::exit(1);
        }
    }
}