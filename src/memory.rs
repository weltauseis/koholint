use log::{info, trace, warn};

// https://gbdev.io/pandocs/Memory_Map.html
// FIXME : add support for MBC and switchable ROM banks
// FIXME : add support for switchable WRAM in gameboy color mode
// FIXME : add support for switchable VRAM in gameboy color mode
// FIXME : add support for switchable external RAM

pub struct Memory {
    rom_bank: [u8; 0x8000], // 0000-7FFF | 32 KiB ROM bank, no MBC support for now
    vram: [u8; 0x2000],     // 8000-9FFF | 8KiB Video Ram
    ext_ram: [u8; 0x2000],  // A000-BFFF | 8 KiB External RAM (cartridge)
    wram: [u8; 0x4000],     // C000-CFFF | 4 KiB Work RAM
    switchable_wram: [u8; 0x4000], // D000-DFFF | 4 KiB Work RAM
    hram: [u8; 0x7F],       // FF80-FFFE | High Ram
}

impl Memory {
    // constructor
    pub fn new() -> Memory {
        return Memory {
            rom_bank: [0; 0x8000],
            vram: [0; 0x2000],
            ext_ram: [0; 0x2000],
            wram: [0; 0x4000],
            switchable_wram: [0; 0x4000],
            hram: [0; 0x7F],
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
            // ROM
            0x0000..0x8000 => {
                return self.rom_bank[address as usize];
            }
            // HRAM
            0xFF80..0xFFFF => {
                return self.hram[(address - 0xFF80) as usize];
            }
            _ => panic!("READ_BYTE : INVALID ADDRESS ({:#06X})", address),
        }
    }

    pub fn write_byte(&mut self, address: u16, value: u8) {
        match address {
            // ROM
            0x0000..0x8000 => {
                panic!("CANNOT WRITE TO ROM BANK");
            }
            // VRAM
            0x8000..0xA000 => {
                trace!(
                    "Wrote byte {:#04X} to VRAM at address {:#06X}",
                    value,
                    address
                );
                self.vram[(address - 0x8000) as usize] = value;
            }
            // EXTERNAL RAM
            0xA000..0xC000 => {
                trace!(
                    "Wrote byte {:#04X} to EXTERNAL RAM at address {:#06X}",
                    value,
                    address
                );
                self.ext_ram[(address - 0xA000) as usize] = value;
            }
            // WRAM
            0xC000..0xD000 => {
                trace!(
                    "Wrote byte {:#04X} to WRAM at address {:#06X}",
                    value,
                    address
                );
                self.wram[(address - 0xC000) as usize] = value;
            }
            // SWITCHABLE WRAM
            0xD000..0xE000 => {
                trace!(
                    "Wrote byte {:#04X} to switchable WRAM at address {:#06X}",
                    value,
                    address
                );
                self.switchable_wram[(address - 0xD000) as usize] = value;
                //warn!("SWITCHABLE WRAM NOT YET SUPPORTED, BEHAVIOR MAY BE UNEXPECTED !");
            }
            // MEMORY IO
            0xFF00..0xFF80 => {
                warn!(
                    "CALL TO UNIMPLEMENTED IO MEMORY WRITE (ADDRESS {:#06X})",
                    address
                );
            }
            // HRAM
            0xFF80..0xFFFF => {
                self.hram[(address - 0xFF80) as usize] = value;
            }
            _ => panic!("WRITE_BYTE : INVALID ADDRESS ({:#04X})", address),
        }
    }

    pub fn read_word(&self, address: u16) -> u16 {
        let mut bytes: [u8; 2] = [0; 2];
        bytes[0] = self.read_byte(address);
        bytes[1] = self.read_byte(address + 1);
        return u16::from_le_bytes(bytes);
    }

    pub fn write_word(&mut self, address: u16, value: u16) {
        let value_bytes = value.to_le_bytes();
        self.write_byte(address, value_bytes[0]);
        self.write_byte(address + 1, value_bytes[1]);
    }
}
