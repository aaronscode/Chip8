use super::constants::RAM_OFFSET;

use std::fs::File;
use std::io::Read;
use std::io::{BufReader, ErrorKind};

pub fn decompile_rom(path: String) -> String {
    let file = File::open(path).expect("Cannot Read ROM");
    let mut buf = BufReader::new(file);
    let mut rom_bytes = [0; (0x1000 - RAM_OFFSET as usize)];
    let mut decomp: Vec<String> = Vec::new();

    match buf.read(&mut rom_bytes) {
        Ok(0) => (println! {"No bytes read from ROM!"}),
        Ok(n) => {
            for word in rom_bytes.iter().take(n).collect::<Vec<_>>().chunks(2) {
                decomp.push(decompile_word(*word[0], *word[1]));
            }
        }
        Err(ref e) if e.kind() == ErrorKind::Interrupted => (),
        Err(e) => panic!("{:?}", e),
    };

    decomp.join("\n")
}

pub fn decompile_word(upper: u8, lower: u8) -> String {
    let n1 = (upper & 0b1111_0000) >> 4;
    let n2 = upper & 0b0000_1111;
    let n3 = (lower & 0b1111_0000) >> 4;
    let n4 = lower & 0b0000_1111;
    let decomp = match (n1, n2, n3, n4) {
        (0x0, 0x0, 0xe, 0x0) => decompile_NNNN(n1, n2, n3, n4),
        (0x0, 0x0, 0xe, 0xe) => decompile_NNNN(n1, n2, n3, n4),
        (0x1, _, _, _) => decompile_Nnnn(n1, n2, n3, n4),
        (0x2, _, _, _) => decompile_Nnnn(n1, n2, n3, n4),
        (0x3, _, _, _) => decompile_Nxkk(n1, n2, n3, n4),
        (0x4, _, _, _) => decompile_Nxkk(n1, n2, n3, n4),
        (0x5, _, _, 0x0) => decompile_NxyN(n1, n2, n3, n4),
        (0x6, _, _, _) => decompile_Nxkk(n1, n2, n3, n4),
        (0x7, _, _, _) => decompile_Nxkk(n1, n2, n3, n4),
        (0x8, _, _, 0x0) => decompile_NxyN(n1, n2, n3, n4),
        (0x8, _, _, 0x1) => decompile_NxyN(n1, n2, n3, n4),
        (0x8, _, _, 0x2) => decompile_NxyN(n1, n2, n3, n4),
        (0x8, _, _, 0x3) => decompile_NxyN(n1, n2, n3, n4),
        (0x8, _, _, 0x4) => decompile_NxyN(n1, n2, n3, n4),
        (0x8, _, _, 0x5) => decompile_NxyN(n1, n2, n3, n4),
        (0x8, _, _, 0x6) => decompile_NxyN(n1, n2, n3, n4),
        (0x8, _, _, 0x7) => decompile_NxyN(n1, n2, n3, n4),
        (0x8, _, _, 0xe) => decompile_NxyN(n1, n2, n3, n4),
        (0x9, _, _, 0x0) => decompile_NxyN(n1, n2, n3, n4),
        (0xa, _, _, _) => decompile_Nnnn(n1, n2, n3, n4),
        (0xb, _, _, _) => decompile_Nnnn(n1, n2, n3, n4),
        (0xc, _, _, _) => decompile_Nxkk(n1, n2, n3, n4),
        (0xd, _, _, _) => decompile_Nxyn(n1, n2, n3, n4),
        (0xe, _, 0x9, 0xe) => decompile_NxNN(n1, n2, n3, n4),
        (0xe, _, 0xa, 0x1) => decompile_NxNN(n1, n2, n3, n4),
        (0xf, _, 0x0, 0x7) => decompile_NxNN(n1, n2, n3, n4),
        (0xf, _, 0x0, 0xa) => decompile_NxNN(n1, n2, n3, n4),
        (0xf, _, 0x1, 0x5) => decompile_NxNN(n1, n2, n3, n4),
        (0xf, _, 0x1, 0x8) => decompile_NxNN(n1, n2, n3, n4),
        (0xf, _, 0x1, 0xe) => decompile_NxNN(n1, n2, n3, n4),
        (0xf, _, 0x2, 0x9) => decompile_NxNN(n1, n2, n3, n4),
        (0xf, _, 0x3, 0x3) => decompile_NxNN(n1, n2, n3, n4),
        (0xf, _, 0x5, 0x5) => decompile_NxNN(n1, n2, n3, n4),
        (0xf, _, 0x6, 0x5) => decompile_NxNN(n1, n2, n3, n4),
        (_, _, _, _) => {
            let word: u16 = ((upper as u16) << 8) | lower as u16;
            format!("{:#06x}", word,).to_string()
        }
    };
    decomp
}
fn decompile_Nnnn(n1: u8, n2: u8, n3: u8, n4: u8) -> String {
    let instruction = match n1 {
        0x1 => "JP  ",
        0x2 => "CALL",
        0xa => "LD   I,  ",
        0xb => "JP   V0, ",
        _ => "Unrecognized",
    };
    let address = ((n2 as u16) << 8) | ((n3 as u16) << 4) | n4 as u16;
    format!("{} {:#06x}", instruction, address)
}
fn decompile_Nxkk(n1: u8, n2: u8, n3: u8, n4: u8) -> String {
    let instruction = match n1 {
        0x3 => "SE  ",
        0x4 => "SNE ",
        0x6 => "LD  ",
        0x7 => "ADD ",
        0xC => "RND ",
        _ => "Unrecognized",
    };
    let register = n2;
    let byte = (n3 << 4) | n4;
    format!("{} v{:01X?},  {:#04x}", instruction, register, byte)
}
fn decompile_NNNN(n1: u8, n2: u8, n3: u8, n4: u8) -> String {
    match (n1, n2, n3, n4) {
        (0x0, 0x0, 0xe, 0x0) => "CLS ".to_string(),
        (0x0, 0x0, 0xe, 0xe) => "RET ".to_string(),
        _ => "Unrecognized".to_string(),
    }
}
fn decompile_NxyN(n1: u8, n2: u8, n3: u8, n4: u8) -> String {
    let instruction = match (n1, n4) {
        (0x5, 0x0) => "SE  ",
        (0x8, 0x0) => "LD  ",
        (0x8, 0x1) => "OR  ",
        (0x8, 0x2) => "AND ",
        (0x8, 0x3) => "XOR ",
        (0x8, 0x4) => "ADD ",
        (0x8, 0x5) => "SUB ",
        (0x8, 0x6) => "SHR ",
        (0x8, 0x7) => "SUBN",
        (0x8, 0xe) => "SHL ",
        (0x9, 0x0) => "SNE ",
        _ => "Unrecognized",
    };
    let r1 = n2;
    let r2 = n3;
    format!("{} v{:01X?},  v{:01X?}", instruction, r1, r2)
}
fn decompile_Nxyn(n1: u8, n2: u8, n3: u8, n4: u8) -> String {
    let instruction = match n1 {
        0xD => "DRW ",
        _ => "Unrecognized",
    };
    let r1 = n2;
    let r2 = n3;
    format!("{} v{:01X?},  v{:01X?}, {:#03x}", instruction, r1, r2, n4)
}
fn decompile_NxNN(n1: u8, n2: u8, n3: u8, n4: u8) -> String {
    let r1 = n2;
    let instruction = match (n1, n3, n4) {
        (0xe, 0x9, 0xe) => format!("SKP  v{:01X?}", r1),
        (0xe, 0xa, 0x1) => format!("SKNP v{:01X?}", r1),
        (0xf, 0x0, 0x7) => format!("LD   v{:01X?},  DT", r1),
        (0xf, 0x0, 0xa) => format!("LD   v{:01X?},  K", r1),
        (0xf, 0x1, 0x5) => format!("LD   DT,  v{:01X?}", r1),
        (0xf, 0x1, 0x8) => format!("LD   ST,  v{:01X?}", r1),
        (0xf, 0x1, 0xe) => format!("ADD  I,   v{:01X?}", r1),
        (0xf, 0x2, 0x9) => format!("LD   F,   v{:01X?}", r1),
        (0xf, 0x3, 0x3) => format!("LD   B,   v{:01X?}", r1),
        (0xf, 0x5, 0x5) => format!("LD   [I], v{:01X?}", r1),
        (0xf, 0x6, 0x5) => format!("LD   v{:01X?},  [I]", r1),
        _ => "Unrecognized".to_string(),
    };
    instruction
}
