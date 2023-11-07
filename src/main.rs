use rubbler;

fn main() {
    let asm_line = "add ra sp gp";
    println!("{asm_line} => {:#034b}", rubbler::decode_asm_line(asm_line).unwrap());
}
