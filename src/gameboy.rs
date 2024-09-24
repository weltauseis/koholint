use core::panic;
use log::{debug, info, warn};

use crate::{
    cpu::CPU,
    decoding::{self, Instruction, Operand, Operation},
    error::{EmulationError, EmulationErrorType},
    input::GBInputState,
    memory::Memory,
};

const SCREEN_W: usize = 160;
const SCREEN_H: usize = 144;
const BYTES_PER_PIXELS: usize = 4; // rgba_u8
const TEXTURES_W: usize = 256;

pub struct Gameboy {
    cpu: CPU,
    memory: Memory,
    // keeps track of cycles elapsed to update various registers
    div_cycles: u64,  // DIV TIMER
    ly_cycles: u64,   // LINE Y
    tima_cycles: u64, // MAIN TIMER
    halted: bool,
    // rendering
    tile_atlas: Box<[u8; TEXTURES_W * TEXTURES_W * BYTES_PER_PIXELS]>, // used for objects to sample
    tilemap: Box<[u8; TEXTURES_W * TEXTURES_W * BYTES_PER_PIXELS]>, // a particular arrangement of tiles used as background
    framebuffer: Box<[u8; SCREEN_W * SCREEN_H * BYTES_PER_PIXELS]>, // the current state of the gameboy screen
}

impl Gameboy {
    // constructor
    pub fn new(rom: Vec<u8>) -> Gameboy {
        let mut mem = Memory::new();
        mem.load_rom(rom);
        return Gameboy {
            cpu: CPU::blank(),
            memory: mem,
            div_cycles: 0,
            ly_cycles: 0,
            tima_cycles: 0,
            halted: false,
            tile_atlas: Box::new([0; TEXTURES_W * TEXTURES_W * BYTES_PER_PIXELS]),
            tilemap: Box::new([0; TEXTURES_W * TEXTURES_W * BYTES_PER_PIXELS]),
            framebuffer: Box::new([0; SCREEN_W * SCREEN_H * BYTES_PER_PIXELS]),
        };
    }

    // accessors to watch values
    pub fn cpu(&self) -> &CPU {
        return &self.cpu;
    }

    pub fn memory(&self) -> &Memory {
        return &self.memory;
    }

    // functions

    pub fn step(&mut self) -> Result<u64, EmulationError> {
        let cycles_elapsed;

        if self.halted {
            // FIXME : handle this better
            if self.memory.interrupt_pending_and_enabled() {
                self.halted = false;
                self.handle_interrupts()?;
            }

            cycles_elapsed = 4;
        } else {
            let instr = decoding::decode_next_instruction(&self)?;
            let pc_before = self.cpu.read_program_counter();

            cycles_elapsed = self.execute_instruction(instr).map_err(|mut e| {
                // some errors that happen for example in memory access
                // can't know the instruction that called them
                // so we attach that info here
                if e.pc.is_none() {
                    e.pc = Some(pc_before);
                }
                e
            })?;
        }

        self.handle_interrupts()?;
        self.update_misc();

        // update timing infos
        self.div_cycles += cycles_elapsed;
        self.ly_cycles += cycles_elapsed;
        self.tima_cycles += cycles_elapsed;

        return Ok(cycles_elapsed);
    }

    pub fn update_input(&mut self, input_state: &GBInputState) {
        let buttons = self.memory.input_buttons_selected();
        let dpad = self.memory.input_dpad_selected();

        let mut bits: u8 = 0x0F;
        if buttons {
            if input_state.right {
                bits &= !0b1;
            }
            if input_state.left {
                bits &= !(0b10);
            }
            if input_state.up {
                bits &= !(0b100);
            }
            if input_state.down {
                bits &= !(0b1000);
            }
        }

        if dpad {
            if input_state.a {
                bits &= !(1);
            }
            if input_state.b {
                bits &= !(0b10);
            }
            if input_state.select {
                bits &= !(0b100);
            }
            if input_state.start {
                bits &= !(0b1000);
            }
        }

        self.memory.update_input_lower(bits);
        if bits < 0x0F {
            self.memory.request_interrupt(4);
        }
    }

    fn handle_interrupts(&mut self) -> Result<(), EmulationError> {
        if !self.cpu().interrupts_enabled() {
            return Ok(());
        }

        // interrupts are priority-based, so we need to check in order
        // VBLANK
        if self.memory.is_interrupt_enabled(0) && self.memory.is_interrupt_requested(0) {
            debug!("VBLANK INTERRUPT");
            self.cpu.disable_interrupts();
            self.memory.clear_interrupt(0);
            self.push_word(self.cpu.read_program_counter())?;
            self.cpu.write_program_counter(0x40);
        }
        // LCD
        else if self.memory.is_interrupt_enabled(1) && self.memory.is_interrupt_requested(1) {
            panic!("LCD INTERRUPT REQUESTED AND ENABLED");
        }
        // TIMER
        else if self.memory.is_interrupt_enabled(2) && self.memory.is_interrupt_requested(2) {
            info!("TIMER INTERRUPT");
            self.cpu.disable_interrupts();
            self.memory.clear_interrupt(2);
            self.push_word(self.cpu.read_program_counter())?;
            self.cpu.write_program_counter(0x50);

            // FIXME : Interrupts should take a lot more time to execute
            // https://gbdev.io/pandocs/Interrupts.html
        }
        // SERIAL
        else if self.memory.is_interrupt_enabled(3) && self.memory.is_interrupt_requested(3) {
            panic!("SERIAL INTERRUPT REQUESTED AND ENABLED");
        }
        // JOYPAD
        else if self.memory.is_interrupt_enabled(4) && self.memory.is_interrupt_requested(4) {
            panic!("JOYPAD INTERRUPT REQUESTED AND ENABLED");
        }

        return Ok(());
    }

    fn update_misc(&mut self) {
        // TIMA counter
        if self.memory.is_timer_started() {
            // incrementing frequency is decided by the TAC register
            // this variable is actually more of a period than a frequency
            let freq = match self.memory.read_byte(0xFF07) & 0b11 {
                0b00 => 1024,
                0b01 => 16,
                0b10 => 64,
                0b11 => 256,
                _ => unreachable!(),
            };

            if self.tima_cycles >= freq {
                // FIXME : maybe this should be a subtraction, tima behavior is weird
                // https://github.com/Hacktix/GBEDG/blob/master/timers/index.md
                self.tima_cycles = 0;

                self.memory.increment_tima();
            }
        }

        // DIV register
        if self.div_cycles >= 256 {
            self.div_cycles -= 256;
            self.memory.increment_div();
        }

        // LY register
        if !self.memory.is_lcd_enabled() {
            self.ly_cycles = 0
        };
        if self.ly_cycles >= (80 + 172 + 204) {
            self.ly_cycles -= 80 + 172 + 204;

            self.memory.increment_ly();
            self.draw_current_line();

            if self.memory.read_byte(0xFF44) == 144 {
                // V-BLANK INTERRUPT
                self.memory.request_interrupt(0);
            }
        }

        // LY - LYC compare : https://gbdev.io/pandocs/STAT.html#ff45--lyc-ly-compare
        let ly = self.memory.read_byte(0xFF44);
        let lyc = self.memory.read_byte(0xFF45);
        self.memory.update_lcd_stat_lcy_eq_ly(ly == lyc);
        // FIXME : update the PPU mode too : https://gbdev.io/pandocs/STAT.html#ff41--stat-lcd-status
    }

    pub fn get_framebuffer(&self) -> &[u8] {
        return &(*self.framebuffer);
    }

    fn draw_current_line(&mut self) {
        let line: usize = self.memory.read_byte(0xFF44) as usize;
        if line >= SCREEN_H {
            //v-blank period
            return;
        }

        if !self.memory.is_lcd_enabled() {
            // screen turned off
            self.framebuffer[(line * SCREEN_W * BYTES_PER_PIXELS)
                ..(line * SCREEN_W * BYTES_PER_PIXELS + SCREEN_W * BYTES_PER_PIXELS)]
                .copy_from_slice(&[30; SCREEN_W * BYTES_PER_PIXELS]);
            return;
        }

        if line == 0 {
            // we just returned from a v-blank period where vram might have been modified
            // so the tile atlas & tilemap needs to be updated
            self.update_tile_atlas();
            self.update_tile_map();
        }

        // first draw the tilemap at this line

        // the tilemap is 256 * 256, but the screen is only 160 * 144
        // so the scrolling registers tell us the top left corner pixel coordinates
        // of the screen view into the tilemap
        let (scroll_x, scroll_y) = self.memory.read_scrolling_registers();

        let tilemap_y = (line + scroll_y) % 256;
        for screen_x in 0..160 {
            let tilemap_x: usize = (screen_x + scroll_x) % 256;

            self.framebuffer[(line * SCREEN_W + screen_x) * BYTES_PER_PIXELS] =
                self.tilemap[(tilemap_y * 256 + tilemap_x) * BYTES_PER_PIXELS];
            self.framebuffer[(line * SCREEN_W + screen_x) * BYTES_PER_PIXELS + 1] =
                self.tilemap[(tilemap_y * 256 + tilemap_x) * BYTES_PER_PIXELS + 1];
            self.framebuffer[(line * SCREEN_W + screen_x) * BYTES_PER_PIXELS + 2] =
                self.tilemap[(tilemap_y * 256 + tilemap_x) * BYTES_PER_PIXELS + 2];
            self.framebuffer[(line * SCREEN_W + screen_x) * BYTES_PER_PIXELS + 3] =
                self.tilemap[(tilemap_y * 256 + tilemap_x) * BYTES_PER_PIXELS + 3];
        }

        // then we can draw the objects
        for obj in 0..40 {
            // the stored value is actually the screen y position + 16
            // y_pos is between -16 and SCREEN_H : sprites can be outside the screen
            // placing a sprite outside the screen (leaving the x & y pos bytes to 0) is actually
            // how you're meant to "disable" it being dsrawn
            let y_pos = self.memory.read_byte(0xFE00 + obj * 4) as isize - 16;

            // if the current line doesn't intersect the sprite, don't bother trying to draw it
            if !(y_pos..(y_pos + 8)).contains(&(line as isize)) {
                continue;
            }

            // same thing for x_pos: it is between -8 and SCREEN_W
            let x_pos = self.memory.read_byte(0xFE00 + obj * 4 + 1) as isize - 8;
            let sprite_id = self.memory.read_byte(0xFE00 + obj * 4 + 2);

            for x_pxl in 0..8 {
                // sprites can be half-outside and half-inside the screen
                if x_pos + (x_pxl as isize) >= 0 {
                    let pixel_start_framebuffer =
                        (line * SCREEN_W + (x_pos as usize) + x_pxl) * BYTES_PER_PIXELS;
                    let pixel_start_tile_atlas = ((sprite_id as usize / 32) * (TEXTURES_W * 8)
                        + (sprite_id as usize % 32) * 8
                        + x_pxl
                        + ((line - y_pos as usize) * TEXTURES_W))
                        * BYTES_PER_PIXELS;

                    self.framebuffer
                        [pixel_start_framebuffer..(pixel_start_framebuffer + BYTES_PER_PIXELS)]
                        .copy_from_slice(
                            &self.tile_atlas[pixel_start_tile_atlas
                                ..(pixel_start_tile_atlas + BYTES_PER_PIXELS)],
                        );
                }
            }
        }
    }

    // returns a 256 * 256 atlas (32 * 32 tiles)
    // with each pixel being an u8 encoding its value (0-3)
    // the gameboy holds only 384 tiles, i.e. 32 * 12
    // so a good chunk of the atlas is empty
    pub fn get_tile_atlas_2bpp(&self) -> Vec<u8> {
        let mut img = vec![0u8; 256 * 256];

        // https://gbdev.io/pandocs/Tile_Data.html
        // each tile is 16 bytes in memory
        // each couple of bytes encodes a line of the tile

        // this whole thing is pretty convoluted but i don't think there's a better way,
        // as the memory layout of the gameboy tiles is very different from that of "normal" images

        // for each tile
        for id in 0..384usize {
            let address = (0x8000 + id * 16) as u16;

            // for each line of the tile
            for y in 0..8 {
                let byte_1 = self.memory.read_byte(address + y * 2);
                let byte_2 = self.memory.read_byte(address + y * 2 + 1);

                for x in 0..8 {
                    let mut value: u8 = 0;

                    let bit_0 = byte_1 >> (7 - x) & 1;
                    let bit_1 = byte_2 >> (7 - x) & 1;

                    value |= bit_0;
                    value |= bit_1 << 1;

                    let pixel =
                        // tile start                              | pixel start
                        8 * (id % 32) + (8 * 8 * 32) * (id / 32) + ((y as usize) * 8 * 32) + (x as usize);

                    img[pixel] = value;
                }
            }
        }

        return img;
    }

    pub fn update_tile_atlas(&mut self) {
        let palette = self.get_palette();

        // https://gbdev.io/pandocs/Tile_Data.html
        // each tile is 16 bytes in memory
        // each couple of bytes encodes a line of the tile

        // this whole thing is pretty convoluted but i don't think there's a better way,
        // as the memory layout of the gameboy tiles is very different from that of "normal" images

        // for each tile
        for id in 0..384usize {
            let address = (0x8000 + id * 16) as u16;

            // for each line of the tile
            for y in 0..8 {
                let byte_1 = self.memory.read_byte(address + y * 2);
                let byte_2 = self.memory.read_byte(address + y * 2 + 1);

                for x in 0..8 {
                    let mut value: u8 = 0;

                    let bit_0 = byte_1 >> (7 - x) & 1;
                    let bit_1 = byte_2 >> (7 - x) & 1;

                    value |= bit_0;
                    value |= bit_1 << 1;

                    let pixel =
                        // tile start                              | pixel start
                        8 * (id % 32) + (8 * 8 * 32) * (id / 32) + ((y as usize) * 8 * 32) + (x as usize);

                    self.tile_atlas[(pixel * 4)..(pixel * 4 + 4)]
                        .copy_from_slice(&palette[value as usize]);
                }
            }
        }
    }

    pub fn update_tile_map(&mut self) {
        //https://gbdev.io/pandocs/Tile_Maps.html
        let mut indexes = [0; 32 * 32];
        let palette = self.get_palette();
        let addressing_mode_bit = self.memory.is_bg_tile_addressing_mode_normal();

        for i in 0..(32 * 32) {
            let index = self.memory.read_byte(0x9800 + i);
            indexes[i as usize] = if addressing_mode_bit {
                // index is just the index, starting at tile 0
                index as u32
            } else {
                // index is signed, and the base tile is 256 (first tile of group 3)
                let signed_index: i32 = i8::from_le_bytes(index.to_le_bytes()) as i32;
                (256 + signed_index) as u32
            };
        }

        let atlas = self.get_tile_atlas_2bpp();

        // for each tile
        for tile in 0..(32 * 32) {
            let index = indexes[tile] as usize;

            // for each line of the tile
            for y in 0..8 {
                for x in 0..8 {
                    let dst_pixel =
                    // tile start                              | pixel start
                    (8 * (tile % 32) + (8 * 8 * 32) * (tile / 32) + ((y as usize) * 8 * 32) + (x as usize)) * 4;

                    let atlas_pos = 8 * (index % 32)
                        + (8 * 8 * 32) * (index / 32)
                        + ((y as usize) * 8 * 32)
                        + (x as usize);

                    // https://lospec.com/palette-list/2bit-demichrome
                    self.tilemap[dst_pixel..(dst_pixel + 4)]
                        .copy_from_slice(&palette[atlas[atlas_pos] as usize]);
                }
            }
        }
    }

    // FIXME : LCD is always turned on for now, in reality it depends on
    // a certain byte in memory : 	LD ($FF00+$40),A	; $005d  Turn on LCD, showing Background
    pub fn get_obj_y_pos_buffer(&self) -> [u32; 40] {
        let mut buffer = [0; 40];
        for obj in 0..40 {
            buffer[obj] = self.memory.read_byte(0xFE00 + (obj * 4) as u16) as u32;
        }

        return buffer;
    }

    pub fn get_obj_x_pos_buffer(&self) -> [u32; 40] {
        let mut buffer = [0; 40];
        for obj in 0..40 {
            buffer[obj] = self.memory.read_byte(0xFE00 + 1 + (obj * 4) as u16) as u32;
        }

        return buffer;
    }

    pub fn get_obj_sprite_ids_buffer(&self) -> [u32; 40] {
        let mut buffer = [0; 40];
        for obj in 0..40 {
            buffer[obj] = self.memory.read_byte(0xFE00 + 2 + (obj * 4) as u16) as u32;
        }

        return buffer;
    }

    fn get_palette(&self) -> [[u8; 4]; 4] {
        // https://gbdev.io/pandocs/Palettes.html
        let palette_reg = self.memory.read_byte(0xFF47);

        let mut palette = [[0; 4]; 4];
        for id in 0..4 {
            let color = (palette_reg >> (id * 2)) & 3;
            palette[id] = match color {
                0 => [250, 251, 246, /* alpha */ 255],
                1 => [198, 183, 190, /* alpha */ 255],
                2 => [86, 90, 117, /* alpha */ 255],
                3 => [15, 15, 27, /* alpha */ 255],
                _ => unreachable!(),
            };
        }

        return palette;
    }

    fn execute_instruction(&mut self, instr: Instruction) -> Result<u64, EmulationError> {
        use Operand::*;

        // store instruction pc for crash messages
        let pc = self.cpu.read_program_counter();

        // increment PC before everything
        // seems consistent with the fact that relative jumps
        // are relative to the end of the jr instructionss

        self.cpu.increment_program_counter(instr.size);

        // keep track of timing
        let mut cycles_elapsed = instr.cycles;

        #[allow(unreachable_patterns)]
        match instr.op {
            Operation::NOP => {
                // nothing to do
            }
            Operation::LD { dst, src } => {
                // some instructions auto-increment the hl register
                // the timing is important
                let mut decrement_hl = false;
                let mut increment_hl = false;

                match dst {
                    // load into a 8-bit register
                    R8_A | R8_B | R8_C | R8_D | R8_E | R8_H | R8_L => {
                        let byte = match src {
                            // load from immediate byte
                            IMM8(imm8) => imm8,

                            // load from another 8-bit register
                            R8_A | R8_B | R8_C | R8_D | R8_E | R8_H | R8_L => {
                                self.cpu.read_r8(&src)
                            }

                            // load from memory with pointer
                            PTR(ptr) => match *ptr {
                                R16_BC | R16_DE | R16_HL | R16_HLD | R16_HLI => {
                                    if matches!(*ptr, R16_HLI) {
                                        increment_hl = true;
                                    }
                                    if matches!(*ptr, R16_HLD) {
                                        decrement_hl = true;
                                    }

                                    let address = self.cpu.read_r16(&ptr);
                                    self.memory.read_byte(address)
                                }
                                R8_C => {
                                    let address = self.cpu.read_r8(&ptr) as u16 + 0xFF00;
                                    self.memory.read_byte(address)
                                }
                                // address from imm8 : IO memory
                                IMM8(imm8) => self.memory.read_byte(0xFF00 + imm8 as u16),
                                // adress from imm16
                                IMM16(imm16) => self.memory.read_byte(imm16),
                                _ => panic!("(CRITICAL) LD : ILLEGAL POINTER {ptr} at {pc:#06X}"),
                            },
                            _ => panic!("(CRITICAL) LD : ILLEGAL SRC {src} at {pc:#06X}"),
                        };

                        self.cpu.write_r8(&dst, byte);
                    }
                    // load into a 16-bit register
                    R16_BC | R16_DE | R16_HL | R16_SP => {
                        let word = match src {
                            // load from immediate word
                            IMM16(imm16) => imm16,

                            // load from another 16-bit register
                            R16_BC | R16_DE | R16_HL | R16_SP => self.cpu.read_r16(&src),

                            // load from memory
                            // for 16-bit load, the memory location is always
                            // relative to the stack pointer, with a signed offset
                            PTR(ptr) => match *ptr {
                                IMM8_SIGNED(offset) => {
                                    let sp = self.cpu.read_r16(&R16_SP);
                                    sp.wrapping_add(offset as u16)
                                }
                                _ => panic!("(CRITICAL) LD : ILLEGAL STACK POINTER OFFSET {ptr} at {pc:#06X}"),
                            },
                            // special case of 0xF8
                            SP_PLUS_SIGNED_IMM8(imm8) => {
                                let sp = self.cpu.read_stack_pointer();
                                if imm8 >= 0 {
                                    sp.wrapping_add(imm8.abs() as u16)
                                } else {
                                    sp.wrapping_sub(imm8.abs() as u16)
                               }
                            }
                            _ => panic!("(CRITICAL) LD : ILLEGAL SRC {src} at {pc:#06X}"),
                        };

                        self.cpu.write_r16(&dst, word);
                    }
                    // load into memory
                    PTR(ptr) => {
                        let address = match *ptr {
                            // address from pointer in r16
                            R16_BC | R16_DE | R16_HL | R16_HLD | R16_HLI => {
                                if matches!(*ptr, R16_HLI) {
                                    increment_hl = true;
                                }
                                if matches!(*ptr, R16_HLD) {
                                    decrement_hl = true;
                                }
                                self.cpu.read_r16(&ptr)
                            }
                            // address from immediate word
                            IMM16(address) => address,
                            // address from r8 : IO memory
                            R8_C => 0xFF00 + self.cpu.read_r8(&R8_C) as u16,
                            // address from imm8 : IO memory
                            IMM8(imm8) => 0xFF00 + imm8 as u16,
                            _ => {
                                panic!("(CRITICAL) LD : ILLEGAL DST POINTER {ptr} at {pc:#06X}")
                            }
                        };

                        match src {
                            // load byte from r8
                            R8_A | R8_B | R8_C | R8_D | R8_E | R8_H | R8_L => {
                                self.memory.write_byte(address, self.cpu.read_r8(&src))?
                            }
                            // load word from sp register
                            R16_SP => {
                                self.memory
                                    .write_word(address, self.cpu.read_r16(&R16_SP))?;
                            }
                            // load immediate byte
                            IMM8(imm8) => {
                                self.memory.write_byte(address, imm8)?;
                            }
                            _ => panic!("(CRITICAL) LD : ILLEGAL SRC {src} at {pc:#06X}"),
                        }
                    }
                    _ => panic!("LD : UNHANDLED DESTINATION {dst} at {pc:#06X}"),
                }

                if increment_hl {
                    self.cpu
                        .write_r16(&R16_HL, self.cpu.read_r16(&R16_HL).wrapping_add(1));
                }
                if decrement_hl {
                    self.cpu
                        .write_r16(&R16_HL, self.cpu.read_r16(&R16_HL).wrapping_sub(1));
                }
            }
            Operation::INC { x } => match x {
                // increment 8-bit register
                R8_A | R8_B | R8_C | R8_D | R8_E | R8_H | R8_L => {
                    let reg = self.cpu.read_r8(&x);
                    let result = reg.wrapping_add(1);
                    self.cpu.write_r8(&x, result);

                    // inc flags : Z 0 H -
                    self.cpu.write_z_flag(result == 0);
                    self.cpu.write_n_flag(false);
                    self.cpu.write_h_flag((reg & 0xF) == 0xF);
                }
                // increment 16-bit register
                R16_BC | R16_DE | R16_HL | R16_SP => {
                    let reg = self.cpu.read_r16(&x);
                    let result = reg.wrapping_add(1);
                    self.cpu.write_r16(&x, result);

                    // no flags for 16-bit increment
                }
                // memory at address in hl
                PTR(ptr) => match *ptr {
                    R16_HL => {
                        let address = self.cpu.read_r16(&R16_HL);
                        let byte = self.memory.read_byte(address);
                        let result = byte.wrapping_add(1);
                        self.memory.write_byte(address, result)?;

                        // inc flags : Z 0 H -
                        self.cpu.write_z_flag(result == 0);
                        self.cpu.write_n_flag(false);
                        self.cpu.write_h_flag((byte & 0xF) == 0xF);
                    }
                    _ => panic!("(CRITICAL) INC : ILLEGAL POINTER {ptr} at {pc:#06X}"),
                },

                _ => panic!("(CRITICAL) INC : ILLEGAL OPERAND {x} at {pc:#06X}"),
            },
            Operation::DEC { x } => match x {
                // decrement 8-bit register
                R8_A | R8_B | R8_C | R8_D | R8_E | R8_H | R8_L => {
                    let reg = self.cpu.read_r8(&x);
                    let result = reg.wrapping_sub(1);
                    self.cpu.write_r8(&x, result);

                    // dec flags : Z 1 H -
                    self.cpu.write_z_flag(result == 0);
                    self.cpu.write_n_flag(true);
                    self.cpu.write_h_flag((reg & 0xF) == 0);
                }
                // decrement 16-bit register
                R16_BC | R16_DE | R16_HL | R16_SP => {
                    let reg = self.cpu.read_r16(&x);
                    let result = reg.wrapping_sub(1);
                    self.cpu.write_r16(&x, result);
                }
                // memory at address in hl
                PTR(ptr) => match *ptr {
                    R16_HL => {
                        let address = self.cpu.read_r16(&R16_HL);
                        let byte = self.memory.read_byte(address);
                        let result = byte.wrapping_sub(1);
                        self.memory.write_byte(address, result)?;

                        // dec flags : Z 1 H -
                        self.cpu.write_z_flag(result == 0);
                        self.cpu.write_n_flag(true);
                        self.cpu.write_h_flag((byte & 0xF) == 0);
                    }
                    _ => panic!("(CRITICAL) DEC : ILLEGAL POINTER {ptr} at {pc:#06X}"),
                },

                _ => panic!("(CRITICAL) DEC : ILLEGAL OPERAND {x} at {pc:#06X}"),
            },
            Operation::ADD { x, y } => {
                // add either does a + y and stores the result in a (8 bits)
                // or HL + y (16 bits)
                match x {
                    Operand::R8_A => {
                        let value = match y {
                            // add 8-bit register
                            R8_A | R8_B | R8_C | R8_D | R8_E | R8_H | R8_L => self.cpu.read_r8(&y),
                            // add imm8
                            IMM8(imm8) => imm8,
                            // add from memory with pointer in hl
                            PTR(ptr) => match *ptr {
                                R16_HL => {
                                    let hl = self.cpu.read_r16(&ptr);
                                    self.memory.read_byte(hl)
                                }
                                _ => panic!("(CRITICAL) ADD : ILLEGAL POINTER {ptr} at {pc:#06X}"),
                            },
                            _ => panic!("(CRITICAL) ADD : ILLEGAL SECOND OPERAND {y} at {pc:#06X}"),
                        };

                        let a = self.cpu.read_a_register();
                        let result = a.wrapping_add(value);
                        self.cpu.write_a_register(result);

                        // flags : z 0 h c
                        self.cpu.write_z_flag(result == 0);
                        self.cpu.write_n_flag(false);
                        self.cpu.write_h_flag((a & 0xF) + (value & 0xF) > 0xF);
                        self.cpu.write_c_flag(a < value);
                    }
                    Operand::R16_HL => {
                        let value = match y {
                            // add 16-bit register
                            R16_BC | R16_DE | R16_HL | R16_SP => self.cpu.read_r16(&y),
                            _ => panic!("(CRITICAL) ADD : ILLEGAL SECOND OPERAND {y} at {pc:#06X}"),
                        };

                        let hl = self.cpu.read_hl_register();
                        let result = hl.wrapping_add(value);
                        self.cpu.write_hl_register(result);

                        // flags : - 0 h c
                        self.cpu.write_n_flag(false);
                        self.cpu
                            .write_h_flag((hl & 0xFFF) + (value & 0xFFF) > 0xFFF);
                        self.cpu.write_c_flag(hl < value);
                    }
                    // signed SP add is also a thing apparently
                    /* Operand::R16_SP => {
                        let sp = self.cpu.read_stack_pointer();
                        let value = match y {
                            IMM8_SIGNED(imm8) => imm8,
                            _ => panic!("(CRITICAL) ADD : ILLEGAL SECOND OPERAND {y} at {pc:#06X}"),
                        };

                        self.cpu.offset_stack_pointer(value);

                        // flags : 0 0 h c
                        self.cpu.write_z_flag(false);
                        self.cpu.write_n_flag(false);
                    } */
                    _ => panic!("(CRITICAL) ADD : ILLEGAL FIRST OPERAND {x} at {pc:#06X}"),
                }
            }
            Operation::ADC { y } => {
                // like add, but also adds the carry flag (hence the "c")

                let value = match y {
                    // add 8-bit register
                    R8_A | R8_B | R8_C | R8_D | R8_E | R8_H | R8_L => self.cpu.read_r8(&y),
                    // add imm8
                    IMM8(imm8) => imm8,
                    // add from memory with pointer in hl
                    PTR(ptr) => match *ptr {
                        R16_HL => {
                            let hl = self.cpu.read_r16(&ptr);
                            self.memory.read_byte(hl)
                        }
                        _ => panic!("(CRITICAL) ADC : ILLEGAL POINTER {ptr} at {pc:#06X}"),
                    },
                    _ => panic!("(CRITICAL) ADC : ILLEGAL SECOND OPERAND {y} at {pc:#06X}"),
                };

                let a = self.cpu.read_a_register();
                let carry = if self.cpu.read_c_flag() { 1 } else { 0 };
                let result = a.wrapping_add(value).wrapping_add(carry);
                self.cpu.write_a_register(result);

                // flags : z 0 h c
                self.cpu.write_z_flag(result == 0);
                self.cpu.write_n_flag(false);
                self.cpu.write_h_flag(
                    // lots of "as usize" here bc we need to be careful that value + carry doesn't cause an overflow
                    // that happens when value = 255
                    // FIXME : there is probably a better way to do that
                    ((a as usize) & 0xF) + ((value as usize + carry as usize) & 0xF) > 0xF,
                );
                self.cpu
                    .write_c_flag((a as usize) < (value as usize + carry as usize));
            }
            Operation::SUB { y } => {
                // sub does a - y and stores the result in a
                let a = self.cpu.read_a_register();
                let value = match y {
                    // subtract 8-bit register
                    R8_A | R8_B | R8_C | R8_D | R8_E | R8_H | R8_L => self.cpu.read_r8(&y),
                    // subtract imm8
                    IMM8(imm8) => imm8,
                    // subtract from memory with pointer in hl
                    PTR(ptr) => match *ptr {
                        R16_HL => {
                            let hl = self.cpu.read_r16(&ptr);
                            self.memory.read_byte(hl)
                        }
                        _ => panic!("(CRITICAL) SUB : ILLEGAL POINTER {ptr} at {pc:#06X}"),
                    },
                    _ => panic!("(CRITICAL) SUB : ILLEGAL SECOND OPERAND {y} at {pc:#06X}"),
                };

                let result = a.wrapping_sub(value);
                self.cpu.write_a_register(result);

                // flags : z 1 h c
                self.cpu.write_z_flag(result == 0);
                self.cpu.write_n_flag(true);
                self.cpu.write_h_flag((a & 0xF) < (value & 0xF));
                self.cpu.write_c_flag(a < value);
            }
            Operation::SBC { y } => {
                // like sub, but also subtracts the carry flag (hence the "c")

                let value = match y {
                    // add 8-bit register
                    R8_A | R8_B | R8_C | R8_D | R8_E | R8_H | R8_L => self.cpu.read_r8(&y),
                    // add imm8
                    IMM8(imm8) => imm8,
                    // add from memory with pointer in hl
                    PTR(ptr) => match *ptr {
                        R16_HL => {
                            let hl = self.cpu.read_r16(&ptr);
                            self.memory.read_byte(hl)
                        }
                        _ => panic!("(CRITICAL) SBC : ILLEGAL POINTER {ptr} at {pc:#06X}"),
                    },
                    _ => panic!("(CRITICAL) SBC : ILLEGAL SECOND OPERAND {y} at {pc:#06X}"),
                };

                let a = self.cpu.read_a_register();
                let carry = if self.cpu.read_c_flag() { 1 } else { 0 };
                let result = a.wrapping_sub(value).wrapping_sub(carry);
                self.cpu.write_a_register(result);

                // flags : z 0 h c
                self.cpu.write_z_flag(result == 0);
                self.cpu.write_n_flag(true);
                self.cpu.write_h_flag(
                    ((a & 0xF).wrapping_sub(value & 0xF).wrapping_sub(carry)) & 0x10 == 0x10,
                );
                self.cpu
                    .write_c_flag((a as usize) < (value as usize + carry as usize));
            }
            Operation::XOR { y } => {
                // xor is always done with the a register as first operand (x)
                let a = self.cpu.read_r8(&R8_A);
                let other = match y {
                    // second operand can only be another 8-bit register or pointer in hl
                    R8_A | R8_B | R8_C | R8_D | R8_E | R8_H | R8_L => self.cpu.read_r8(&y),
                    PTR(ptr) => match *ptr {
                        R16_HL => self.memory.read_byte(self.cpu.read_r16(&R16_HL)),
                        _ => panic!("(CRITICAL) XOR : ILLEGAL POINTER {ptr} at {pc:#06X}"),
                    },
                    IMM8(imm8) => imm8,
                    _ => panic!("(CRITICAL) XOR : ILLEGAL SECOND OPERAND {y:?} at {pc:#06X}"),
                };

                self.cpu.write_r8(&R8_A, a ^ other);

                // xor flags : Z 0 0 0
                self.cpu.write_z_flag(self.cpu.read_r8(&R8_A) == 0);
                self.cpu.write_n_flag(false);
                self.cpu.write_h_flag(false);
                self.cpu.write_c_flag(false);
            }
            Operation::OR { y } => {
                // or is always done with the a register as first operand (x)
                let a = self.cpu.read_r8(&R8_A);
                let other = match y {
                    // second operand can only be another 8-bit register or pointer in hl
                    R8_A | R8_B | R8_C | R8_D | R8_E | R8_H | R8_L => self.cpu.read_r8(&y),
                    PTR(ptr) => match *ptr {
                        R16_HL => self.memory.read_byte(self.cpu.read_r16(&R16_HL)),
                        _ => panic!("(CRITICAL) OR : ILLEGAL POINTER {ptr} at {pc:#06X}"),
                    },
                    IMM8(imm8) => imm8,
                    _ => panic!("(CRITICAL) OR : ILLEGAL SECOND OPERAND {y:?} at {pc:#06X}"),
                };

                self.cpu.write_r8(&R8_A, a | other);

                // xor flags : Z 0 0 0
                self.cpu.write_z_flag(self.cpu.read_r8(&R8_A) == 0);
                self.cpu.write_n_flag(false);
                self.cpu.write_h_flag(false);
                self.cpu.write_c_flag(false);
            }
            Operation::AND { y } => {
                // and is always done with the a register as first operand (x)
                let a = self.cpu.read_r8(&R8_A);
                let other = match y {
                    // second operand can only be another 8-bit register or pointer in hl
                    R8_A | R8_B | R8_C | R8_D | R8_E | R8_H | R8_L => self.cpu.read_r8(&y),
                    PTR(ptr) => match *ptr {
                        R16_HL => self.memory.read_byte(self.cpu.read_r16(&R16_HL)),
                        _ => panic!("(CRITICAL) AND : ILLEGAL POINTER {ptr} at {pc:#06X}"),
                    },
                    IMM8(imm8) => imm8,
                    _ => panic!("(CRITICAL) AND : ILLEGAL SECOND OPERAND {y:?} at {pc:#06X}"),
                };

                self.cpu.write_r8(&R8_A, a & other);

                // xor flags : Z 0 1 0
                self.cpu.write_z_flag(self.cpu.read_r8(&R8_A) == 0);
                self.cpu.write_n_flag(false);
                self.cpu.write_h_flag(true);
                self.cpu.write_c_flag(false);
            }
            Operation::BIT { bit, src } => {
                // test bit in register / memory, set the zero flag to complement of bit

                let byte = match src {
                    // test bit in 8-bit register
                    R8_A | R8_B | R8_C | R8_D | R8_E | R8_H | R8_L => self.cpu.read_r8(&src),
                    // test bit in memory
                    PTR(ptr) => match *ptr {
                        R16_HL => self.memory.read_byte(self.cpu.read_r16(&R16_HL)),
                        _ => panic!("(CRITICAL) BIT : ILLEGAL POINTER {ptr:?} at {pc:#06X}"),
                    },
                    _ => panic!("(CRITICAL) BIT : ILLEGAL SRC {src:?} at {pc:#06X}"),
                };

                // bit instruction flags : Z 0 1 -
                self.cpu.write_z_flag((byte >> bit) & 1 == 0); // true if bit is 0, false if bit is 1
                self.cpu.write_n_flag(false);
                self.cpu.write_h_flag(true);
            }
            Operation::RES { bit, x } => {
                // set bit in register / memory to zero
                match x {
                    R8_A | R8_B | R8_C | R8_D | R8_E | R8_H | R8_L => {
                        let byte = self.cpu.read_r8(&x);
                        self.cpu.write_r8(&x, byte & !(1 << bit));
                    }
                    PTR(ptr) => match *ptr {
                        R16_HL => {
                            let address = self.cpu.read_hl_register();
                            let byte = self.memory.read_byte(address);
                            self.memory.write_byte(address, byte & !(1 << bit))?;
                        }
                        _ => panic!("(CRITICAL) RES : ILLEGAL POINTER {ptr:?} at {pc:#06X}"),
                    },
                    _ => panic!("(CRITICAL) RES : ILLEGAL OPERAND {x:?} at {pc:#06X}"),
                }

                // no flags
            }
            Operation::SET { bit, x } => {
                // set bit in register / memory to 1
                match x {
                    R8_A | R8_B | R8_C | R8_D | R8_E | R8_H | R8_L => {
                        let byte = self.cpu.read_r8(&x);
                        self.cpu.write_r8(&x, byte | (1 << bit));
                    }
                    PTR(ptr) => match *ptr {
                        R16_HL => {
                            let address = self.cpu.read_hl_register();
                            let byte = self.memory.read_byte(address);
                            self.memory.write_byte(address, byte | (1 << bit))?;
                        }
                        _ => panic!("(CRITICAL) RES : ILLEGAL POINTER {ptr:?} at {pc:#06X}"),
                    },
                    _ => panic!("(CRITICAL) RES : ILLEGAL OPERAND {x:?} at {pc:#06X}"),
                }
                // no flags
            }
            Operation::SWAP { x } => {
                // swaps the upper 4 bits and the lower 4 ones

                match x {
                    R8_A | R8_B | R8_C | R8_D | R8_E | R8_H | R8_L => {
                        let register = self.cpu.read_r8(&x);
                        let lower_to_upper = register << 4;
                        let upper_to_lower = register >> 4;
                        self.cpu.write_r8(&x, lower_to_upper | upper_to_lower);

                        self.cpu
                            .write_z_flag((lower_to_upper | upper_to_lower) == 0);
                    }
                    PTR(ptr) => match *ptr {
                        R16_HL => {
                            let hl = self.cpu.read_hl_register();
                            let value = self.memory.read_byte(hl);
                            let lower_to_upper = value << 4;
                            let upper_to_lower = value >> 4;
                            self.memory
                                .write_byte(hl, lower_to_upper | upper_to_lower)?;

                            self.cpu
                                .write_z_flag((lower_to_upper | upper_to_lower) == 0);
                        }
                        _ => panic!("(CRITICAL) SWAP : ILLEGAL POINTER {ptr} at {pc:#06X}"),
                    },
                    _ => panic!("(CRITICAL) SWAP : ILLEGAL OPERAND {x} at {pc:#06X}"),
                }

                // flags : z 0 0 0

                self.cpu.write_n_flag(false);
                self.cpu.write_h_flag(false);
                self.cpu.write_c_flag(false);
            }
            Operation::SRL { x } => {
                // shift right logical
                // for flags, see https://rgbds.gbdev.io/docs/v0.8.0/gbz80.7#SRL_r8
                match x {
                    R8_B | R8_C | R8_D | R8_E | R8_H | R8_L | R8_A => {
                        let register = self.cpu.read_r8(&x);
                        let carry = register & 1 == 1;
                        let result = register >> 1;
                        self.cpu.write_r8(&x, result);

                        // flags : z 0 0 c
                        self.cpu.write_z_flag(result == 0);
                        self.cpu.write_n_flag(false);
                        self.cpu.write_h_flag(false);
                        self.cpu.write_c_flag(carry);
                    }
                    PTR(ptr) => {
                        match *ptr {
                            R16_HL => {
                                let address = self.cpu.read_r16(&R16_HL);
                                let value = self.memory.read_byte(address);
                                let carry = value & 1 == 1;
                                let result = value >> 1;
                                self.memory.write_byte(address, result)?;

                                // flags : z 0 0 c
                                self.cpu.write_z_flag(result == 0);
                                self.cpu.write_n_flag(false);
                                self.cpu.write_h_flag(false);
                                self.cpu.write_c_flag(carry);
                            }
                            _ => panic!("(CRITICAL) SRL : ILLEGAL POINTER {ptr} at {pc:#06X}"),
                        }
                    }
                    _ => panic!("(CRITICAL) SRL : ILLEGAL OPERAND {x} at {pc:#06X}"),
                }
            }
            Operation::SLA { x } => {
                // shift left arithmetic
                // for flags, see https://rgbds.gbdev.io/docs/v0.8.0/gbz80.7#SLA_r8
                match x {
                    R8_B | R8_C | R8_D | R8_E | R8_H | R8_L | R8_A => {
                        let register = self.cpu.read_r8(&x);
                        let carry = register >> 7 == 1;
                        let result = register << 1;
                        self.cpu.write_r8(&x, result);

                        // flags : z 0 0 c
                        self.cpu.write_z_flag(result == 0);
                        self.cpu.write_n_flag(false);
                        self.cpu.write_h_flag(false);
                        self.cpu.write_c_flag(carry);
                    }
                    PTR(ptr) => {
                        match *ptr {
                            R16_HL => {
                                let address = self.cpu.read_r16(&R16_HL);
                                let value = self.memory.read_byte(address);
                                let carry = value >> 7 == 1;
                                let result = value << 1;
                                self.memory.write_byte(address, result)?;

                                // flags : z 0 0 c
                                self.cpu.write_z_flag(result == 0);
                                self.cpu.write_n_flag(false);
                                self.cpu.write_h_flag(false);
                                self.cpu.write_c_flag(carry);
                            }
                            _ => panic!("(CRITICAL) SRL : ILLEGAL POINTER {ptr} at {pc:#06X}"),
                        }
                    }
                    _ => panic!("(CRITICAL) SRL : ILLEGAL OPERAND {x} at {pc:#06X}"),
                }
            }
            Operation::RL { x } => {
                match x {
                    // rotate 8-bit register
                    R8_A | R8_B | R8_C | R8_D | R8_E | R8_H | R8_L => {
                        let mut to_rotate = self.cpu.read_r8(&x);

                        // b7 to carry
                        let previous_carry: u8 = if self.cpu.read_c_flag() { 1 } else { 0 };
                        self.cpu.write_c_flag((to_rotate >> 7) & 1 == 1);

                        // rotate number left with carry
                        to_rotate <<= 1;
                        to_rotate |= previous_carry;

                        // write back the number
                        self.cpu.write_r8(&x, to_rotate);

                        // flags : z 0 0 c
                        self.cpu.write_z_flag(to_rotate == 0);
                        self.cpu.write_n_flag(false);
                        self.cpu.write_h_flag(false);
                    }
                    // rotate memory byte
                    PTR(ptr) => match *ptr {
                        R16_HL => {
                            let address = self.cpu.read_r16(&ptr);
                            let mut to_rotate = self.memory.read_byte(address);

                            // b7 to carry
                            let previous_carry: u8 = if self.cpu.read_c_flag() { 1 } else { 0 };
                            self.cpu.write_c_flag((to_rotate >> 7) & 1 == 1);

                            // rotate number left with carry
                            to_rotate <<= 1;
                            to_rotate |= previous_carry;

                            // write back the number
                            self.memory.write_byte(address, to_rotate)?;

                            // flags : z 0 0 c
                            self.cpu.write_z_flag(to_rotate == 0);
                            self.cpu.write_n_flag(false);
                            self.cpu.write_h_flag(false);
                        }
                        _ => panic!("(CRITICAL) RL : ILLEGAL POINTER {ptr} at {pc:#06X}"),
                    },
                    _ => panic!("(CRITICAL) RL : ILLEGAL OPERAND {x} at {pc:#06X}"),
                }
            }
            Operation::RR { x } => {
                match x {
                    // rotate 8-bit register
                    R8_A | R8_B | R8_C | R8_D | R8_E | R8_H | R8_L => {
                        let mut to_rotate = self.cpu.read_r8(&x);

                        // b0 to carry
                        let previous_carry: u8 = if self.cpu.read_c_flag() { 1 } else { 0 };
                        self.cpu.write_c_flag((to_rotate) & 1 == 1);

                        // rotate number right with carry
                        to_rotate >>= 1;
                        to_rotate |= previous_carry << 7;

                        // write back the number
                        self.cpu.write_r8(&x, to_rotate);

                        // flags : z 0 0 c
                        self.cpu.write_z_flag(to_rotate == 0);
                        self.cpu.write_n_flag(false);
                        self.cpu.write_h_flag(false);
                    }
                    // rotate memory byte
                    PTR(ptr) => match *ptr {
                        R16_HL => {
                            let address = self.cpu.read_r16(&ptr);
                            let mut to_rotate = self.memory.read_byte(address);

                            // b0 to carry
                            let previous_carry: u8 = if self.cpu.read_c_flag() { 1 } else { 0 };
                            self.cpu.write_c_flag((to_rotate) & 1 == 1);

                            // rotate number right with carry
                            to_rotate >>= 1;
                            to_rotate |= previous_carry << 7;

                            // write back the number
                            self.memory.write_byte(address, to_rotate)?;

                            // flags : z 0 0 c
                            self.cpu.write_z_flag(to_rotate == 0);
                            self.cpu.write_n_flag(false);
                            self.cpu.write_h_flag(false);
                        }
                        _ => panic!("(CRITICAL) RR : ILLEGAL POINTER {ptr} at {pc:#06X}"),
                    },
                    _ => panic!("(CRITICAL) RR : ILLEGAL OPERAND {x} at {pc:#06X}"),
                }
            }
            Operation::RLC { x } => {
                match x {
                    // rotate 8-bit register
                    R8_A | R8_B | R8_C | R8_D | R8_E | R8_H | R8_L => {
                        let mut to_rotate = self.cpu.read_r8(&x);

                        // b7 to carry
                        let previous_b7: u8 = (to_rotate >> 7) & 1;
                        self.cpu.write_c_flag(previous_b7 == 1);

                        // rotate number left with carry
                        to_rotate <<= 1;
                        to_rotate |= previous_b7;

                        // write back the number
                        self.cpu.write_r8(&x, to_rotate);

                        // flags : z 0 0 c
                        self.cpu.write_z_flag(to_rotate == 0);
                        self.cpu.write_n_flag(false);
                        self.cpu.write_h_flag(false);
                    }
                    // rotate memory byte
                    PTR(ptr) => match *ptr {
                        R16_HL => {
                            let address = self.cpu.read_r16(&ptr);
                            let mut to_rotate = self.memory.read_byte(address);

                            // b7 to carry
                            let previous_b7: u8 = (to_rotate >> 7) & 1;
                            self.cpu.write_c_flag(previous_b7 == 1);

                            // rotate number left with carry
                            to_rotate <<= 1;
                            to_rotate |= previous_b7;

                            // write back the number
                            self.memory.write_byte(address, to_rotate)?;

                            // flags : z 0 0 c
                            self.cpu.write_z_flag(to_rotate == 0);
                            self.cpu.write_n_flag(false);
                            self.cpu.write_h_flag(false);
                        }
                        _ => panic!("(CRITICAL) RLC : ILLEGAL POINTER {ptr} at {pc:#06X}"),
                    },
                    _ => panic!("(CRITICAL) RLC : ILLEGAL OPERAND {x} at {pc:#06X}"),
                }
            }
            Operation::RRC { x } => {
                match x {
                    // rotate 8-bit register
                    R8_A | R8_B | R8_C | R8_D | R8_E | R8_H | R8_L => {
                        let mut to_rotate = self.cpu.read_r8(&x);

                        // b0 to carry
                        let previous_b0: u8 = (to_rotate) & 1;
                        self.cpu.write_c_flag(previous_b0 == 1);

                        // rotate number left with carry
                        to_rotate >>= 1;
                        to_rotate |= previous_b0 << 7;

                        // write back the number
                        self.cpu.write_r8(&x, to_rotate);

                        // flags : z 0 0 c
                        self.cpu.write_z_flag(to_rotate == 0);
                        self.cpu.write_n_flag(false);
                        self.cpu.write_h_flag(false);
                    }
                    // rotate memory byte
                    PTR(ptr) => match *ptr {
                        R16_HL => {
                            let address = self.cpu.read_r16(&ptr);
                            let mut to_rotate = self.memory.read_byte(address);

                            // b0 to carry
                            let previous_b0: u8 = (to_rotate) & 1;
                            self.cpu.write_c_flag(previous_b0 == 1);

                            // rotate number left with carry
                            to_rotate >>= 1;
                            to_rotate |= previous_b0 << 7;

                            // write back the number
                            self.memory.write_byte(address, to_rotate)?;

                            // flags : z 0 0 c
                            self.cpu.write_z_flag(to_rotate == 0);
                            self.cpu.write_n_flag(false);
                            self.cpu.write_h_flag(false);
                        }
                        _ => panic!("(CRITICAL) RRC : ILLEGAL POINTER {ptr} at {pc:#06X}"),
                    },
                    _ => panic!("(CRITICAL) RRC : ILLEGAL OPERAND {x} at {pc:#06X}"),
                }
            }
            Operation::RLA => {
                // rotate a register
                let mut to_rotate = self.cpu.read_r8(&R8_A);

                // b7 to carry
                let previous_carry: u8 = if self.cpu.read_c_flag() { 1 } else { 0 };
                self.cpu.write_c_flag((to_rotate >> 7) & 1 == 1);

                // rotate number left with carry
                to_rotate <<= 1;
                to_rotate |= previous_carry;

                // write back the number
                self.cpu.write_r8(&R8_A, to_rotate);

                // flags : 0 0 0 c
                self.cpu.write_z_flag(false);
                self.cpu.write_n_flag(false);
                self.cpu.write_h_flag(false);
            }
            Operation::RRA => {
                // rotate a register
                let mut to_rotate = self.cpu.read_r8(&R8_A);

                // b0 to carry
                let previous_carry: u8 = if self.cpu.read_c_flag() { 1 } else { 0 };
                self.cpu.write_c_flag((to_rotate) & 1 == 1);

                // rotate number left with carry
                to_rotate >>= 1;
                to_rotate |= previous_carry << 7;

                // write back the number
                self.cpu.write_r8(&R8_A, to_rotate);

                // flags : 0 0 0 c
                self.cpu.write_z_flag(false);
                self.cpu.write_n_flag(false);
                self.cpu.write_h_flag(false);
            }
            Operation::RLCA => {
                let mut to_rotate = self.cpu.read_r8(&R8_A);

                // b7 to carry
                let previous_b7: u8 = (to_rotate >> 7) & 1;
                self.cpu.write_c_flag(previous_b7 == 1);

                // rotate number left with carry
                to_rotate <<= 1;
                to_rotate |= previous_b7;

                // write back the number
                self.cpu.write_r8(&R8_A, to_rotate);

                // flags : 0 0 0 c
                self.cpu.write_z_flag(false);
                self.cpu.write_n_flag(false);
                self.cpu.write_h_flag(false);
            }
            Operation::RRCA => {
                // rotate 8-bit register

                let mut to_rotate = self.cpu.read_r8(&R8_A);

                // b0 to carry
                let previous_b0: u8 = (to_rotate) & 1;
                self.cpu.write_c_flag(previous_b0 == 1);

                // rotate number left with carry
                to_rotate >>= 1;
                to_rotate |= previous_b0 << 7;

                // write back the number
                self.cpu.write_r8(&R8_A, to_rotate);

                // flags : 0 0 0 c
                self.cpu.write_z_flag(false);
                self.cpu.write_n_flag(false);
                self.cpu.write_h_flag(false);
            }
            Operation::DAA => {
                // https://blog.ollien.com/posts/gb-daa/
                let mut offset = 0;
                let a = self.cpu.read_a_register();
                let half_carry = self.cpu.read_h_flag();
                let carry = self.cpu.read_c_flag();
                let n = self.cpu.read_n_flag();

                if (!n && a & 0xF > 0x9) || half_carry {
                    offset |= 0x06;
                }

                if (!n && a > 0x99) || carry {
                    offset |= 0x60;
                }

                self.cpu.write_a_register(if !n {
                    a.wrapping_add(offset)
                } else {
                    a.wrapping_sub(offset)
                });

                // flags : z - 0 c
                self.cpu.write_z_flag(self.cpu.read_a_register() == 0);
                self.cpu.write_n_flag(false);
                self.cpu
                    .write_c_flag(carry || (!n && self.cpu.read_a_register() > 0x99));
            }
            Operation::JR { offset_oprd } => {
                let offset = match offset_oprd {
                    IMM8_SIGNED(offset) => offset,
                    _ => panic!("(CRITICAL) JR : ILLEGAL OFFSET {offset_oprd} at {pc:#06X}"),
                };

                self.cpu.offset_program_counter(offset);
            }
            Operation::JR_CC { cc, offset_oprd } => {
                let should_jump = self.cpu.get_cc(&cc);
                if should_jump {
                    let offset = match offset_oprd {
                        IMM8_SIGNED(offset) => offset,
                        _ => panic!("(CRITICAL) JR_CC : ILLEGAL OFFSET {offset_oprd} at {pc:#06X}"),
                    };

                    cycles_elapsed = instr.branch_cycles.unwrap();

                    self.cpu.offset_program_counter(offset);
                }
            }
            Operation::JP { addr } => {
                // jp instruction only takes either an imm16 or the hl register
                let address = match addr {
                    IMM16(imm16) => imm16,
                    R16_HL => self.cpu.read_hl_register(),
                    _ => panic!("(CRITICAL) JP : ILLEGAL ADDRESS {addr} at {pc:#06X}"),
                };

                self.cpu.write_program_counter(address);
            }
            Operation::JP_CC { cc, addr } => {
                if self.cpu.get_cc(&cc) {
                    // jp instruction only takes either an imm16 or the hl register
                    let address = match addr {
                        IMM16(imm16) => imm16,
                        R16_HL => self.cpu.read_hl_register(),
                        _ => panic!("(CRITICAL) JP CC : ILLEGAL ADDRESS {addr} at {pc:#06X}"),
                    };

                    self.cpu.write_program_counter(address);

                    cycles_elapsed = instr.branch_cycles.unwrap();
                }
            }
            Operation::CALL { proc } => {
                let address = match proc {
                    IMM16(imm16) => imm16,
                    _ => {
                        panic!("(CRITICAL) CALL : ILLEGAL PROCEDURE ADDRESS {proc} at {pc:#06X}")
                    }
                };

                // push the return address to the stack
                let current_pc = self.cpu.read_program_counter();
                self.push_word(current_pc)?;

                // jump to the procedure
                self.cpu.write_program_counter(address);
            }
            Operation::CALL_CC { cc, proc } => {
                let address = match proc {
                    IMM16(imm16) => imm16,
                    _ => {
                        panic!("(CRITICAL) CALL CC : ILLEGAL PROCEDURE ADDRESS {proc} at {pc:#06X}")
                    }
                };

                if self.cpu.get_cc(&cc) {
                    // push the return address to the stack
                    let current_pc = self.cpu.read_program_counter();
                    self.push_word(current_pc)?;

                    // jump to the procedure
                    self.cpu.write_program_counter(address);

                    // update the cycles elapsed since we branched
                    cycles_elapsed = instr.branch_cycles.unwrap();
                }
            }
            Operation::RST { addr } => {
                // rst is like call, but only for a few fixed addresses
                let address = match addr {
                    IMM16(imm16) => imm16,
                    _ => {
                        panic!("(CRITICAL) RST : ILLEGAL ADDRESS {addr} at {pc:#06X}")
                    }
                };

                // push the return address to the stack
                let current_pc = self.cpu.read_program_counter();
                self.push_word(current_pc)?;

                // jump to the procedure
                self.cpu.write_program_counter(address);
            }
            Operation::RET => {
                let return_address = self.pop_word();

                // jump to where the procedure was called
                self.cpu.write_program_counter(return_address);
            }
            Operation::RET_CC { cc } => {
                let should_return = self.cpu.get_cc(&cc);

                if should_return {
                    let return_address = self.pop_word();

                    // jump to where the procedure was called
                    self.cpu.write_program_counter(return_address);
                }
            }
            Operation::RETI => {
                let return_address = self.pop_word();

                // jump to where the procedure was called
                self.cpu.write_program_counter(return_address);

                // re-enable interrupts
                self.cpu.enable_interrupts();
            }
            Operation::PUSH { reg } => {
                let to_push = match reg {
                    R16_BC | R16_DE | R16_HL | R16_AF => self.cpu.read_r16(&reg),
                    _ => panic!("(CRITICAL) PUSH : ILLEGAL OPERAND {reg} at {pc:#06X}"),
                };

                self.push_word(to_push)?;
            }

            Operation::POP { reg } => {
                match reg {
                    R16_BC | R16_DE | R16_HL | R16_AF => {
                        let word = self.pop_word();
                        self.cpu.write_r16(&reg, word);
                    }
                    _ => panic!("(CRITICAL) POP : ILLEGAL OPERAND {reg} at {pc:#06X}"),
                };
            }

            Operation::CP { y } => {
                // compare register a with another value
                let a = self.cpu.read_r8(&R8_A);
                let other = match y {
                    // second operand can be another 8-bit register, imm8 or pointer in hl
                    R8_A | R8_B | R8_C | R8_D | R8_E | R8_H | R8_L => self.cpu.read_r8(&y),
                    IMM8(imm8) => imm8,
                    PTR(ptr) => match *ptr {
                        R16_HL => self.memory.read_byte(self.cpu.read_r16(&R16_HL)),
                        _ => panic!("(CRITICAL) CP : ILLEGAL POINTER {ptr} at {pc:#06X}"),
                    },
                    _ => panic!("(CRITICAL) CP : ILLEGAL SECOND OPERAND {y} at {pc:#06X}"),
                };

                // cp flags : Z 1 H C
                self.cpu.write_z_flag(a == other);
                self.cpu.write_n_flag(true);
                self.cpu.write_h_flag((a & 0xF) < (other & 0xF));
                self.cpu.write_c_flag(a < other);
            }
            Operation::DI => {
                self.cpu.disable_interrupts();
                warn!("DI : INTERRUPTS DISABLED");
            }
            Operation::EI => {
                self.cpu.enable_interrupts();
                warn!("EI : INTERRUPTS ENABLED")
            }
            Operation::CPL => {
                let accumulator = self.cpu.read_a_register();
                self.cpu.write_a_register(!accumulator);
                // flags : - 1 1 -
                self.cpu.write_n_flag(true);
                self.cpu.write_h_flag(true);
            }

            Operation::HALT => {
                // https://rgbds.gbdev.io/docs/v0.8.0/gbz80.7#HALT
                // FIXME : there is no way this is accurate
                if self.cpu.interrupts_enabled() {
                    self.halted = true;
                    info!("HALTED !");
                } else {
                    if !self.memory.interrupt_pending_and_enabled() {
                        self.halted = true;
                        info!("HALTED !");
                    } else {
                        // so called "halt bug"
                        warn!("HALT BUG : NOT IMPLEMENTED")
                    }
                }
            }
            _ => {
                return Err(EmulationError {
                    ty: EmulationErrorType::UnhandledInstructionExec(instr),
                    pc: Some(pc),
                });
            }
        }

        return Ok(cycles_elapsed);
    }
    // utilities common to multiple opcodes

    /* fn push_byte(&mut self, byte: u8) {
        // decrement stack pointer
        self.cpu.offset_stack_pointer(-1);

        // write byte
        self.memory.write_byte(self.cpu.read_stack_pointer(), byte);
    } */

    fn push_word(&mut self, word: u16) -> Result<(), EmulationError> {
        // decrement stack pointer
        self.cpu.offset_stack_pointer(-2);

        // write word
        self.memory
            .write_word(self.cpu.read_stack_pointer(), word)?;

        Ok(())
    }

    /* fn pop_byte(&mut self) -> u8 {
        // read byte
        let byte = self.memory.read_byte(self.cpu.read_stack_pointer());

        // decrement stack pointer
        self.cpu.offset_stack_pointer(1);

        return byte;
    } */

    fn pop_word(&mut self) -> u16 {
        // read word
        let word = self.memory.read_word(self.cpu.read_stack_pointer());

        // decrement stack pointer
        self.cpu.offset_stack_pointer(2);

        return word;
    }
}
