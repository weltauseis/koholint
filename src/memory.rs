use log::{info, trace, warn};

use crate::error::{EmulationError, EmulationErrorType};

// https://gbdev.io/pandocs/Memory_Map.html
// TODO : add support for MBC and switchable ROM banks
// TODO : add support for switchable WRAM in gameboy color mode
// TODO : add support for switchable VRAM in gameboy color mode
// TODO : add support for switchable external RAM

pub struct Memory {
    boot_rom: [u8; 0x100],        // 0000-00FF | Boot ROM (mapped only during boot)
    fixed_rom_bank: [u8; 0x4000], // 0000-3FFF | 16 KiB fixed ROM bank
    switch_rom_bank: Vec<[u8; 0x4000]>, // 4000-7FFF | 16 KiB switchable ROM bank
    vram: [u8; 0x2000],           // 8000-9FFF | 8KiB Video Ram
    ext_ram: [u8; 0x2000],        // A000-BFFF | 8 KiB External RAM (cartridge)
    wram: [u8; 0x4000],           // C000-CFFF | 4 KiB Work RAM
    switchable_wram: [u8; 0x4000], // D000-DFFF | 4 KiB Work RAM
    oam: [u8; 160],               // FE00-FE9F | Object Attribute Memory
    io_hw: [u8; 0x80],            // FF00-FF7F | Memory-Mapped I/O
    hram: [u8; 0x7F],             // FF80-FFFE | High Ram
    ie: u8,                       // FFFF      | Interrupt Enable Register (IE)
    // ---------------
    mbc: MBC,
    selected_rom_bank: u8,
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
            fixed_rom_bank: [0; 0x4000],
            switch_rom_bank: vec![[0; 0x4000]],
            vram: [0; 0x2000],
            ext_ram: [0; 0x2000],
            wram: [0; 0x4000],
            switchable_wram: [0; 0x4000],
            oam: [0; 0x00A0],
            io_hw: [0; 0x80],
            hram: [0; 0x7F],
            ie: 0x00,
            mbc: MBC::NONE,
            selected_rom_bank: 1,
        };
    }

    pub fn load_rom(&mut self, rom: Vec<u8>) {
        assert!(rom.len() >= 0x4000, "ROM is too small !");

        info!(
            "CARTRIDGE TITLE : {}",
            std::str::from_utf8(&rom[0x0134..0x0143])
                .expect("TITLE field in cartdrige header is not ASCII : your rom may be corrupted")
        );

        // check the cartridge memory bank controller byte
        let mbc_byte = rom[0x0147];
        match mbc_byte {
            0x00 => {
                info!("CARTRIDGE TYPE : ROM ONLY");
                assert!(
                    rom.len() <= 0x8000,
                    "ROM size too big for its MBC type : the ROM file may be corrupted."
                );

                // simply map the rom to the two banks
                self.fixed_rom_bank.copy_from_slice(&rom[0..0x4000]);
                if rom.len() > 0x4000 {
                    self.switch_rom_bank[0][..(rom.len() - 0x4000)].copy_from_slice(&rom[0x4000..]);
                }
            }
            0x01 => {
                info!("CARTRIDGE TYPE : MBC1");
                self.mbc = MBC::MBC1;

                // map the fixed rom bank,
                // then the switchable banks until all the rom has been mapped
                self.fixed_rom_bank[..].copy_from_slice(&rom[0..0x4000]);
                let mut switchable_banks: Vec<[u8; 0x4000]> = Vec::new();
                let mut mapped = 0x4000;
                while mapped < rom.len() {
                    let mut bank = [0; 0x4000];
                    let to_copy: usize = 0x4000.min(rom.len() - mapped);
                    bank[0..to_copy].copy_from_slice(&rom[mapped..(mapped + to_copy)]);
                    switchable_banks.push(bank);

                    mapped += 0x4000;
                }

                self.switch_rom_bank = switchable_banks;

                info!(
                    "MBC1 : {} ROM BANKS (TOTAL SIZE : {}KiB)",
                    1 + self.switch_rom_bank.len(),
                    rom.len() / 0x400,
                );
            }
            _ => {
                panic!(
                    "ROMS using this MEMORY BANK CONTROLLER are not yet supported : {}",
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
            }
        }

        //FIXME : until display is implemented, pretend we are always in V-Blank
        // value at 0xFF44 is used to determine vertical-blank period
        self.io_hw[0x44] = 144;
    }

    // accessors
    pub fn read_byte(&self, address: u16) -> u8 {
        match address {
            // ROM / BOOT ROM
            0x0000..0x4000 => {
                // the bootrom stays mapped onto 0x00-0xFF until 0x01 is written to 0xFF50
                // then cartridge data is accessible
                if (self.read_byte(0xFF50) == 0) && (address <= 0xFF) {
                    return self.boot_rom[address as usize];
                } else {
                    return self.fixed_rom_bank[address as usize];
                }
            }
            // SECOND ROM BANK
            0x4000..0x8000 => match self.mbc {
                MBC::NONE => self.switch_rom_bank[0][(address - 0x4000) as usize],
                MBC::MBC1 => {
                    let selected = self.selected_rom_bank.max(1);
                    // bank 0 is the fixed bank, hence the -1 here
                    self.switch_rom_bank[selected as usize - 1][(address - 0x4000) as usize]
                }
                _ => {
                    todo!("other mbc types")
                }
            },
            // VRAM
            0x8000..0xA000 => {
                return self.vram[(address - 0x8000) as usize];
            }
            // WRAM
            0xC000..0xD000 => {
                return self.wram[(address - 0xC000) as usize];
            }
            // SWITCHABLE WRAM
            0xD000..0xE000 => {
                return self.switchable_wram[(address - 0xD000) as usize];
            }
            // MEMORY IO
            0xFF00..0xFF80 => {
                // filtering the adress to warn for unimplemented things
                match address {
                    0xFF00 => {
                        warn!("JOYPAD INPUT READ");
                        return 0x0F;
                    }
                    0xFF42..=0xFF43 => { /* screen scrolling bytes,it's fine to access */ }
                    0xFF44..=0xFF45 => {
                        // LY indicates the current horizontal line
                        // LYC indicates on which line an interrupt should be triggered
                    }
                    0xFF07 => { /* timer info byte, fine too */ }
                    0xFF04 => { /* divider register byte, fine too */ }
                    0xFF50 => { /* disables the boot rom when non-zero */ }
                    0xFF0F => { /* interrupt request register */ }

                    0xFF10..=0xFF26 => {
                        /* audio stuff is less important for now */
                        info!("CALL TO AUDIO MEMORY READ (ADDRESS {:#06X})", address);
                    }

                    _ => {
                        info!("READ MEMORY FROM FF00-FF80 RANGE (IO & MEM-MAPPED HW REGISTERS) (ADDRESS {:#06X})", address);
                    }
                }

                return self.io_hw[(address - 0xFF00) as usize];
            }
            // HRAM
            0xFF80..0xFFFF => {
                return self.hram[(address - 0xFF80) as usize];
            }
            // INTERRUP ENABLE
            0xFFFF => {
                warn!("CALL TO INTERRUPT ENABLE READ");
                return self.ie;
            }
            _ => panic!("READ_BYTE : INVALID ADDRESS ({:#06X})", address),
        }
    }

    pub fn write_byte(&mut self, address: u16, value: u8) -> Result<(), EmulationError> {
        match address {
            // ROM
            0x0000..0x2000 => {
                // writing to this rom space enables external ram
                // but for now we ignore it
                // FIXME
                warn!("WRITE TO EXT RAM ENABLE ({:#06X})", address);
            }
            0x2000..0x4000 => {
                // writing to this rom address range selects the rom bank
                // for the MBC
                let nb_additional_banks = self.switch_rom_bank.len() as u8;
                let mut corrected_value = value;

                if corrected_value == 0 {
                    // bank 0 is mapped in a fixed manner to 0000..=4000
                    // and can't be mapped twice
                    corrected_value = 1
                };
                if corrected_value > nb_additional_banks {
                    // on the gameboy, this register is masked
                    // so that you can't map a bank that doesn't exist
                    corrected_value = nb_additional_banks;
                }

                self.selected_rom_bank = corrected_value;
            }
            0x4000..0x6000 => {
                // writing to this range  switches the selected ram bank
                // for 32 KiB RAM cartridges
                todo!("ram switching")
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
            // OAM
            0xFE00..0xFEA0 => {
                warn!("WRITE TO OAM");
                self.oam[(address - 0xFE00) as usize] = value;
            }
            0xFEA0..0xFF00 => {
                // Nintendo says use of this area is prohibited.
                warn!("WRITE TO PROHIBITED MEMORY");
            }
            // IO & MEMORY MAPPED HARDWARE REGISTERS
            0xFF00..0xFF80 => match address {
                0xFF01 => {
                    info!("WRITE TO SERIAL DATA REGISTER");
                }
                0xFF02 => {
                    info!("WRITE TO SERIAL CONTROL REGISTER");
                }
                0xFF04 => {
                    // writing to the DIV register clears it
                    self.io_hw[0x04] = 0x00;
                }
                0xFF10..0xFF40 => {
                    // audio registers, not important for now
                    info!("WRITE TO AUDIO REGISTER ({:#06X})", address);
                }
                0xFF41 => {
                    // the lower 3 bits of LCD STAT are read-only
                    // and should not be overwritten by a call to write_byte
                    self.io_hw[0x41] = (self.io_hw[0x41] & 0b_0000_0111) | (value & 0b_1111_1000);
                }
                0xFF0F |            // IF 
                0xFF40 |            // LCD CONTROL
                0xFF42 | 0xFF43 |   // SCX & SCY
                0xFF50 |            // DISABLES BOOT ROM
                0xFF05..=0xFF07     // TIMA, TMA, TAC
                => {
                    // those are all registers that are R/W
                    // they act like normal registers / memory
                    self.io_hw[(address - 0xFF00) as usize] = value;
                }
                0xFF47 => {
                    info!("WRITE TO PALETTE REGISTER");
                }
                0xFF48 | 0xFF49 => {
                    info!("WRITE TO OBJ PALETTE REGISTER");
                }
                0xFF7F => {
                    // this register is supposed to be unused but dr mario somehow writes to it
                    // maybe a bug in my emulator
                    warn!("WEIRD DR MARIO WRITE AT 0xFF7F");
                }
                _ => {
                    /* return Err(EmulationError {
                        ty: EmulationErrorType::UnauthorizedWrite(address),
                        pc: None,
                    }); */
                }
            },
            // HRAM
            0xFF80..0xFFFF => {
                self.hram[(address - 0xFF80) as usize] = value;
            }
            // INTERRUP ENABLE
            0xFFFF => {
                warn!("CALL TO INTERRUPT ENABLE WRITE");
                self.ie = value;
            }
            _ => {
                return Err(EmulationError {
                    ty: EmulationErrorType::UnauthorizedWrite(address),
                    pc: None,
                });
            }
        }

        Ok(())
    }

    pub fn read_word(&self, address: u16) -> u16 {
        let mut bytes: [u8; 2] = [0; 2];
        bytes[0] = self.read_byte(address);
        bytes[1] = self.read_byte(address + 1);
        return u16::from_le_bytes(bytes);
    }

    pub fn write_word(&mut self, address: u16, value: u16) -> Result<(), EmulationError> {
        let value_bytes = value.to_le_bytes();
        self.write_byte(address, value_bytes[0])?;
        self.write_byte(address + 1, value_bytes[1])?;

        Ok(())
    }

    // functions to write to the hw registers bypassing the MMU

    // the lower 3 bits of LCD STAT are read-only
    // and they should not be overwritten by a call to write_byte
    // however the cpu needs to have a way to update them
    // bits 0 & 1 are the current PPU MODE
    pub fn _update_lcd_stat_ppu_mode(&mut self, ppu_mode: u8) {
        assert!(ppu_mode < 4, "PPU mode should be a 2-bit value (0-3) !");
        let lcd_stat = self.io_hw[0x41];

        self.io_hw[0x41] = (lcd_stat & 0b_1111_1100) | ppu_mode;
    }
    // bit 2 is set if LCY == LY
    pub fn update_lcd_stat_lcy_eq_ly(&mut self, lcy_eq_ly: bool) {
        let lcd_stat = self.io_hw[0x41];
        self.io_hw[0x41] = if lcy_eq_ly {
            lcd_stat | 0b_0000_1000
        } else {
            lcd_stat & 0b_1111_0111
        }
    }

    // writing to the DIV register from code should set it to 0x00
    // so this function is used to update it without clearing it
    pub fn increment_div(&mut self) {
        let div = self.io_hw[0x04];
        self.io_hw[0x04] = div.wrapping_add(1);
    }

    // TIMA overflow should request an interrupt
    pub fn increment_tima(&mut self) {
        let new_tima = self.io_hw[0x05].wrapping_add(1);
        self.io_hw[0x05] = new_tima;

        // overflow !!
        if new_tima == 0 {
            // TIMA is reset to the value in TMA
            // and a timer interrupt is requested
            self.io_hw[0x05] = self.io_hw[0x06];

            // timer interrupt is bit 2
            let interrupts = self.io_hw[0x0F];
            self.io_hw[0x0F] = interrupts | 0b0000_0100;
        }
    }

    // LCD Y is read only
    pub fn increment_ly(&mut self) {
        let ly = self.io_hw[0x44];
        self.io_hw[0x44] = (ly + 1) % 154;
    }

    // LCD control byte flags
    /* pub fn read_lcd_ctrl_flag(&self, bit: u8) -> bool {
        let lcd_ctrl = self.read_byte(0xFF40);

        return ((lcd_ctrl >> bit) & 1) == 1;
    } */

    // Interrupts functions
    // https://gbdev.io/pandocs/Interrupts.html
    pub fn is_interrupt_enabled(&self, interrupt: u8) -> bool {
        let interrupt_enable_byte = self.ie;

        return (interrupt_enable_byte >> interrupt) & 1 == 1;
    }

    pub fn is_interrupt_requested(&self, interrupt: u8) -> bool {
        let interrupt_request_byte = self.read_byte(0xFF0F);

        return (interrupt_request_byte >> interrupt) & 1 == 1;
    }

    pub fn request_interrupt(&mut self, interrupt: u8) {
        let interrupt_request_byte = self.read_byte(0xFF0F);
        self.io_hw[0x0F] = interrupt_request_byte | (1 << interrupt);
    }

    pub fn clear_interrupt(&mut self, interrupt: u8) {
        let interrupt_request_byte = self.read_byte(0xFF0F);
        self.io_hw[0x0F] = interrupt_request_byte & !(1 << interrupt);
    }

    pub fn interrupt_pending_and_enabled(&self) -> bool {
        let ie = self.ie;
        let i_request = self.io_hw[0x0F];

        return (ie & i_request) != 0;
    }

    // Timer
    pub fn is_timer_started(&self) -> bool {
        // 0xFF07 : 2       |   1   0
        //          Enable  |   Clock Select

        return self.read_byte(0xFF07) >> 2 & 1 == 1;
    }
}

enum MBC {
    NONE,
    MBC1,
    MBC2,
    MBC3,
}
