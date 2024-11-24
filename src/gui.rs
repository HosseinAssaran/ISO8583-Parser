use eframe::egui;
use crate::parse_iso8583;

pub struct ISO8583ParserApp {
    message: String,
    include_length_header: bool,
    parse_private_tlv: bool,
    parse_private_ltv: bool,
    parsed_output: String,
    has_error: bool,
}

impl Default for ISO8583ParserApp {
    fn default() -> Self {
        Self {
            message: String::new(),
            include_length_header: false,
            parse_private_tlv: false,
            parse_private_ltv: false,
            parsed_output: String::new(),
            has_error: false,
        }
    }
}

impl eframe::App for ISO8583ParserApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("ISO8583 Message Parser");
            ui.add_space(10.0);

            ui.checkbox(&mut self.include_length_header, "The message includes length and header");
            ui.checkbox(&mut self.parse_private_tlv, "Parse Private TLV");
            ui.checkbox(&mut self.parse_private_ltv, "Parse Private LTV");
            ui.add_space(10.0);

            ui.label("Enter the message:");
            ui.label(egui::RichText::new("(e.g. '01002000000000000000930000')").color(egui::Color32::GRAY));
            ui.label(egui::RichText::new("(e.g. '0012600008000001002000000000000000930000' while including header and length)")
                .color(egui::Color32::GRAY));
            
            ui.add_space(5.0);
            
            ui.add_sized(
                [ui.available_width(), 150.0],
                egui::TextEdit::multiline(&mut self.message)
                    .desired_rows(10)
                    .hint_text("Enter ISO8583 message here"),
            );

            if ui.button("Parse Message").clicked() && !self.message.is_empty() {
                self.parse_message();
            }

            ui.add_space(20.0);

            if !self.parsed_output.is_empty() {
                ui.heading("Parsed Message:");
                ui.add_space(5.0);
                
                let output_text = if self.has_error {
                    egui::RichText::new(&self.parsed_output).color(egui::Color32::RED)
                } else {
                    egui::RichText::new(&self.parsed_output).monospace()
                };
                
                egui::ScrollArea::vertical()
                .max_height(200.0) // You can adjust this value based on your needs
                .show(ui, |ui| {
                    ui.add(egui::Label::new(output_text).wrap());
            });
            }
        });
    }
}

impl ISO8583ParserApp {
    fn parse_message(&mut self) {
        match parse_iso8583(
            &self.message,
            self.include_length_header,
            self.parse_private_tlv,
            self.parse_private_ltv,
        ) {
            Ok(result) => {
                let mut output = String::new();
                
                if let Some(len) = result.message_length {
                    output.push_str(&format!("Length Of Message: {}\n", len));
                }
                
                if let Some(header) = result.header {
                    output.push_str(&format!("Header: {}\n", header));
                }
                
                output.push_str(&format!("MTI: {}\n", result.mti));
                output.push_str(&format!("First Bit Map: {:?}\n\n", result.bitmap));
                
                for field in result.fields {
                    output.push_str(&format!("{}\n", field));
                }
                
                if !result.unparsed.is_empty() {
                    output.push_str(&format!("\nNot parsed Part: {}", result.unparsed));
                }
                
                self.has_error = false;
                self.parsed_output = output;
            }
            Err(e) => {
                self.has_error = true;
                self.parsed_output = format!("Error parsing message: {}", e);
            }
        }
    }
}
