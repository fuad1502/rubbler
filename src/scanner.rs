mod token;
use token::Token;
use token::TokenType;

use std::{iter::Peekable, str::Chars};

pub struct Scanner {
    source: String,
}

impl Scanner {
    pub fn new(source: String) -> Scanner {
        Scanner { source: source }
    }

    pub fn scan_tokens(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = vec![];
        let mut line_number = 0;
        let mut find_end_of_line = false;
        let mut chars = self.source.chars().peekable();
        loop {
            let c = chars.next();
            let c = match c {
                Some(c) => c,
                None => break,
            };
            // Ignore any token after comment token
            if find_end_of_line && c == '\n' {
                find_end_of_line = false;
            } else if find_end_of_line {
                continue;
            }
            // Identify single character token
            if c == '#' || c == ';' {
                find_end_of_line = true;
            } else if c == '(' {
                tokens.push(Token::new(
                    c.to_string(),
                    TokenType::LeftParantheses,
                    line_number,
                ));
            } else if c == ')' {
                tokens.push(Token::new(
                    c.to_string(),
                    TokenType::RightParantheses,
                    line_number,
                ));
            } else if c == ':' {
                tokens.push(Token::new(c.to_string(), TokenType::Colon, line_number));
            } else if c == '.' {
                tokens.push(Token::new(c.to_string(), TokenType::Dot, line_number));
            } else if c == ',' {
                tokens.push(Token::new(c.to_string(), TokenType::Comma, line_number));
            } else if c == '%' {
                tokens.push(Token::new(c.to_string(), TokenType::Percent, line_number));
            }
            // Line break
            else if c == '\n' {
                tokens.push(Token::new(c.to_string(), TokenType::LineBreak, line_number));
                line_number += 1;
            }
            // Ignore whitespace
            else if c.is_whitespace() {
                continue;
            }
            // Identify multi-character tokens
            // String
            else if c == '"' {
                if let Ok(string) = Self::extract_string(c, &mut chars) {
                    tokens.push(Token::new(string, TokenType::String, line_number));
                } else {
                    return Err(Self::error(
                        line_number,
                        "Syntax error",
                        "Error in parsing string",
                    ));
                }
            }
            // Number
            else if c.is_ascii_digit() {
                if let Ok((string, number)) = Self::extract_number(c, &mut chars) {
                    tokens.push(Token::new_number(string, number, line_number));
                } else {
                    return Err(Self::error(
                        line_number,
                        "Syntax error",
                        "Error in parsing number",
                    ));
                }
            }
            // Identifier
            else if c.is_ascii_alphabetic() || c == '_' || c == '.' || c == '$' || c == '@' {
                if let Ok(string) = Self::extract_identifier(c, &mut chars) {
                    tokens.push(Token::new(string, TokenType::Identifier, line_number));
                } else {
                    return Err(Self::error(
                        line_number,
                        "Syntax error",
                        "Error in parsing identifier",
                    ));
                }
            }
            // Unexpected character
            else {
                return Err(Self::error(
                    line_number,
                    "Syntax error",
                    "Unexpected character",
                ));
            }
        }
        Ok(tokens)
    }

    fn extract_string(c: char, chars: &mut Peekable<Chars>) -> Result<String, ()> {
        let mut string = "".to_string();
        while let Some(&c) = chars.peek() {
            if c == '\n' {
                return Err(());
            } else if c == '"' {
                chars.next();
                return Ok(string);
            } else {
                string += &c.to_string();
                chars.next();
            }
        }
        Err(())
    }

    fn extract_number(c: char, chars: &mut Peekable<Chars>) -> Result<(String, i32), ()> {
        let mut string = c.to_string();
        while let Some(&c) = chars.peek() {
            if c.is_whitespace()
                || c == '#'
                || c == ';'
                || c == '('
                || c == ')'
                || c == ','
                || c == ':'
            {
                break;
            } else if c.is_ascii_hexdigit() || c == 'x' || c == 'h' || c == 'q' || c == 'y' {
                string += &c.to_string();
                chars.next();
            } else {
                return Err(());
            }
        }
        if let Ok(number) = super::imm_string_to_i32(&string) {
            return Ok((string, number));
        } else {
            println!("Number string: {}", string);
            return Err(());
        }
    }

    fn extract_identifier(c: char, chars: &mut Peekable<Chars>) -> Result<String, ()> {
        let mut string = c.to_string();
        while let Some(&c) = chars.peek() {
            if c.is_whitespace()
                || c == '#'
                || c == ';'
                || c == '('
                || c == ')'
                || c == ','
                || c == ':'
            {
                break;
            } else if c.is_ascii_alphabetic()
                || c.is_ascii_digit()
                || c == '_'
                || c == '.'
                || c == '$'
                || c == '@'
            {
                string += &c.to_string();
                chars.next();
            } else {
                return Err(());
            }
        }
        Ok(string)
    }

    fn error(line_number: i32, what: &str, description: &str) -> String {
        "[Line ".to_string() + &line_number.to_string() + "] " + what + ": " + description
    }
}

#[cfg(test)]
mod test {
    use super::Scanner;

    #[test]
    fn main() {
        let source = "
.equ RTC_BASE,      0x40000000
.equ TIMER_BASE,    0x40004000

# setup machine trap vector
1:      auipc   t0, %pcrel_hi(mtvec)        # load mtvec(hi)
        addi    t0, t0, %pcrel_lo(1)       # load mtvec(lo)
        csrrw   zero, mtvec, t0

# set mstatus.MIE=1 (enable M mode interrupt)
        li      t0, 8
        csrrs   zero, mstatus, t0

# set mie.MTIE=1 (enable M mode timer interrupts)
        li      t0, 128
        csrrs   zero, mie, t0

# read from mtime
        li      a0, RTC_BASE
        ld      a1, 0(a0)

# write to mtimecmp
        li      a0, TIMER_BASE
        li      t0, 1000000000
        add     a1, a1, t0
        sd      a1, 0(a0)

# loop
loop:
        wfi
        j loop

# break on interrupt
mtvec:
        csrrc  t0, mcause, zero
        bgez t0, fail       # interrupt causes are less than zero
        slli t0, t0, 1      # shift off high bit
        srli t0, t0, 1
        li t1, 7            # check this is an m_timer interrupt
        bne t0, t1, fail
        j pass

pass:
        la a0, pass_msg
        jal puts
        j shutdown

fail:
        la a0, fail_msg
        jal puts
        j shutdown

.section .rodata

pass_msg:
        .string \"PASS\\n\"

fail_msg:
        .string \"FAIL\\n\"
";
        let mut scanner = Scanner::new(source.to_string());
        let tokens = scanner.scan_tokens().unwrap();
        println!("{:#?}", tokens);
    }
}
