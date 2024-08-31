use std::error::Error;

use debugger::debug_console;
use gameboy::Gameboy;
use log::info;
use winit::{
    application::ApplicationHandler, event::WindowEvent, event_loop::EventLoop, window::Window,
};

#[allow(dead_code)]
mod cpu;
mod debugger;
#[allow(dead_code)]
mod decoding;
mod gameboy;
#[allow(non_contiguous_range_endpoints)]
mod memory;

struct Application {
    window: Option<Window>,
}

impl ApplicationHandler<EmulatorEvent> for Application {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        info!("Resumed the event loop");

        // Create initial window.
        self.window = Some(
            event_loop
                .create_window(Window::default_attributes())
                .expect("failed to create initial window"),
        );
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                // Redraw the application.
                //
                // It's preferable for applications that do not render continuously to render in
                // this event rather than in AboutToWait, since rendering in here allows
                // the program to gracefully handle redraws requested by the OS.

                // Draw.

                // Queue a RedrawRequested event.
                //
                // You only need to call this if you've determined that you need to redraw in
                // applications which do not always need to. Applications that redraw continuously
                // can render here instead.
                self.window.as_ref().unwrap().request_redraw();
            }
            _ => (),
        }
    }

    fn user_event(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        event: EmulatorEvent,
    ) {
        info!("User event: {event:?}");
    }
}

#[derive(Debug, Clone, Copy)]
enum EmulatorEvent {
    VRAMUpdate,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("Usage : gbemulator <rom file>");
        return;
    }

    env_logger::init();

    /* loop {
        gameboy.step();
    } */

    let event_loop = EventLoop::<EmulatorEvent>::with_user_event()
        .build()
        .unwrap();
    let event_loop_proxy = event_loop.create_proxy();

    std::thread::spawn(move || {
        let rom = std::fs::read(&args[1]).unwrap();
        let gameboy = Gameboy::new(rom);

        /* loop {
            let _ = event_loop_proxy.send_event(EmulatorEvent::VRAMUpdate);
            std::thread::sleep(std::time::Duration::from_secs(1));
        } */

        debug_console(gameboy);
    });

    let mut app = Application { window: None };
    event_loop.run_app(&mut app).unwrap();
}
