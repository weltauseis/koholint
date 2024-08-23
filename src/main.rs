use gameboy::Gameboy;

mod cpu;
mod gameboy;
mod memory;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("Usage : gbemulator <rom file>");
        return;
    }

    env_logger::init();

    let rom = std::fs::read(&args[1]).unwrap();
    let mut gameboy = Gameboy::new(rom);

    loop {
        gameboy.step();
    }
}
