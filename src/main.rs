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

    let flag_paused = args.iter().any(|a| a.eq("-p"));

    let mut debugger = Debugger::new(console.clone(), flag_paused);
    std::thread::spawn(move || debugger.run());

    let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();
    glfw.window_hint(glfw::WindowHint::ClientApi(glfw::ClientApiHint::NoApi));

    let (mut window, events) = glfw
        .create_window(
            640, // this is x4 the gameboy's resolution
            576,
            "Koholint Gameboy Emulator",
            glfw::WindowMode::Windowed,
        )
        .expect("Failed to create GLFW window.");

    window.set_key_polling(true);
    //window.make_current();

    let mut renderer = renderer::Renderer::new(&mut window, console.clone()).block_on();

    let mut fps_start = std::time::Instant::now();
    let mut frames = 0;
    while !renderer.window().should_close() {
        let frame_start = std::time::Instant::now();
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            renderer.handle_window_event(event);
        }

        // FIXME : rendering one big frame at 60hz is not accurate enough :
        // many games modify stuff mid-frame to create effects
        // for good accuracy, the frame needs to be drawn line-by-line
        // i might have to revert to singlethreaded for that
        renderer.render().unwrap();
        frames += 1;
        if fps_start.elapsed().as_millis() >= 1000 {
            println!("FPS : {frames}");

            frames = 0;
            fps_start = std::time::Instant::now();
        }

        while frame_start.elapsed().as_millis() < 16 {}
    }
}
