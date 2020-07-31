#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Symbol {
    LBrace,
    RBrace,
    LParen,
    RParen,
    Semicolon,
}
use self::Symbol::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Operator {
    LogicalNegation,    // !
    Minus,              // -
    BitwiseComplement,  // ~
    Plus,               // +
    Star,               // *
    Slash,              // /
}
use self::Operator::*;

impl Operator {
    pub fn is_unary(&self) -> bool {
        match self {
            | Operator::Minus
            | Operator::LogicalNegation
            | Operator::BitwiseComplement => true,
            _ => false,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Keyword {
    Int,
    Return,
}
use self::Keyword::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Symbol(Symbol),
    Operator(Operator),
    Keyword(Keyword),
    Id(String),
    Integer(i32),
}

pub fn lex(source: &str) -> Vec<Token> {
    let mut tokens = Vec::<Token>::new();

    let chars: Vec<char> = source.chars().collect();

    let mut i = 0usize;
    while i < chars.len() {
        let c = chars[i];
        match c {
            '{' => tokens.push(Token::Symbol(LBrace)),
            '}' => tokens.push(Token::Symbol(RBrace)),
            '(' => tokens.push(Token::Symbol(LParen)),
            ')' => tokens.push(Token::Symbol(RParen)),
            ';' => tokens.push(Token::Symbol(Semicolon)),
            '~' => tokens.push(Token::Operator(BitwiseComplement)),
            '!' => tokens.push(Token::Operator(LogicalNegation)),
            '-' => tokens.push(Token::Operator(Minus)),
            '+' => tokens.push(Token::Operator(Plus)),
            '*' => tokens.push(Token::Operator(Star)),
            '/' => tokens.push(Token::Operator(Slash)),
            _ => {
                if c.is_alphabetic() || c == '_' {
                    let mut full = c.to_string();
                    i += 1;
                    while i < chars.len() { // Read an identifier
                        if chars[i].is_alphabetic() || chars[i].is_digit(10) {
                            full.push(chars[i]);
                        } else {
                            i -= 1;
                            break;
                        }
                        i += 1;
                    }

                    match &full.to_lowercase()[..] {
                        "int" => tokens.push(Token::Keyword(Int)),
                        "return" => tokens.push(Token::Keyword(Return)),
                        _ => tokens.push(Token::Id(full)),
                    }
                }
                else if c.is_digit(10) {
                    let mut full = c.to_string();
                    i += 1;
                    while i < chars.len() { // Read the entire number
                        if chars[i].is_digit(10) {
                            full.push(chars[i]);
                        } else {
                            i -= 1;
                            break;
                        }
                        i += 1;
                    }
                    tokens.push(Token::Integer(full.parse().unwrap()));
                }
            }
        }
        i += 1;
    }

    tokens
}