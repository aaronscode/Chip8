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
        (0x0, 0x0, 0xe, 0x0) | (0x0, 0x0, 0xe, 0xe) => decompile_NNNN(n1, n2, n3, n4),
        (0x1, _, _, _) | (0x2, _, _, _) | (0xa, _, _, _) | (0x_b, _, _, _) => {
            decompile_Nnnn(n1, n2, n3, n4)
        }
        (0x3, _, _, _) | (0x4, _, _, _) | (0x6, _, _, _) | (0x7, _, _, _) | (0xc, _, _, _) => {
            decompile_Nxkk(n1, n2, n3, n4)
        }
        (0x5, _, _, 0x0)
        | (0x8, _, _, 0x0)
        | (0x8, _, _, 0x1)
        | (0x8, _, _, 0x2)
        | (0x8, _, _, 0x3)
        | (0x8, _, _, 0x4)
        | (0x8, _, _, 0x5)
        | (0x8, _, _, 0x6)
        | (0x8, _, _, 0x7)
        | (0x8, _, _, 0xe)
        | (0x9, _, _, 0x0) => decompile_NxyN(n1, n2, n3, n4),
        (0xd, _, _, _) => decompile_Nxyn(n1, n2, n3, n4),
        (0xe, _, 0x9, 0xe)
        | (0xe, _, 0xa, 0x1)
        | (0xf, _, 0x0, 0x7)
        | (0xf, _, 0x0, 0xa)
        | (0xf, _, 0x1, 0x5)
        | (0xf, _, 0x1, 0x8)
        | (0xf, _, 0x1, 0xe)
        | (0xf, _, 0x2, 0x9)
        | (0xf, _, 0x3, 0x3)
        | (0xf, _, 0x5, 0x5)
        | (0xf, _, 0x6, 0x5) => decompile_NxNN(n1, n2, n3, n4),
        (_, _, _, _) => {
            let word: u16 = ((upper as u16) << 8) | lower as u16;
            format!("{:#06x}", word,)
        }
    };
    decomp
}

#[allow(non_snake_case)]
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

#[allow(non_snake_case)]
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
    if (instruction != "Unrecognized") {
        format!("{} v{:01X?},  {:#04x}", instruction, register, byte)
    } else {
        instruction.to_owned()
    }
}

#[allow(non_snake_case)]
fn decompile_NNNN(n1: u8, n2: u8, n3: u8, n4: u8) -> String {
    match (n1, n2, n3, n4) {
        (0x0, 0x0, 0xe, 0x0) => "CLS ".to_string(),
        (0x0, 0x0, 0xe, 0xe) => "RET ".to_string(),
        _ => "Unrecognized".to_string(),
    }
}

#[allow(non_snake_case)]
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

#[allow(non_snake_case)]
fn decompile_Nxyn(n1: u8, n2: u8, n3: u8, n4: u8) -> String {
    let instruction = match n1 {
        0xD => "DRW ",
        _ => "Unrecognized",
    };
    let r1 = n2;
    let r2 = n3;
    format!("{} v{:01X?},  v{:01X?}, {:#03x}", instruction, r1, r2, n4)
}

#[allow(non_snake_case)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(non_snake_case)]
    #[test]
    fn test_decompile_Nnnn() {
        assert_eq!(decompile_Nnnn(0x1, 0x2, 0x3, 0x4), "JP   0x0234".to_owned());
        assert_eq!(decompile_Nnnn(0x2, 0x2, 0x3, 0x4), "CALL 0x0234".to_owned());
        assert_eq!(
            decompile_Nnnn(0xa, 0x2, 0x3, 0x4),
            "LD   I,   0x0234".to_owned()
        );
        assert_eq!(
            decompile_Nnnn(0xb, 0x2, 0x3, 0x4),
            "JP   V0,  0x0234".to_owned()
        );
    }

    #[allow(non_snake_case)]
    #[test]
    fn test_decompile_Nxkk() {
        assert_eq!(decompile_Nxkk(0x3, 0x4, 0x5, 0x6), "SE   v4,  0x56");
        assert_eq!(decompile_Nxkk(0x4, 0x4, 0x5, 0x6), "SNE  v4,  0x56");
        assert_eq!(decompile_Nxkk(0x6, 0x4, 0x5, 0x6), "LD   v4,  0x56");
        assert_eq!(decompile_Nxkk(0x7, 0x4, 0x5, 0x6), "ADD  v4,  0x56");
        assert_eq!(decompile_Nxkk(0xC, 0x4, 0x5, 0x6), "RND  v4,  0x56");
        assert_eq!(decompile_Nxkk(0xA, 0x4, 0x5, 0x6), "Unrecognized");
    }

    #[allow(non_snake_case)]
    #[test]
    fn test_decompile_NNNN() {}

    #[allow(non_snake_case)]
    #[test]
    fn test_decompile_NxyN() {}

    #[allow(non_snake_case)]
    #[test]
    fn test_decompile_Nxyn() {}

    #[allow(non_snake_case)]
    #[test]
    fn test_decompile_NxNN() {
        assert_eq!(decompile_NxNN(0xE, 0x2, 0x9, 0xE), "SKP  v2".to_owned());
        assert_eq!(decompile_NxNN(0xE, 0xA, 0xA, 0x1), "SKNP vA".to_owned());
        assert_eq!(
            decompile_NxNN(0xF, 0x8, 0x0, 0x7),
            "LD   v8,  DT".to_owned()
        );
        assert_eq!(decompile_NxNN(0xF, 0x7, 0x0, 0xA), "LD   v7,  K".to_owned());
        assert_eq!(
            decompile_NxNN(0xF, 0x0, 0x1, 0x5),
            "LD   DT,  v0".to_owned()
        );
        assert_eq!(
            decompile_NxNN(0xF, 0x4, 0x1, 0x8),
            "LD   ST,  v4".to_owned()
        );
        assert_eq!(
            decompile_NxNN(0xF, 0x2, 0x1, 0xE),
            "ADD  I,   v2".to_owned()
        );
        assert_eq!(
            decompile_NxNN(0xF, 0x2, 0x2, 0x9),
            "LD   F,   v2".to_owned()
        );
        assert_eq!(
            decompile_NxNN(0xF, 0x2, 0x3, 0x3),
            "LD   B,   v2".to_owned()
        );
        assert_eq!(
            decompile_NxNN(0xF, 0x2, 0x5, 0x5),
            "LD   [I], v2".to_owned()
        );
        assert_eq!(
            decompile_NxNN(0xF, 0x2, 0x6, 0x5),
            "LD   v2,  [I]".to_owned()
        );
    }
}
