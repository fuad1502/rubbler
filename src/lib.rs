mod inst;
mod reg;

use inst::*;
use reg::*;
use regex::Regex;
use std::ffi::c_char;
use std::ffi::CStr;

#[no_mangle]
pub extern fn decode_asm_line_ffi(asm_line: *const c_char) -> u32 {
    let c_str = unsafe { CStr::from_ptr(asm_line) };
    decode_asm_line(c_str.to_str().unwrap()).unwrap()
}

pub fn decode_asm_line(asm_line: &str) -> Result<u32, &str> {
    // Grab lines and ensure it only contains one line
    let lines: Vec<&str> = asm_line.lines().collect();
    if lines.len() != 1 {
        return Err("Found multiple lines");
    }

    // Grab instruction string
    let mut tokens: Vec<&str> = lines[0].split(' ').collect();
    if tokens.len() == 0 {
        return Err("Line empty");
    }
    let inst_string = tokens[0];
    tokens.remove(0);

    // Clone and sort INSTRUCTIONS by asm_string
    let mut instructions = INSTRUCTIONS;
    instructions.sort_by_key(|i| i.asm_string);

    // Find instruction
    let inst_idx = match instructions.binary_search_by_key(&inst_string, |i| i.asm_string) {
        Ok(i) => i,
        Err(_) => return Err("Invalid instruction"),
    };
    let inst = &instructions[inst_idx];

    // Set instruction bits
    let mut inst_bits: u32 = inst.opcode_func;

    // Parse arguments
    if inst.num_of_arguments != tokens.len() {
        return Err("Wrong number of arguments");
    }
    for (arg, token) in inst.arguments.iter().zip(tokens.iter()) {
        match arg {
            AsmArgs::RegSrc1 => set_reg(&mut inst_bits, token, RegFunc::Src1)?,
            AsmArgs::RegSrc2 => set_reg(&mut inst_bits, token, RegFunc::Src2)?,
            AsmArgs::RegDest => set_reg(&mut inst_bits, token, RegFunc::Dest)?,
            AsmArgs::Imm => set_imm(&mut inst_bits, token, &inst.inst_type)?,
            AsmArgs::Mem => set_mem(&mut inst_bits, token, &inst.inst_type)?,
            AsmArgs::NoArg => break,
        }
    }

    Ok(inst_bits)
}

fn set_mem(
    inst_bits: &mut u32,
    mem_string: &str,
    inst_type: &InstructionType,
) -> Result<(), &'static str> {
    let re = Regex::new(r"(?<imm>.+)\((?<reg>\w+)\)").unwrap();
    let Some(captures) = re.captures(mem_string) else {
        return Err("Parse failed");
    };
    set_reg(inst_bits, &captures["reg"], RegFunc::Src1)?;
    set_imm(inst_bits, &captures["imm"], inst_type)?;
    Ok(())
}

fn set_imm(
    inst_bits: &mut u32,
    imm_string: &str,
    inst_type: &InstructionType,
) -> Result<(), &'static str> {
    let imm = imm_string_to_i32(imm_string)?;
    let imm = imm as u32;
    match inst_type {
        InstructionType::I => *inst_bits |= (imm & 0xFFF) << 20,
        InstructionType::S => {
            let imm_11_5 = (imm >> 5) & 0x7F;
            let imm_4_0 = imm & 0x1F;
            *inst_bits |= (imm_11_5 << 25) + (imm_4_0 << 7);
        }
        InstructionType::B => {
            let imm_12 = (imm >> 12) & 0x1;
            let imm_10_5 = (imm >> 5) & 0x3F;
            let imm_4_1 = (imm >> 1) & 0xF;
            let imm_11 = (imm >> 11) & 0x1;
            *inst_bits |= (imm_12 << 31) + (imm_10_5 << 25) + (imm_4_1 << 8) + (imm_11 << 7);
        }
        InstructionType::U => {
            let imm_31_12 = (imm >> 12) & 0xFFFFF;
            *inst_bits |= imm_31_12 << 12;
        }
        InstructionType::J => {
            let imm_20 = (imm >> 20) & 0x1;
            let imm_10_1 = (imm >> 1) & 0x3FF;
            let imm_11 = (imm >> 11) & 0x1;
            let imm_19_12 = (imm >> 12) & 0xFF;
            *inst_bits |= (imm_20 << 31) + (imm_10_1 << 21) + (imm_11 << 20) + (imm_19_12 << 12);
        }
        InstructionType::R => panic!("R-type instruction should've not entered here"),
    }
    return Ok(());
}

fn set_reg(
    inst_bits: &mut u32,
    reg_string: &str,
    reg_function: RegFunc,
) -> Result<(), &'static str> {
    // Clone and sort REG_FILE by name
    let mut reg_file = REG_FILE;
    reg_file.sort_by_key(|r| r.name);

    // Find register
    let reg_idx = match reg_file.binary_search_by_key(&reg_string, |r| r.name) {
        Ok(i) => i,
        Err(_) => return Err("Invalid register"),
    };
    let reg = &reg_file[reg_idx];

    // Set instruction bits
    match reg_function {
        RegFunc::Src1 => *inst_bits |= reg.number << 15,
        RegFunc::Src2 => *inst_bits |= reg.number << 20,
        RegFunc::Dest => *inst_bits |= reg.number << 7,
    }
    Ok(())
}

fn imm_string_to_i32(imm_string: &str) -> Result<i32, &'static str> {
    let mut imm_chars = imm_string.chars().peekable();
    let mut sign = 1;
    // Skip + or - sign
    if let Some(&c) = imm_chars.peek() {
        if c == '+' || c == '-' {
            if c == '-' {
                sign = -1;
            }
            imm_chars.next();
        }
    } else {
        return Err("Found empty string");
    }
    // Check radix
    // TODO: support specifying radix at the end
    // TODO: allow underscores in binary literal
    // See https://www.nasm.us/doc/nasmdoc3.html#section-3.4.1
    let mut radix = 10;
    if let Some(&c) = imm_chars.peek() {
        if c == '0' {
            imm_chars.next();
            if let Some(c) = imm_chars.peek() {
                match c {
                    'x' | 'h' => {
                        radix = 16;
                        imm_chars.next();
                    }
                    'b' | 'y' => {
                        radix = 2;
                        imm_chars.next();
                    }
                    'o' | 'q' => {
                        radix = 8;
                        imm_chars.next();
                    }
                    'd' => {
                        radix = 10;
                        imm_chars.next();
                    }
                    _ => radix = 10,
                }
            }
        } else if c == '$' {
            radix = 16;
            imm_chars.next();
        }
    } else {
        return Err("Found empty literal");
    }
    // Decode string
    let imm_string: String = imm_chars.collect();
    match i32::from_str_radix(&imm_string, radix) {
        Ok(x) => Ok(sign * x),
        Err(_) => Err("String decode failed"),
    }
}

#[test]
fn test_imm_string_to_i32() {
    let imm_string = "200";
    assert_eq!(imm_string_to_i32(imm_string).unwrap(), 200);
    let imm_string = "0200";
    assert_eq!(imm_string_to_i32(imm_string).unwrap(), 200);
    let imm_string = "0d200";
    assert_eq!(imm_string_to_i32(imm_string).unwrap(), 200);
    let imm_string = "$0c8";
    assert_eq!(imm_string_to_i32(imm_string).unwrap(), 0xc8);
    let imm_string = "0xc8";
    assert_eq!(imm_string_to_i32(imm_string).unwrap(), 0xc8);
    let imm_string = "0hc8";
    assert_eq!(imm_string_to_i32(imm_string).unwrap(), 0xc8);
    let imm_string = "0o310";
    assert_eq!(imm_string_to_i32(imm_string).unwrap(), 0o310);
    let imm_string = "0q310";
    assert_eq!(imm_string_to_i32(imm_string).unwrap(), 0o310);
    let imm_string = "0b11001000";
    assert_eq!(imm_string_to_i32(imm_string).unwrap(), 0b11001000);
    let imm_string = "0y11001000";
    assert_eq!(imm_string_to_i32(imm_string).unwrap(), 0b11001000);
}
#[test]
fn test_op_imm() {
    let asm_line = "addi t2 t1 -3";
    let expected_result: u32 = 0b111111111101_00110_000_00111_0010011;
    assert_eq!(decode_asm_line(asm_line).unwrap(), expected_result);
}
#[test]
fn test_u_inst() {
    let asm_line = "lui t2 -3";
    let expected_result: u32 = 0b11111111111111111111_00111_0110111;
    assert_eq!(decode_asm_line(asm_line).unwrap(), expected_result);
}
#[test]
fn test_op() {
    let asm_line = "add t2 t1 t0";
    let expected_result: u32 = 0b0000000_00101_00110_000_00111_0110011;
    assert_eq!(decode_asm_line(asm_line).unwrap(), expected_result);
}
#[test]
fn test_jal() {
    let asm_line = "jal t2 -3";
    let expected_result: u32 = 0b1_1111111110_1_11111111_00111_1101111;
    assert_eq!(decode_asm_line(asm_line).unwrap(), expected_result);
}
#[test]
fn test_jalr() {
    let asm_line = "jalr t2 t1 -3";
    let expected_result: u32 = 0b111111111101_00110_000_00111_1100111;
    assert_eq!(decode_asm_line(asm_line).unwrap(), expected_result);
}
#[test]
fn test_b_inst() {
    let asm_line = "beq t2 t1 -3";
    let expected_result: u32 = 0b1_111111_00110_00111_000_1110_1_1100011;
    assert_eq!(decode_asm_line(asm_line).unwrap(), expected_result);
}
#[test]
fn test_load() {
    let asm_line = "lw t2 -3(t1)";
    let expected_result: u32 = 0b111111111101_00110_010_00111_0000011;
    assert_eq!(decode_asm_line(asm_line).unwrap(), expected_result);
}
#[test]
fn test_store() {
    let asm_line = "sw t2 -3(t1)";
    let expected_result: u32 = 0b1111111_00111_00110_010_11101_0100011;
    assert_eq!(decode_asm_line(asm_line).unwrap(), expected_result);
}
