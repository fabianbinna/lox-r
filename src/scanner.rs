use std::collections::VecDeque;
use crate::token::{Token, TokenType};

pub fn scan_tokens(source: String) -> Vec<Token> {
    let mut scanner = Scanner::new(source);
    let mut tokens = Vec::new();

    while !scanner.is_at_end() {
        match scanner.scan_token() {
            Some(token) => tokens.push(token),
            None => {}
        }
    }

    tokens.push(scanner.eof());

    tokens
}

struct Scanner {
    source: VecDeque<char>,
    current_lexeme: String,
    line: usize
}

impl Scanner {

    fn new(source: String) -> Self {
        Scanner { 
            source: source.chars().collect(),
            current_lexeme: String::new(),
            line: 1
        }
    }

    fn scan_token(&mut self) -> Option<Token> {
        match self.advance() {
            '(' => Some(self.consume(TokenType::LeftParen)),
            ')' => Some(self.consume(TokenType::RightParen)),
            '{' => Some(self.consume(TokenType::LeftBrace)),
            '}' => Some(self.consume(TokenType::RightBrace)),
            ',' => Some(self.consume(TokenType::Comma)),
            '.' => Some(self.consume(TokenType::Dot)),
            '-' => Some(self.consume(TokenType::Minus)),
            '+' => Some(self.consume(TokenType::Plus)),
            ';' => Some(self.consume(TokenType::Semicolon)),
            '*' => Some(self.consume(TokenType::Star)),
            '!' => {
                if self.is_next('=') {
                    Some(self.advance_and_consume(TokenType::BangEqual))
                } else {
                    Some(self.consume(TokenType::Bang))
                }
            },
            '=' => {
                if self.is_next('=') {
                    Some(self.advance_and_consume(TokenType::EqualEqual))
                } else {
                    Some(self.consume(TokenType::Equal))
                }
            },
            '<' => {
                if self.is_next('=') {
                    Some(self.advance_and_consume(TokenType::LessEqual))
                } else {
                    Some(self.consume(TokenType::Less))
                }
            },
            '>' => {
                if self.is_next('=') {
                    Some(self.advance_and_consume(TokenType::GreaterEqual))
                } else {
                    Some(self.consume(TokenType::Greater))
                }
            },
            '/' => {
                if self.is_next('/') {
                    while !self.is_at_end() && !self.is_next('\n') {
                        self.advance();
                    }
                    self.drop_lexeme();
                    None
                } else {
                    Some(self.consume(TokenType::Slash))
                }
            },
            '"' => {
                while !self.is_next('"') {
                    self.advance_w(true);
                }
                Some(self.advance_and_consume(TokenType::String))
            }
            c => {
                // number
                if c.is_ascii_digit() {
                    while self.peek().is_ascii_digit() {
                        self.advance();
                    }
    
                    if self.peek().eq(&'.') && self.peek_next().is_ascii_digit() {
                        self.advance();
    
                        while self.peek().is_ascii_digit() {
                            self.advance();
                        }
                    }
    
                    Some(self.consume(TokenType::Number))
                // identifier & keywords
                } else if c.is_alphabetic() {
                    while self.peek().is_alphanumeric() {
                        self.advance();
                    }
    
                    Some(self.consume(map_keyword(self.peek_lexeme())))
                } else {
                    None
                }
            }
        }
    }

    fn peek(&self) -> char {
        self.source.get(0).unwrap().clone()
    }

    fn peek_next(&self) -> char {
        self.source.get(1).unwrap().clone()
    }

    fn peek_lexeme(&self) -> &str {
        &self.current_lexeme
    }

    fn is_next(&self, c: char) -> bool {
        self.source.get(0).unwrap().eq(&c) //unwrap
    }

    fn is_at_end(&self) -> bool {
        self.source.is_empty()
    }

    fn advance_w(&mut self, consume_whitespace: bool) -> char {
        let c = match self.source.pop_front() {
            Some(c) => c,
            None => panic!()
        };

        if c.eq(&'\n') {
            self.line += 1;
        }

        if consume_whitespace || !c.is_whitespace() {
            self.current_lexeme.push(c);
        }

        c
    }

    fn advance(&mut self) -> char {
        self.advance_w(false)
    }

    fn advance_and_consume(&mut self, token_type: TokenType) -> Token {
        self.advance();
        self.consume(token_type)
    }

    fn consume(&mut self, token_type: TokenType) -> Token {
        let mut lexeme = self.current_lexeme.to_owned();
        self.current_lexeme = String::new();

        if token_type == TokenType::String {
            lexeme.remove(0);
            lexeme.pop();
        }

        Token::new(
            token_type, 
            lexeme, 
            String::new(), 
            self.line)
    }

    fn drop_lexeme(&mut self) {
        self.current_lexeme = String::new();
    }

    fn eof(&self) -> Token {
        if !self.is_at_end() {
            panic!("We are not finished!");
        }

        Token::new(
            TokenType::Eof, 
            String::new(), 
            String::new(), 
            self.line)
    }

}

fn map_keyword(keyword: &str) -> TokenType {
    match keyword {
        "and" => TokenType::And,
        "class" => TokenType::Class,
        "else" => TokenType::Else,
        "false" => TokenType::False,
        "for" => TokenType::For,
        "fun" => TokenType::Fun,
        "if" => TokenType::If,
        "nil" => TokenType::Nil,
        "or" => TokenType::Or,
        "print" => TokenType::Print,
        "return" => TokenType::Return,
        "super" => TokenType::Super,
        "this" => TokenType::This,
        "true" => TokenType::True,
        "var" => TokenType::Var,
        "while" => TokenType::While,
        _ => TokenType::Identifier
    }
}