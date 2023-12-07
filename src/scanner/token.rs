#[derive(Debug)]
pub struct Token {
    lexeme: String,
    token_type: TokenType,
    literal: Option<i32>,
    line_number: i32,
}

impl Token {
    pub fn new(lexeme: String, token_type: TokenType, line_number: i32) -> Token {
        Token {
            lexeme,
            token_type,
            line_number,
            literal: None,
        }
    }
    pub fn new_number(lexeme: String, literal: i32, line_number: i32) -> Token {
        Token {
            lexeme,
            token_type: TokenType::Number,
            line_number,
            literal: Some(literal),
        }
    }
}

#[derive(Debug)]
pub enum TokenType {
    // Single character tokens
    LeftParantheses,
    RightParantheses,
    Colon,
    Dot,
    Comma,
    Percent,
    LineBreak,

    // Multi character token
    Identifier,
    Number,
    String,
}
