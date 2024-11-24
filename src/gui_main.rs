use eframe::egui;
use crate::egui::ViewportBuilder;
use iso8583_parser::gui::ISO8583ParserApp;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size([800.0, 600.0]),
        ..Default::default()
    };
    
    eframe::run_native(
        "ISO8583 Message Parser",
        options,
        Box::new(|_cc| Ok(Box::new(ISO8583ParserApp::default()))),
    )
}