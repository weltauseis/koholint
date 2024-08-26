use log::{error, info, trace, warn};

// https://gbdev.io/pandocs/Memory_Map.html
// FIXME : add support for MBC and switchable ROM banks
// FIXME : add support for switchable WRAM in gameboy color mode
// FIXME : add support for switchable VRAM in gameboy color mode
// FIXME : add support for switchable external RAM

pub struct Memory {
    boot_rom: [u8; 0x100],         // 0000-00FF | Boot ROM (mapped only during boot)
    rom_bank: [u8; 0x8000],        // 0000-7FFF | 32 KiB ROM bank, no MBC support for now
    vram: [u8; 0x2000],            // 8000-9FFF | 8KiB Video Ram
    ext_ram: [u8; 0x2000],         // A000-BFFF | 8 KiB External RAM (cartridge)
    wram: [u8; 0x4000],            // C000-CFFF | 4 KiB Work RAM
    switchable_wram: [u8; 0x4000], // D000-DFFF | 4 KiB Work RAM
    io: [u8; 0x80],                // FF00-FF7F | Memory-Mapped I/O
    hram: [u8; 0x7F],              // FF80-FFFE | High Ram
}

impl Memory {
    // constructor
    pub fn new() -> Memory {
        return Memory {
            // https://gbdev.gg8.se/wiki/articles/Gameboy_Bootstrap_ROM
            // i don't think it is illegal to include this here
            boot_rom: [
                0x31, 0xfe, 0xff, 0xaf, 0x21, 0xff, 0x9f, 0x32, 0xcb, 0x7c, 0x20, 0xfb, 0x21, 0x26,
                0xff, 0x0e, 0x11, 0x3e, 0x80, 0x32, 0xe2, 0x0c, 0x3e, 0xf3, 0xe2, 0x32, 0x3e, 0x77,
                0x77, 0x3e, 0xfc, 0xe0, 0x47, 0x11, 0x04, 0x01, 0x21, 0x10, 0x80, 0x1a, 0xcd, 0x95,
                0x00, 0xcd, 0x96, 0x00, 0x13, 0x7b, 0xfe, 0x34, 0x20, 0xf3, 0x11, 0xd8, 0x00, 0x06,
                0x08, 0x1a, 0x13, 0x22, 0x23, 0x05, 0x20, 0xf9, 0x3e, 0x19, 0xea, 0x10, 0x99, 0x21,
                0x2f, 0x99, 0x0e, 0x0c, 0x3d, 0x28, 0x08, 0x32, 0x0d, 0x20, 0xf9, 0x2e, 0x0f, 0x18,
                0xf3, 0x67, 0x3e, 0x64, 0x57, 0xe0, 0x42, 0x3e, 0x91, 0xe0, 0x40, 0x04, 0x1e, 0x02,
                0x0e, 0x0c, 0xf0, 0x44, 0xfe, 0x90, 0x20, 0xfa, 0x0d, 0x20, 0xf7, 0x1d, 0x20, 0xf2,
                0x0e, 0x13, 0x24, 0x7c, 0x1e, 0x83, 0xfe, 0x62, 0x28, 0x06, 0x1e, 0xc1, 0xfe, 0x64,
                0x20, 0x06, 0x7b, 0xe2, 0x0c, 0x3e, 0x87, 0xe2, 0xf0, 0x42, 0x90, 0xe0, 0x42, 0x15,
                0x20, 0xd2, 0x05, 0x20, 0x4f, 0x16, 0x20, 0x18, 0xcb, 0x4f, 0x06, 0x04, 0xc5, 0xcb,
                0x11, 0x17, 0xc1, 0xcb, 0x11, 0x17, 0x05, 0x20, 0xf5, 0x22, 0x23, 0x22, 0x23, 0xc9,
                0xce, 0xed, 0x66, 0x66, 0xcc, 0x0d, 0x00, 0x0b, 0x03, 0x73, 0x00, 0x83, 0x00, 0x0c,
                0x00, 0x0d, 0x00, 0x08, 0x11, 0x1f, 0x88, 0x89, 0x00, 0x0e, 0xdc, 0xcc, 0x6e, 0xe6,
                0xdd, 0xdd, 0xd9, 0x99, 0xbb, 0xbb, 0x67, 0x63, 0x6e, 0x0e, 0xec, 0xcc, 0xdd, 0xdc,
                0x99, 0x9f, 0xbb, 0xb9, 0x33, 0x3e, 0x3c, 0x42, 0xb9, 0xa5, 0xb9, 0xa5, 0x42, 0x3c,
                0x21, 0x04, 0x01, 0x11, 0xa8, 0x00, 0x1a, 0x13, 0xbe, 0x20, 0xfe, 0x23, 0x7d, 0xfe,
                0x34, 0x20, 0xf5, 0x06, 0x19, 0x78, 0x86, 0x23, 0x05, 0x20, 0xfb, 0x86, 0x20, 0xfe,
                0x3e, 0x01, 0xe0, 0x50,
            ],
            rom_bank: [0; 0x8000],
            vram: [0; 0x2000],
            ext_ram: [0; 0x2000],
            wram: [0; 0x4000],
            switchable_wram: [0; 0x4000],
            io: [0; 0x80],
            hram: [0; 0x7F],
        };
    }

    pub fn load_rom(&mut self, rom: Vec<u8>) {
        // check the cartridge type byte
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
            // ROM / BOOT ROM
            0x0000..0x8000 => {
                // the bootrom stays mapped onto 0x00-0xFF until 0x01 is written to 0xFF50
                // then cartridge data is accessible
                if (self.io[(0xFF50 - 0xFF00) as usize] == 0) && (address <= 0xFF) {
                    return self.boot_rom[address as usize];
                } else {
                    return self.rom_bank[address as usize];
                }
            }
            // MEMORY IO
            0xFF00..0xFF80 => {
                error!(
                    "CALL TO UNIMPLEMENTED IO MEMORY READ (ADDRESS {:#06X})",
                    address
                );

                return 0;
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
                warn!("CALL TO IO MEMORY WRITE (ADDRESS {:#06X})", address);
                self.io[(address - 0xFF00) as usize] = value;
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
