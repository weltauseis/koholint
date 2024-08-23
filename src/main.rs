use cpu::CPU;
use memory::Memory;

mod cpu;
mod memory;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("Usage : gbemulator <rom file>");
        return;
    }

    let mut memory = Memory::new();
    //let mut cpu = CPU::new();

    let rom = std::fs::read(&args[1]).unwrap();
    memory.load_rom(rom);
}
