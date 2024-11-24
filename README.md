# ISO8583 parser 
![Crates.io](https://img.shields.io/crates/v/iso8583_parser?style=flat-square)
![Crates.io](https://img.shields.io/crates/d/iso8583_parser?style=flat-square)
![build workflow](https://github.com/HosseinAssaran/ISO8583-Parser/actions/workflows/rust.yml/badge.svg)
![release workflow](https://github.com/HosseinAssaran/ISO8583-Parser/actions/workflows/release.yml/badge.svg)

This Rust program parses ISO8583 messages in hex string format and extracts specific fields. It provides multiple interfaces including a GUI, CLI, and a library for integration into other projects.

## Features

- Parse ISO8583 messages with or without length headers
- Support for private TLV (Tag-Length-Value) parsing
- Support for private LTV (Length-Tag-Value) parsing
- Multiple interfaces:
  - Graphical User Interface (GUI)
  - Command Line Interface (CLI)
  - PHP Web Server Interface
  - Library for integration
- Detailed field parsing with field names and descriptions

## Usage

### GUI Application

1. Run the GUI version using:
```bash
cargo run --bin iso8583_parser_gui
```

### Command Line Interface (CLI)

1. Clone the repository:
```bash
git clone https://github.com/HosseinAssaran/ISO8583-Parser
cd ISO8583-Parser
```

2. Run with command line arguments:
```bash
cargo run -- --message <hex-message> [--including-header-length] [--tlv-private] [--ltv-private]
```

Or run without arguments to use interactive mode:
```bash
cargo run
```

### Run it as a PHP Web Server
1. Download the source code and go to the root directory of your source code
2. Run below command inside **PowerShell**:
   ```
    .\iso_parser_downloader.bat
   ```
3. Run PHP Web Server using below command:
   ```
   php -S localhost:12345
   ```
4. Open your browser and go to the link below:
   ```
   localhost:12345
   ```

**Important Note:** As the PHP Web server uses a rust program to parse the message, you will need it. You can achieve this program by building release of the rust written program from the source or you can downlaod the executable file with **iso_parser_downloader**.

### Library Usage

1. Add the dependency to your `Cargo.toml`:
```toml
[dependencies]
iso8583_parser = "0.1.12"
```

2. Use in your code:
```rust
use iso8583_parser::{parse_iso8583, StringManipulation};

fn main() {
    let message = "0100..."; // Your ISO8583 message in hex
    let result = parse_iso8583(
        message,
        false, // including_header_length
        false, // tlv_private
        false  // ltv_private
    );

    match result {
        Ok(parsed) => {
            println!("MTI: {}", parsed.mti);
            println!("Bitmap: {:?}", parsed.bitmap);
            // ... process other fields
        },
        Err(e) => println!("Error parsing message: {}", e),
    }
}
```

## Testing

Run the test suite:
```bash
cargo test
```

## Building for Release

Build optimized binaries:
```bash
cargo build --release
```

This will create optimized executables in the `target/release` directory:
- `iso8583_parser` - CLI application
- `iso8583_parser_gui` - GUI application

## License

Licensed under either of:
- MIT license
- Apache License, Version 2.0

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.