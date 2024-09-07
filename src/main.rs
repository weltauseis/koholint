use std::sync::{Arc, Mutex};

use debugger::Debugger;
use gameboy::Gameboy;
use pollster::FutureExt;

#[allow(dead_code)]
mod cpu;
mod debugger;
#[allow(dead_code)]
mod decoding;
mod gameboy;
#[allow(non_contiguous_range_endpoints)]
mod memory;
#[allow(dead_code)]
mod renderer;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("Usage : gbemulator <rom file>");
        return;
    }
    env_logger::init();

    let rom = std::fs::read(&args[1]).unwrap();
    let console = Arc::new(Mutex::new(Gameboy::new(rom)));
    let mut debugger = Debugger::new(console.clone());
    std::thread::spawn(move || loop {
        debugger.prompt_command();
    });

    let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();
    glfw.window_hint(glfw::WindowHint::ClientApi(glfw::ClientApiHint::NoApi));

    let (mut window, events) = glfw
        .create_window(
            600,
            600,
            "Koholint Gameboy Emulator",
            glfw::WindowMode::Windowed,
        )
        .expect("Failed to create GLFW window.");

    window.set_key_polling(true);
    //window.make_current();

    let mut state = renderer::RendererState::new(&mut window, console.clone()).block_on();

    while !state.window().should_close() {
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            state.handle_window_event(event);
        }

        state.render().unwrap();
    }
}
