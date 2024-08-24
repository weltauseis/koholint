use log::{info, trace, warn};

// https://gbdev.io/pandocs/Memory_Map.html
// FIXME : add support for MBC and switchable ROM banks
// FIXME : add support for switchable WRAM in gameboy color mode

pub struct Memory {
    rom_bank: [u8; 0x8000], // 0000-7FFF | 32 KiB ROM bank, no MBC support for now
    wram: [u8; 0x4000],     // C000-CFFF | 4 KiB Work RAM
    switchable_wram: [u8; 0x4000], // D000-DFFF | 4 KiB Work RAM
}

impl Memory {
    // constructor
    pub fn new() -> Memory {
        return Memory {
            rom_bank: [0; 0x8000],
            wram: [0; 0x4000],
            switchable_wram: [0; 0x4000],
        };
    }

    pub fn load_rom(&mut self, rom: Vec<u8>) {
        if rom.len() == 256 {
            // special case of the boot ROM
            info!("BOOT ROM LOADED");
            self.rom_bank[..rom.len()].copy_from_slice(&rom);
            return;
        }

        // check the cartridge byte

        let mbc_byte = rom[0x0147];
        info!(
            "CARTRIDGE TYPE : {}",
            match mbc_byte {
                0x00 => "ROM ONLY",
                0x01 => "MBC1",
                0x02 => "MBC1+RAM",
                0x03 => "MBC1+RAM+BATTERY",
                0x05 => "MBC2",
                0x06 => "MBC2+BATTERY",
                0x08 => "ROM+RAM",
                0x09 => "ROM+RAM+BATTERY",
                0x0B => "MMM01",
                0x0C => "MMM01+RAM",
                0x0D => "MMM01+RAM+BATTERY",
                0x0F => "MBC3+TIMER+BATTERY",
                0x10 => "MBC3+TIMER+RAM+BATTERY",
                0x11 => "MBC3",
                0x12 => "MBC3+RAM",
                0x13 => "MBC3+RAM+BATTERY",
                0x19 => "MBC5",
                0x1A => "MBC5+RAM",
                0x1B => "MBC5+RAM+BATTERY",
                0x1C => "MBC5+RUMBLE",
                0x1D => "MBC5+RUMBLE+RAM",
                0x1E => "MBC5+RUMBLE+RAM+BATTERY",
                0xFC => "POCKET CAMERA",
                0xFD => "BANDAI TAMA5",
                0xFE => "HuC3",
                0xFF => "HuC1+RAM+BATTERY",
                _ => "UNKNOWN",
            }
        );

        if mbc_byte != 0x00 {
            panic!("ROMS using a MEMORY BANK CONTROLLER are not yet supported !");
        }

        if rom.len() > self.rom_bank.len() {
            panic!("ROM is too big ! {} < {}", rom.len(), self.rom_bank.len());
        }

        self.rom_bank[..rom.len()].copy_from_slice(&rom);
    }

    // accessors
    pub fn read_byte(&self, address: u16) -> u8 {
        match address {
            0x0000..0x8000 => {
                let byte = self.rom_bank[address as usize];
                trace!(
                    "Read byte {:#X} from ROM bank at address {:#04X}",
                    byte,
                    address
                );
                return byte;
            }
            _ => panic!("READ_BYTE : INVALID ADDRESS ({:#06X})", address),
        }
    }

    pub fn write_byte(&mut self, address: u16, value: u8) {
        match address {
            0x0000..0x8000 => {
                panic!("CANNOT WRITE TO ROM BANK");
            }
            0xC000..0xD000 => {
                trace!(
                    "Wrote byte {:#04X} to WRAM at address {:#06X}",
                    value,
                    address
                );
                self.wram[(address - 0xC000) as usize] = value;
            }
            0xD000..0xE000 => {
                trace!(
                    "Wrote byte {:#04X} to switchable WRAM at address {:#06X}",
                    value,
                    address
                );
                self.switchable_wram[(address - 0xD000) as usize] = value;
                warn!("SWITCHABLE WRAM NOT YET SUPPORTED, BEHAVIOR MAY BE UNEXPECTED !");
            }
            _ => panic!("WRITE_BYTE : INVALID ADDRESS ({:#04X})", address),
        }
        //self.memory[address] = value;
    }

    pub fn read_word(&self, address: u16) -> u16 {
        match address {
            0x0000..0x8000 => {
                let word = u16::from_le_bytes(
                    self.rom_bank[(address as usize)..((address + 2) as usize)]
                        .try_into()
                        .unwrap(),
                );

                trace!(
                    "Read word {:#X} from ROM bank at address {:#X}",
                    word,
                    address
                );

                return word;
            }
            _ => panic!("INVALID ADDRESS"),
        }
    }

    /*
    pub fn write_word(&mut self, address: usize, value: u16) {
        let value_bytes = value.to_le_bytes();
        self.memory[address] = value_bytes[0];
        self.memory[address + 1] = value_bytes[1];
    } */
}
