use std::convert::TryInto;
use std::fs::File;
use std::io::Read;
use std::io::{BufReader, ErrorKind};

use rand;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use std::time::Duration;

const CELL_W: u32 = 16; // in pixels
const CELL_H: u32 = 16; // in pixels
const CHIP8_DISP_W: u32 = 64; // in cells (chip8 pixels)
const CHIP8_DISP_H: u32 = 32; // in cells (chip8 pixels)
const DEBUG: bool = true;
const INSTRUCTIONS_PER_TICK: u32 = 10;
const FPS: u32 = 3;
const RAM_OFFSET: u16 = 0x0200; // offset in the ram where user programs start
const FONT: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

type Address = u16;
type Greg = u8;

#[derive(Default)]
struct Chip8Regs {
    vx: [Greg; 16],
    dt: u8,
    st: u8,
    i: Address,
    pc: Address,
    sp: i8,
}

struct Chip8 {
    registers: Chip8Regs,
    ram: [u8; 0x1000],
    vram: [u64; CHIP8_DISP_H as usize],
    stack: [u16; 16],
    keyboard: u16,
    previous_keyboard: u16,
}

impl Chip8 {
    fn new(path: String) -> Chip8 {
        let mut chip8 = Chip8 {
            registers: Chip8Regs::default(),
            ram: [0u8; 0x1000],
            vram: [0u64; CHIP8_DISP_H as usize],
            stack: [0u16; 16],
            keyboard: 0x00,
            previous_keyboard: 0x00,
        };
        // load font into RAM
        for i in 0..80 {
            chip8.ram[i] = FONT[i];
        }

        let file = File::open(path).expect("Cannot Read ROM");
        let mut buf = BufReader::new(file);

        let mut rom_bytes = [0; (0x1000 - RAM_OFFSET as usize)];
        match buf.read(&mut rom_bytes) {
            Ok(0) => (println! {"No bytes read from ROM!"}),
            Ok(n) => {
                for i in 0..n {
                    chip8.ram[RAM_OFFSET as usize + i] = rom_bytes[i]
                }
            }
            Err(ref e) if e.kind() == ErrorKind::Interrupted => (),
            Err(e) => panic!("{:?}", e),
        };

        // initialize pointers
        chip8.registers.sp = -1;
        chip8.registers.pc = RAM_OFFSET;
        chip8
    }

    fn set_vram_bit(&mut self, x: usize, y: usize, value: bool) {
        if value {
            self.vram[y] |= 1 << x;
        } else {
            self.vram[y] &= !(1 << x);
        }
    }

    fn get_vram_bit(&self, x: usize, y: usize) -> bool {
        self.vram[y] & (1 << x) == (1 << x)
    }

    fn keydown(&mut self, key: u16) {
        self.keyboard |= key;
    }

    fn keyup(&mut self, key: u16) {
        self.keyboard &= !(key);
    }

    fn nop(&mut self) {}

    /****************\
     * Instructions *
    \****************/
    // clear screen
    fn cls(&mut self) {
        self.vram = [0u64; CHIP8_DISP_H as usize];
    }

    // return from subroutine
    fn ret(&mut self) {
        self.registers.pc = self.stack[self.registers.sp as usize];
        self.registers.sp -= 1;
    }

    // absolute jump
    fn jp(&mut self, addr: Address) {
        self.registers.pc = addr - 2;
    }

    // call subroutine
    fn call(&mut self, addr: Address) {
        self.registers.sp += 1;
        self.stack[self.registers.sp as usize] = self.registers.pc;
        self.registers.pc = addr - 2;
    }

    // skip next instruction if reg is equal to byte
    fn se_byte(&mut self, vx: Greg, lit: u8) {
        if self.registers.vx[vx as usize] == lit {
            self.registers.pc += 2;
        }
    }

    // conditional skip
    fn sne_byte(&mut self, vx: Greg, lit: u8) {
        if self.registers.vx[vx as usize] != lit {
            self.registers.pc += 2;
        }
    }

    // skip next instruction if reg is equal to another reg
    fn se_reg(&mut self, vx: Greg, vy: Greg) {
        if self.registers.vx[vx as usize] == self.registers.vx[vy as usize] {
            self.registers.pc += 2;
        }
    }

    fn ld_byte(&mut self, vx: Greg, lit: u8) {
        self.registers.vx[vx as usize] = lit;
    }

    fn add_byte(&mut self, vx: Greg, lit: u8) {
        let reg_val = self.registers.vx[vx as usize];
        self.registers.vx[vx as usize] = reg_val.wrapping_add(lit);
    }

    fn ld_reg(&mut self, vx: Greg, vy: Greg) {
        self.registers.vx[vx as usize] = self.registers.vx[vy as usize];
    }

    fn or(&mut self, vx: Greg, vy: Greg) {
        self.registers.vx[vx as usize] |= self.registers.vx[vy as usize];
    }

    fn and(&mut self, vx: Greg, vy: Greg) {
        self.registers.vx[vx as usize] &= self.registers.vx[vy as usize];
    }

    fn xor(&mut self, vx: Greg, vy: Greg) {
        self.registers.vx[vx as usize] ^= self.registers.vx[vy as usize];
    }

    fn add_reg(&mut self, vx: Greg, vy: Greg) {
        let x: usize = self.registers.vx[vx as usize] as usize;
        let y: usize = self.registers.vx[vy as usize] as usize;
        if x + y > 255 {
            self.registers.vx[0xfusize] = 1;
        } else {
            self.registers.vx[0xfusize] = 0;
        }
        let tmp = self.registers.vx[vx as usize];

        self.registers.vx[vx as usize] = tmp.wrapping_add(self.registers.vx[vy as usize]);
    }

    fn sub_reg(&mut self, vx: Greg, vy: Greg) {
        let x: usize = self.registers.vx[vx as usize] as usize;
        let y: usize = self.registers.vx[vy as usize] as usize;
        if x > y {
            self.registers.vx[0xfusize] = 1;
        } else {
            self.registers.vx[0xfusize] = 0;
        }

        let tmp = self.registers.vx[vx as usize];

        self.registers.vx[vx as usize] = tmp.wrapping_sub(self.registers.vx[vy as usize]);
    }

    fn shr(&mut self, vx: Greg) {
        if (self.registers.vx[vx as usize] & 0b0000_0001) == 1 {
            self.registers.vx[0xfusize] = 1;
        } else {
            self.registers.vx[0xfusize] = 0;
        }
        self.registers.vx[vx as usize] /= 2;
    }

    fn subn(&mut self, vx: Greg, vy: Greg) {
        let vx: usize = self.registers.vx[vx as usize] as usize;
        let vy: usize = self.registers.vx[vy as usize] as usize;
        if vy > vx {
            self.registers.vx[0xfusize] = 1;
        } else {
            self.registers.vx[0xfusize] = 0;
        }

        self.registers.vx[vx as usize] =
            self.registers.vx[vy as usize] - self.registers.vx[vx as usize];
    }

    fn shl(&mut self, vx: Greg) {
        if (self.registers.vx[vx as usize] & 0b1000_0000) == 1 {
            self.registers.vx[0xfusize] = 1;
        } else {
            self.registers.vx[0xfusize] = 0;
        }
        self.registers.vx[vx as usize] *= 2;
    }

    fn sne_reg(&mut self, vx: Greg, vy: Greg) {
        if self.registers.vx[vx as usize] != self.registers.vx[vy as usize] {
            self.registers.pc += 2;
        }
    }

    fn ld_i(&mut self, lit: Address) {
        self.registers.i = lit;
    }

    fn jp_offset(&mut self, lit: Address) {
        self.registers.pc = self.registers.vx[0] as u16 + lit - 2;
    }

    fn rnd(&mut self, vx: Greg, lit: u8) {
        let val: u8 = rand::random();
        self.registers.vx[vx as usize] = lit & val;
    }

    fn drw(&mut self, vx: Greg, vy: Greg, lit: u8) {
        let mut erased = false;
        for y in 0..(lit & 0b0000_1111) {
            let spriterow = self.ram[self.registers.i as usize + y as usize];
            for x in 0..8 {
                let xpos =
                    (self.registers.vx[vx as usize] as u32 + (7 - x) as u32) as u32 % (CHIP8_DISP_W);
                let ypos = (self.registers.vx[vy as usize] + y) as u32 % (CHIP8_DISP_H);
                println!(
                    "x: {}, vx: {}, {:#04x}, xpos: {}, ypos: {}",
                    x, vx, self.registers.vx[vx as usize], xpos, ypos
                );
                let source_bit = (spriterow >> x) & 0b1;
                let dest_bit = (self.vram[ypos as usize] >> xpos) & 0b1;
                erased = erased || (source_bit == 1 && dest_bit == 1);
                self.vram[ypos as usize] ^= (source_bit as u64) << xpos;
            }
        }

        if erased {
            self.registers.vx[0xfusize] = 1;
        } else {
            self.registers.vx[0xfusize] = 0;
        }
    }

    fn skp(&mut self, vx: Greg) {
        let reg_val = self.registers.vx[vx as usize];
        if self.keyboard >> reg_val == 1 {
            self.registers.pc += 2;
        }
    }

    fn sknp(&mut self, vx: Greg) {
        let reg_val = self.registers.vx[vx as usize];
        if self.keyboard >> reg_val == 0 {
            self.registers.pc += 2;
        }
    }

    fn ld_vx_dt(&mut self, vx: Greg) {
        self.registers.vx[vx as usize] = self.registers.dt;
    }

    fn ld_k(&mut self, vx: Greg) {
        //println!("Inside ld_k");
        let mut key_pressed = self.keyboard;
        if key_pressed != 0 {
            let mut key: u8 = 0;
            while (key_pressed >> 1) != 0 {
                key_pressed >>= 1;
                key += 1;
            }
            self.registers.vx[vx as usize] = key;
        } else {
            self.registers.pc -= 2;
        }
    }

    fn ld_dt_vx(&mut self, vx: Greg) {
        self.registers.dt = self.registers.vx[vx as usize];
    }

    fn ld_st_vx(&mut self, vx: Greg) {
        self.registers.st = self.registers.vx[vx as usize];
    }

    fn add_i(&mut self, vx: Greg) {
        self.registers.i += self.registers.vx[vx as usize] as u16;
    }

    fn ld_f(&mut self, vx: Greg) {
        self.registers.i = 0x000 + 5 * (self.registers.vx[vx as usize] as u16);
    }

    fn ld_b(&mut self, vx: Greg) {
        self.ram[(self.registers.i) as usize] = (self.registers.vx[vx as usize] / 100) % 10;
        self.ram[(self.registers.i + 1) as usize] = (self.registers.vx[vx as usize] / 10) % 10;
        self.ram[(self.registers.i + 2) as usize] = (self.registers.vx[vx as usize] / 1) % 10;
    }

    // store registers v0-vx in memory starting at address I
    fn ld_s(&mut self, vx: Greg) {
        for x in 0..vx + 1 {
            self.ram[(self.registers.i + x as u16) as usize] = self.registers.vx[x as usize];
        }
    }

    // read registers v0-vx from memory starting at address I
    fn ld_r(&mut self, vx: Greg) {
        for x in 0..vx + 1 {
            self.registers.vx[x as usize] = self.ram[(self.registers.i + x as u16) as usize];
        }
    }

    fn tick(&mut self) {
        for _ in 0..INSTRUCTIONS_PER_TICK {
            self.instruction_dispatch(
                self.ram[self.registers.pc as usize],
                self.ram[(self.registers.pc + 1) as usize],
            );
            self.registers.pc += 2;
            if self.registers.pc >= 0x0fff {
                self.registers.pc = 0x0200;
            }
        }

        if self.registers.dt > 0 {
            self.registers.dt -= 1;
        }

        if self.registers.st > 0 {
            self.registers.st -= 1;
        }
    }

    fn instruction_dispatch(&mut self, upper: u8, lower: u8) {
        let nibble1 = (upper & 0b1111_0000) >> 4;
        let nibble2 = upper & 0b0000_1111;
        let nibble3 = (lower & 0b1111_0000) >> 4;
        let nibble4 = lower & 0b0000_1111;
        if DEBUG {
            let decomp = self.decompile(nibble1, nibble2, nibble3, nibble4);
            println!(
                "op: {:#06x} {:<15}, pc: {:#06x}, I: {:#06x}, dt: {}, st: {}, key: {:#018b}, vx: {:?}",
                ((upper as u16) << 8) | lower as u16,
                decomp,
                self.registers.pc,
                self.registers.i,
                self.registers.dt,
                self.registers.st,
                self.keyboard,
                self.registers.vx
            );
        }
        match (nibble1, nibble2, nibble3, nibble4) {
            (0x0, 0x0, 0x0, 0x0) => self.nop(),
            (0x0, 0x0, 0xe, 0x0) => self.cls(),
            (0x0, 0x0, 0xe, 0xe) => self.ret(),
            (0x1, n1, n2, n3) => {
                let address = ((n1 as u16) << 8) | ((n2 as u16) << 4) | n3 as u16;
                self.jp(address);
            }
            (0x2, n1, n2, n3) => {
                let address = ((n1 as u16) << 8) | ((n2 as u16) << 4) | n3 as u16;
                self.call(address);
            }
            (0x3, x, k1, k2) => {
                let literal = ((k1 as u8) << 4) | k2 as u8;
                self.se_byte(x, literal);
            }
            (0x4, x, k1, k2) => {
                let literal = ((k1 as u8) << 4) | k2 as u8;
                self.sne_byte(x, literal);
            }
            (0x5, x, y, 0x0) => {
                self.se_reg(x, y);
            }
            (0x6, x, k1, k2) => {
                let literal = ((k1 as u8) << 4) | k2 as u8;
                self.ld_byte(x, literal);
            }
            (0x7, x, k1, k2) => {
                let literal = ((k1 as u8) << 4) | k2 as u8;
                self.add_byte(x, literal);
            }
            (0x8, x, y, 0x0) => {
                self.ld_reg(x, y);
            }
            (0x8, x, y, 0x1) => {
                self.or(x, y);
            }
            (0x8, x, y, 0x2) => {
                self.and(x, y);
            }
            (0x8, x, y, 0x3) => {
                self.xor(x, y);
            }
            (0x8, x, y, 0x4) => {
                self.add_reg(x, y);
            }
            (0x8, x, y, 0x5) => {
                self.sub_reg(x, y);
            }
            (0x8, x, _, 0x6) => {
                self.shr(x);
            }
            (0x8, x, y, 0x7) => {
                self.subn(x, y);
            }
            (0x8, x, _, 0xe) => {
                self.shl(x);
            }
            (0x9, x, y, 0x0) => {
                self.sne_reg(x, y);
            }
            (0xa, n1, n2, n3) => {
                let address = ((n1 as u16) << 8) | ((n2 as u16) << 4) | n3 as u16;
                self.ld_i(address);
            }
            (0xb, n1, n2, n3) => {
                let address = ((n1 as u16) << 8) | ((n2 as u16) << 4) | n3 as u16;
                self.jp_offset(address);
            }
            (0xc, x, k1, k2) => {
                let literal = ((k1 as u8) << 4) | k2 as u8;
                self.rnd(x, literal);
            }
            (0xd, x, y, n) => {
                self.drw(x, y, n);
            }
            (0xe, x, 0x9, 0xe) => self.skp(x),
            (0xe, x, 0xa, 0x1) => self.sknp(x),
            (0xf, x, 0x0, 0x7) => self.ld_vx_dt(x),
            (0xf, x, 0x0, 0xa) => self.ld_k(x),
            (0xf, x, 0x1, 0x5) => self.ld_dt_vx(x),
            (0xf, x, 0x1, 0x8) => self.ld_st_vx(x),
            (0xf, x, 0x1, 0xe) => self.add_i(x),
            (0xf, x, 0x2, 0x9) => self.ld_f(x),
            (0xf, x, 0x3, 0x3) => self.ld_b(x),
            (0xf, x, 0x5, 0x5) => self.ld_s(x),
            (0xf, x, 0x6, 0x5) => self.ld_r(x),
            (_, _, _, _) => {
                println!("Unrecognized opcode: {:#x} {:#x}", upper, lower);
            }
        };
    }

    fn decompile(&self, n1: u8, n2: u8, n3: u8, n4: u8) -> String {
        let decomp = match (n1, n2, n3, n4) {
            (0x0, 0x0, 0xe, 0x0) => self.decompile_NNNN(n1, n2, n3, n4),
            (0x0, 0x0, 0xe, 0xe) => self.decompile_NNNN(n1, n2, n3, n4),
            (0x1, _, _, _) => self.decompile_Nnnn(n1, n2, n3, n4),
            (0x2, _, _, _) => self.decompile_Nnnn(n1, n2, n3, n4),
            (0x3, _, _, _) => self.decompile_Nxkk(n1, n2, n3, n4),
            (0x4, _, _, _) => self.decompile_Nxkk(n1, n2, n3, n4),
            (0x5, _, _, 0x0) => self.decompile_NxyN(n1, n2, n3, n4),
            (0x6, _, _, _) => self.decompile_Nxkk(n1, n2, n3, n4),
            (0x7, _, _, _) => self.decompile_Nxkk(n1, n2, n3, n4),
            (0x8, _, _, 0x0) => self.decompile_NxyN(n1, n2, n3, n4),
            (0x8, _, _, 0x1) => self.decompile_NxyN(n1, n2, n3, n4),
            (0x8, _, _, 0x2) => self.decompile_NxyN(n1, n2, n3, n4),
            (0x8, _, _, 0x3) => self.decompile_NxyN(n1, n2, n3, n4),
            (0x8, _, _, 0x4) => self.decompile_NxyN(n1, n2, n3, n4),
            (0x8, _, _, 0x5) => self.decompile_NxyN(n1, n2, n3, n4),
            (0x8, _, _, 0x6) => self.decompile_NxyN(n1, n2, n3, n4),
            (0x8, _, _, 0x7) => self.decompile_NxyN(n1, n2, n3, n4),
            (0x8, _, _, 0xe) => self.decompile_NxyN(n1, n2, n3, n4),
            (0x9, _, _, 0x0) => self.decompile_NxyN(n1, n2, n3, n4),
            (0xa, _, _, _) => self.decompile_Nnnn(n1, n2, n3, n4),
            (0xb, _, _, _) => self.decompile_Nnnn(n1, n2, n3, n4),
            (0xc, _, _, _) => self.decompile_Nxkk(n1, n2, n3, n4),
            (0xd, _, _, _) => self.decompile_Nxyn(n1, n2, n3, n4),
            (0xe, _, 0x9, 0xe) => self.decompile_NxNN(n1, n2, n3, n4),
            (0xe, _, 0xa, 0x1) => self.decompile_NxNN(n1, n2, n3, n4),
            (0xf, _, 0x0, 0x7) => self.decompile_NxNN(n1, n2, n3, n4),
            (0xf, _, 0x0, 0xa) => self.decompile_NxNN(n1, n2, n3, n4),
            (0xf, _, 0x1, 0x5) => self.decompile_NxNN(n1, n2, n3, n4),
            (0xf, _, 0x1, 0x8) => self.decompile_NxNN(n1, n2, n3, n4),
            (0xf, _, 0x1, 0xe) => self.decompile_NxNN(n1, n2, n3, n4),
            (0xf, _, 0x2, 0x9) => self.decompile_NxNN(n1, n2, n3, n4),
            (0xf, _, 0x3, 0x3) => self.decompile_NxNN(n1, n2, n3, n4),
            (0xf, _, 0x5, 0x5) => self.decompile_NxNN(n1, n2, n3, n4),
            (0xf, _, 0x6, 0x5) => self.decompile_NxNN(n1, n2, n3, n4),
            (_, _, _, _) => "Unrecognized opcode: {:#x} {:#x}".to_string(),
        };
        decomp
    }

    fn decompile_Nnnn(&self, n1: u8, n2: u8, n3: u8, n4: u8) -> String {
        let instruction = match n1 {
            0x1 => "JP",
            0x2 => "CALL",
            0xa => "Ld I,",
            0xb => "Jp V0,",
            _ => "Unrecognized",
        };
        let address = ((n2 as u16) << 8) | ((n3 as u16) << 4) | n4 as u16;
        format!("{} {:#06x}", instruction, address)
    }

    fn decompile_Nxkk(&self, n1: u8, n2: u8, n3: u8, n4: u8) -> String {
        let instruction = match n1 {
            0x3 => "SE",
            0x4 => "SNE",
            0x6 => "LD",
            0x7 => "ADD",
            0xC => "RND",
            _ => "Unrecognized",
        };
        let register = n2;
        let byte = (n3 << 4) | n4;
        format!("{} v{}, {:#04x}", instruction, register, byte)
    }

    fn decompile_NNNN(&self, n1: u8, n2: u8, n3: u8, n4: u8) -> String {
        match (n1, n2, n3, n4) {
            (0x0, 0x0, 0xe, 0x0) => "CLS".to_string(),
            (0x0, 0x0, 0xe, 0xe) => "RET".to_string(),
            _ => "Unrecognized".to_string(),
        }
    }

    fn decompile_NxyN(&self, n1: u8, n2: u8, n3: u8, n4: u8) -> String {
        let instruction = match (n1, n4) {
            (0x5, 0x0) => "SE",
            (0x8, 0x0) => "LD",
            (0x8, 0x1) => "OR",
            (0x8, 0x2) => "AND",
            (0x8, 0x3) => "XOR",
            (0x8, 0x4) => "ADD",
            (0x8, 0x5) => "SUB",
            (0x8, 0x6) => "SHR",
            (0x8, 0x7) => "SUBN",
            (0x8, 0xe) => "SHL",
            (0x9, 0x0) => "SNE",

            _ => "Unrecognized",
        };
        let r1 = n2;
        let r2 = n3;
        format!("{} v{}, v{}", instruction, r1, r2)
    }

    fn decompile_Nxyn(&self, n1: u8, n2: u8, n3: u8, n4: u8) -> String {
        let instruction = match n1 {
            0xD => "DRW",
            _ => "Unrecognized",
        };
        let r1 = n2;
        let r2 = n3;
        format!("{} v{}, v{}, {:#03x}", instruction, r1, r2, n4)
    }

    fn decompile_NxNN(&self, n1: u8, n2: u8, n3: u8, n4: u8) -> String {
        let r1 = n2;
        let instruction = match (n1, n3, n4) {
            (0xe, 0x9, 0xe) => format!("SKP v{}", r1),
            (0xe, 0xa, 0x1) => format!("SKNP v{}", r1),
            (0xf, 0x0, 0x7) => format!("LD v{}, DT", r1),
            (0xf, 0x0, 0xa) => format!("LD v{}, K", r1),
            (0xf, 0x1, 0x5) => format!("LD DT, v{}", r1),
            (0xf, 0x1, 0x8) => format!("LD ST, v{}", r1),
            (0xf, 0x1, 0xe) => format!("ADD I, v{}", r1),
            (0xf, 0x2, 0x9) => format!("LD F, v{}", r1),
            (0xf, 0x3, 0x3) => format!("LD B, v{}", r1),
            (0xf, 0x5, 0x5) => format!("LD [I], v{}", r1),
            (0xf, 0x6, 0x5) => format!("LD v{}, [I]", r1),
            _ => "Unrecognized".to_string(),
        };
        instruction
    }
}

pub fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window(
            "Chip-8 Emulator",
            CELL_W * (CHIP8_DISP_W),
            CELL_H * (CHIP8_DISP_H),
        )
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    //let mut chip8 = Chip8::new("Roms/IBM Logo.ch8".to_string());
    //let mut chip8 = Chip8::new("Roms/Maze [David Winter, 199x].ch8".to_string());
    //let mut chip8 = Chip8::new("Roms/Chip8 Picture.ch8".to_string());
    //let mut chip8 = Chip8::new("Roms/Chip8 emulator Logo [Garstyciuks].ch8".to_string());
    //let mut chip8 = Chip8::new("Roms/Keypad Test [Hap, 2006].ch8".to_string());
    //let mut chip8 = Chip8::new("Roms/chip8-test-rom.ch8".to_string());
    //let mut chip8 = Chip8::new("Roms/Breakout [Carmelo Cortez, 1979].ch8".to_string());
    let mut chip8 = Chip8::new("Roms/Brix [Andreas Gustafsson, 1990].ch8".to_string());

    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        chip8.tick();

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.set_draw_color(Color::RGB(255, 255, 255));

        for x in 0..CHIP8_DISP_W {
            for y in 0..CHIP8_DISP_H {
                //println!("{}", vram[y as usize]);
                if chip8.get_vram_bit(x as usize, y as usize) {
                    canvas
                        .fill_rect(Rect::new(
                            (x * CELL_W).try_into().unwrap(),
                            (y * CELL_H).try_into().unwrap(),
                            CELL_W,
                            CELL_H,
                        ))
                        .unwrap();
                }
            }
        }

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(Keycode::Num1),
                    ..
                } => chip8.keydown(0b0000_0000_0000_0010),
                Event::KeyDown {
                    keycode: Some(Keycode::Num2),
                    ..
                } => chip8.keydown(0b0000_0000_0000_0100),
                Event::KeyDown {
                    keycode: Some(Keycode::Num3),
                    ..
                } => chip8.keydown(0b0000_0000_0000_1000),
                Event::KeyDown {
                    keycode: Some(Keycode::Num4),
                    ..
                } => chip8.keydown(0b0001_0000_0000_0000),
                Event::KeyDown {
                    keycode: Some(Keycode::Q),
                    ..
                } => chip8.keydown(0b0000_0000_0001_0000),
                Event::KeyDown {
                    keycode: Some(Keycode::W),
                    ..
                } => chip8.keydown(0b0000_0000_0010_0000),
                Event::KeyDown {
                    keycode: Some(Keycode::E),
                    ..
                } => chip8.keydown(0b0000_0000_0100_0000),
                Event::KeyDown {
                    keycode: Some(Keycode::R),
                    ..
                } => chip8.keydown(0b0010_0000_0000_0000),
                Event::KeyDown {
                    keycode: Some(Keycode::A),
                    ..
                } => chip8.keydown(0b0000_0000_1000_0000),
                Event::KeyDown {
                    keycode: Some(Keycode::S),
                    ..
                } => chip8.keydown(0b0000_0001_0000_0000),
                Event::KeyDown {
                    keycode: Some(Keycode::D),
                    ..
                } => chip8.keydown(0b0000_0010_0000_0000),
                Event::KeyDown {
                    keycode: Some(Keycode::F),
                    ..
                } => chip8.keydown(0b0100_0000_0000_0000),
                Event::KeyDown {
                    keycode: Some(Keycode::Z),
                    ..
                } => chip8.keydown(0b0000_0100_0000_0000),
                Event::KeyDown {
                    keycode: Some(Keycode::X),
                    ..
                } => chip8.keydown(0b0000_0000_0000_0001),
                Event::KeyDown {
                    keycode: Some(Keycode::C),
                    ..
                } => chip8.keydown(0b0000_1000_0000_0000),
                Event::KeyDown {
                    keycode: Some(Keycode::V),
                    ..
                } => chip8.keydown(0b1000_0000_0000_0000),
                Event::KeyUp {
                    keycode: Some(Keycode::Num1),
                    ..
                } => chip8.keyup(0b0000_0000_0000_0010),
                Event::KeyUp {
                    keycode: Some(Keycode::Num2),
                    ..
                } => chip8.keyup(0b0000_0000_0000_0100),
                Event::KeyUp {
                    keycode: Some(Keycode::Num3),
                    ..
                } => chip8.keyup(0b0000_0000_0000_1100),
                Event::KeyUp {
                    keycode: Some(Keycode::Num4),
                    ..
                } => chip8.keyup(0b0001_0000_0000_0000),
                Event::KeyUp {
                    keycode: Some(Keycode::Q),
                    ..
                } => chip8.keyup(0b0000_0000_0001_0000),
                Event::KeyUp {
                    keycode: Some(Keycode::W),
                    ..
                } => chip8.keyup(0b0000_0000_0010_0000),
                Event::KeyUp {
                    keycode: Some(Keycode::E),
                    ..
                } => chip8.keyup(0b0000_0000_0100_0000),
                Event::KeyUp {
                    keycode: Some(Keycode::R),
                    ..
                } => chip8.keyup(0b0010_0000_0000_0000),
                Event::KeyUp {
                    keycode: Some(Keycode::A),
                    ..
                } => chip8.keyup(0b0000_0000_1000_0000),
                Event::KeyUp {
                    keycode: Some(Keycode::S),
                    ..
                } => chip8.keyup(0b0000_0001_0000_0000),
                Event::KeyUp {
                    keycode: Some(Keycode::D),
                    ..
                } => chip8.keyup(0b0000_0010_0000_0000),
                Event::KeyUp {
                    keycode: Some(Keycode::F),
                    ..
                } => chip8.keyup(0b0100_0000_0000_0000),
                Event::KeyUp {
                    keycode: Some(Keycode::Z),
                    ..
                } => chip8.keyup(0b0000_0100_0000_0000),
                Event::KeyUp {
                    keycode: Some(Keycode::X),
                    ..
                } => chip8.keyup(0b0000_0000_0000_0001),
                Event::KeyUp {
                    keycode: Some(Keycode::C),
                    ..
                } => chip8.keyup(0b0000_1000_0000_0000),
                Event::KeyUp {
                    keycode: Some(Keycode::V),
                    ..
                } => chip8.keyup(0b1000_0000_0000_0000),
                _ => {}
            }
        }
        // The rest of the game loop goes here...

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / FPS));
    }
}
