use std::{
    error::Error,
    sync::{Arc, Mutex},
};

use debugger::debug_console;
use gameboy::Gameboy;
use log::info;
use winit::{
    application::ApplicationHandler, event::WindowEvent, event_loop::EventLoop, window::Window,
};

mod app;
#[allow(dead_code)]
mod cpu;
mod debugger;
#[allow(dead_code)]
mod decoding;
mod gameboy;
#[allow(non_contiguous_range_endpoints)]
mod memory;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("Usage : gbemulator <rom file>");
        return;
    }
    env_logger::init();

    let rom = std::fs::read(&args[1]).unwrap();
    let console = Gameboy::new(rom);

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_min_inner_size([300.0, 220.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Koholint Gameboy Emulator",
        native_options,
        Box::new(|cc| Ok(Box::new(app::EmulatorApp::new(cc, console)))),
    )
    .unwrap();
}
