// Author: tochiu (github.com/tochiu/lc3-assembler)
//
// September 7th, 2023
//
// This is a very simple assembler for the LC-3 ISA. It is not meant to be
// robust or feature-complete, but rather a simple tool to help people translate valid
// LC-3 assembly into machine code.

use num_parse::*;

#[derive(Debug, Clone, Copy)]
enum Instruction {
    Add,
    And,
    Branch,
    Jump,
    JumpSubroutine,
    JumpSubroutineRegister,
    Load,
    LoadIndirect,
    LoadRegister,
    LoadEffectiveAddress,
    Not,
    Return,
    ReturnInterrupt,
    Store,
    StoreIndirect,
    StoreRegister,
    Trap,
}

impl Instruction {
    fn binary(self) -> u16 {
        match self {
            Self::Add => 0b0001,
            Self::And => 0b0101,
            Self::Branch => 0b0000,
            Self::Jump => 0b1100,
            Self::JumpSubroutine => 0b0100,
            Self::JumpSubroutineRegister => 0b0100,
            Self::Load => 0b0010,
            Self::LoadIndirect => 0b0010,
            Self::LoadRegister => 0b0110,
            Self::LoadEffectiveAddress => 0b1110,
            Self::Not => 0b1001,
            Self::Return => 0b1100,
            Self::ReturnInterrupt => 0b1100,
            Self::Store => 0b0011,
            Self::StoreIndirect => 0b0011,
            Self::StoreRegister => 0b0111,
            Self::Trap => 0b1111,
        }
    }

    // this means that any instructions that share the same keyword must have the same arity
    fn num_args(self) -> usize {
        match self {
            Self::Add => 3,
            Self::And => 3,
            Self::Branch => 2,
            Self::Jump => 1,
            Self::JumpSubroutine => 1,
            Self::JumpSubroutineRegister => 1,
            Self::Load => 2,
            Self::LoadIndirect => 2,
            Self::LoadRegister => 3,
            Self::LoadEffectiveAddress => 2,
            Self::Not => 2,
            Self::Return => 0,
            Self::ReturnInterrupt => 0,
            Self::Store => 2,
            Self::StoreIndirect => 2,
            Self::StoreRegister => 3,
            Self::Trap => 1,
        }
    }
}

impl TryFrom<&str> for Instruction {

    type Error = &'static str;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "add" => Ok(Self::Add),
            "and" => Ok(Self::And),
            "br" => Ok(Self::Branch),
            "jmp" => Ok(Self::Jump),
            "jsr" => Ok(Self::JumpSubroutine),
            "jsrr" => Ok(Self::JumpSubroutineRegister),
            "ld" => Ok(Self::Load),
            "ldi" => Ok(Self::LoadIndirect),
            "ldr" => Ok(Self::LoadRegister),
            "lea" => Ok(Self::LoadEffectiveAddress),
            "not" => Ok(Self::Not),
            "ret" => Ok(Self::Return),
            "rti" => Ok(Self::ReturnInterrupt),
            "st" => Ok(Self::Store),
            "sti" => Ok(Self::StoreIndirect),
            "str" => Ok(Self::StoreRegister),
            "trap" => Ok(Self::Trap),
            _ => Err("Invalid instruction"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum InstructionData {
    Add {
        dr: u8,
        sr1: u8,
        sr2: u8,
    },

    AddImmediate {
        dr: u8,
        sr1: u8,
        imm5: i8,
    },
    
    And {
        dr: u8,
        sr1: u8,
        sr2: u8,
    },

    AndImmediate {
        dr: u8,
        sr1: u8,
        imm5: i8,
    },

    Branch {
        nzp: u8,
        pc_offset9: i16,
    },
    
    Jump {
        base_r: u8,
    },

    JumpSubroutine {
        pc_offset11: i16,
    },

    JumpSubroutineRegister {
        base_r: u8,
    },

    Load {
        dr: u8,
        pc_offset9: i16,
    },

    LoadIndirect {
        dr: u8,
        pc_offset9: i16,
    },

    LoadRegister {
        dr: u8,
        base_r: u8,
        offset6: i8,
    },

    LoadEffectiveAddress {
        dr: u8,
        pc_offset9: i16,
    },

    Not {
        dr: u8,
        sr: u8,
    },

    Return,

    ReturnInterrupt,

    Store {
        sr: u8,
        pc_offset9: i16,
    },

    StoreIndirect {
        sr: u8,
        pc_offset9: i16,
    },

    StoreRegister {
        sr: u8,
        base_r: u8,
        offset6: i8,
    },

    Trap {
        trapvect8: u8,
    },
}

impl InstructionData {
    fn binary(self) -> u16 {
        match self {
            Self::Add { dr, sr1, sr2 } => (dr as u16) << 9 | (sr1 as u16) << 6 | (sr2 as u16),
            Self::AddImmediate { dr, sr1, imm5 } => (dr as u16) << 9 | (sr1 as u16) << 6 | 1 << 5 | (imm5 as u16) & ((1 << 5) - 1),
            Self::And { dr, sr1, sr2 } => (dr as u16) << 9 | (sr1 as u16) << 6 | (sr2 as u16),
            Self::AndImmediate { dr, sr1, imm5 } => (dr as u16) << 9 | (sr1 as u16) << 6 | 1 << 5 | (imm5 as u16) & ((1 << 5) - 1),
            Self::Branch { nzp, pc_offset9 } => (nzp as u16) << 9 | (pc_offset9 as u16) & ((1 << 9) - 1),
            Self::Jump { base_r } => (base_r as u16) << 6,
            Self::JumpSubroutine { pc_offset11 } => 1 << 11 | pc_offset11 as u16 & ((1 << 11) - 1),
            Self::JumpSubroutineRegister { base_r } => (base_r as u16) << 6,
            Self::Load { dr, pc_offset9 } => (dr as u16) << 9 | (pc_offset9 as u16) & ((1 << 9) - 1),
            Self::LoadIndirect { dr, pc_offset9 } => (dr as u16) << 9 | (pc_offset9 as u16) & ((1 << 9) - 1),
            Self::LoadRegister { dr, base_r, offset6 } => (dr as u16) << 9 | (base_r as u16) << 6 | (offset6 as u16) & ((1 << 6) - 1),
            Self::LoadEffectiveAddress { dr, pc_offset9 } => (dr as u16) << 9 | (pc_offset9 as u16) & ((1 << 9) - 1),
            Self::Not { dr, sr } => (dr as u16) << 9 | (sr as u16) << 6 | 0b111111,
            Self::Return => 0b000111000000,
            Self::ReturnInterrupt => 0b000000000000,
            Self::Store { sr, pc_offset9 } => (sr as u16) << 9 | (pc_offset9 as u16) & ((1 << 9) - 1),
            Self::StoreIndirect { sr, pc_offset9 } => (sr as u16) << 9 | (pc_offset9 as u16) & ((1 << 9) - 1),
            Self::StoreRegister { sr, base_r, offset6 } => (sr as u16) << 9 | (base_r as u16) << 6 | (offset6 as u16) & ((1 << 6) - 1),
            Self::Trap { trapvect8 } => trapvect8 as u16,
        }
    }
}

fn parse_register(s: &str) -> Result<u8, String> {
    let mut chars = s.chars();
    if let Some('r' | 'R') = chars.next() {
        if let Some(c) = chars.next() {
            if let Some(register) = c.to_digit(10) {
                if register < 8 {
                    return Ok(register as u8);
                }
            }
        }
    }

    return Err("Invalid register".into());
}

fn parse<'a>(args: &mut &[&str]) -> Result<(Instruction, InstructionData), String>
{
    if args.is_empty() {
        return Err("No instruction".into());
    }

    let instruction = Instruction::try_from(args[0])?;
    *args = &args[1..];

    if instruction.num_args() > args.len() {
        return Err("Invalid number of arguments".into());
    }

    let instruction_data = match instruction {
        Instruction::Add => {
            let dr = parse_register(args[0])?;
            let sr1 = parse_register(args[1])?;

            if let Ok(sr2) = parse_register(args[2]) {
                InstructionData::Add { dr, sr1, sr2 }
            } else {
                let imm5 = parse_uint::<i8>(args[2]).unwrap();
                InstructionData::AddImmediate { dr, sr1, imm5 }
            }
        },
        Instruction::And => {
            let dr = parse_register(args[0])?;
            let sr1 = parse_register(args[1])?;

            if let Ok(sr2) = parse_register(args[2]) {
                InstructionData::And { dr, sr1, sr2 }
            } else {
                let imm5 = parse_uint::<i8>(args[2]).unwrap();
                InstructionData::AndImmediate { dr, sr1, imm5 }
            }
        },
        Instruction::Branch => {
            let mut nzp = 0;
            if args[0].contains('n') {
                nzp |= 0b100;
            }
            if args[0].contains('z') {
                nzp |= 0b010;
            }
            if args[0].contains('p') {
                nzp |= 0b001;
            }

            let pc_offset9 = parse_int::<i16>(args[1]).unwrap();
            InstructionData::Branch { nzp, pc_offset9 }
        },
        Instruction::Jump => {
            let base_r = parse_register(args[0])?;
            InstructionData::Jump { base_r }
        },
        Instruction::JumpSubroutine => {
            let pc_offset11 = parse_int::<i16>(args[0]).unwrap();
            InstructionData::JumpSubroutine { pc_offset11 }
        },
        Instruction::JumpSubroutineRegister => {
            let base_r = parse_register(args[0])?;
            InstructionData::JumpSubroutineRegister { base_r }
        },
        Instruction::Load => {
            let dr = parse_register(args[0])?;
            let pc_offset9 = parse_int::<i16>(args[1]).unwrap();
            InstructionData::Load { dr, pc_offset9 }
        },
        Instruction::LoadIndirect => {
            let dr = parse_register(args[0])?;
            let pc_offset9 = parse_uint::<i16>(args[1]).unwrap();
            InstructionData::LoadIndirect { dr, pc_offset9 }
        },
        Instruction::LoadRegister => {
            let dr = parse_register(args[0])?;
            let base_r = parse_register(args[1])?;
            let offset6 = parse_int::<i8>(args[2]).unwrap();
            InstructionData::LoadRegister { dr, base_r, offset6 }
        },
        Instruction::LoadEffectiveAddress => {
            let dr = parse_register(args[0])?;
            let pc_offset9 = parse_int::<i16>(args[1]).unwrap();
            InstructionData::LoadEffectiveAddress { dr, pc_offset9 }
        },
        Instruction::Not => {
            let dr = parse_register(args[0])?;
            let sr = parse_register(args[1])?;
            InstructionData::Not { dr, sr }
        },
        Instruction::Return => InstructionData::Return,
        Instruction::ReturnInterrupt => InstructionData::ReturnInterrupt,
        Instruction::Store => {
            let sr = parse_register(args[0])?;
            let pc_offset9 = parse_int::<i16>(args[1]).unwrap();
            InstructionData::Store { sr, pc_offset9 }
        },
        Instruction::StoreIndirect => {
            let sr = parse_register(args[0])?;
            let pc_offset9 = parse_int::<i16>(args[1]).unwrap();
            InstructionData::StoreIndirect { sr, pc_offset9 }
        },
        Instruction::StoreRegister => {
            let sr = parse_register(args[0])?;
            let base_r = parse_register(args[1])?;
            let offset6 = parse_int::<i8>(args[2]).unwrap();
            InstructionData::StoreRegister { sr, base_r, offset6 }
        },
        Instruction::Trap => {
            let trapvect8 = parse_uint::<u8>(args[0]).unwrap();
            InstructionData::Trap { trapvect8 }
        },
    };

    *args = &args[instruction.num_args()..];
    Ok((instruction, instruction_data))
}

struct Tokenizer<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        let mut chars = self.input[self.pos..].chars();
        let mut count = 0;

        while let Some(c) = chars.next() {
            if c.is_whitespace() || c == ',' {
                if count > 0 {
                    break;
                } else {
                    self.pos += 1;
                }
            } else {
                count += 1;
            }
        }

        if count > 0 {
            let s = Some(&self.input[self.pos..self.pos + count]);
            self.pos += count;
            s
        } else {
            None
        }
    }
}

fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let file_content = std::fs::read_to_string(&args[0]).unwrap().to_lowercase();
    let tokens = Tokenizer { input: &file_content, pos: 0 }.collect::<Vec<_>>();
    let mut token_slice = tokens.as_slice();

    let mut results = Vec::new();


    while token_slice.len() > 0 {
        results.push(parse(&mut token_slice).unwrap());
    }

    for ((instruction, instruction_data), line) in results.into_iter().zip(file_content.lines()) {
        println!("{:04b}{:012b} // {}", instruction.binary(), instruction_data.binary(), line.to_uppercase());
    }
}
