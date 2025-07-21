use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Literals
    Number(f64),
    String(String),
    Boolean(bool),
    Nil,

    // Identifiers
    Identifier(String),

    // Keywords
    And,
    Break,
    Do,
    Else,
    ElseIf,
    End,
    False,
    For,
    Function,
    If,
    In,
    Local,
    Not,
    Or,
    Repeat,
    Return,
    Then,
    True,
    Until,
    While,

    // Operators
    Plus,
    Minus,
    Multiply,
    Divide,
    Modulo,
    Power,
    Length,
    Equal,
    NotEqual,
    LessThan,
    LessEqual,
    GreaterThan,
    GreaterEqual,
    Assign,

    // Delimiters
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Semicolon,
    Comma,
    Dot,
    Colon,
    DoubleDot,
    Ellipsis,

    // Special
    EOF,
}

pub struct Lexer {
    source: String,
    position: usize,
    line: usize,
    keywords: HashMap<String, Token>,
}

impl Lexer {
    pub fn new(source: String) -> Self {
        let mut keywords = HashMap::new();
        keywords.insert("and".to_string(), Token::And);
        keywords.insert("break".to_string(), Token::Break);
        keywords.insert("do".to_string(), Token::Do);
        keywords.insert("else".to_string(), Token::Else);
        keywords.insert("elseif".to_string(), Token::ElseIf);
        keywords.insert("end".to_string(), Token::End);
        keywords.insert("false".to_string(), Token::False);
        keywords.insert("for".to_string(), Token::For);
        keywords.insert("function".to_string(), Token::Function);
        keywords.insert("if".to_string(), Token::If);
        keywords.insert("in".to_string(), Token::In);
        keywords.insert("local".to_string(), Token::Local);
        keywords.insert("not".to_string(), Token::Not);
        keywords.insert("or".to_string(), Token::Or);
        keywords.insert("repeat".to_string(), Token::Repeat);
        keywords.insert("return".to_string(), Token::Return);
        keywords.insert("then".to_string(), Token::Then);
        keywords.insert("true".to_string(), Token::True);
        keywords.insert("until".to_string(), Token::Until);
        keywords.insert("while".to_string(), Token::While);

        Lexer {
            source,
            position: 0,
            line: 1,
            keywords,
        }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        while let Some(token) = self.next_token() {
            if token == Token::EOF {
                tokens.push(token);
                break;
            }
            tokens.push(token);
        }
        tokens
    }

    fn next_token(&mut self) -> Option<Token> {
        self.skip_whitespace();

        if self.is_at_end() {
            return Some(Token::EOF);
        }

        let c = self.advance();

        match c {
            '+' => Some(Token::Plus),
            '-' => Some(Token::Minus),
            '*' => Some(Token::Multiply),
            '/' => Some(Token::Divide),
            '%' => Some(Token::Modulo),
            '^' => Some(Token::Power),
            '#' => Some(Token::Length),
            '(' => Some(Token::LeftParen),
            ')' => Some(Token::RightParen),
            '{' => Some(Token::LeftBrace),
            '}' => Some(Token::RightBrace),
            '[' => Some(Token::LeftBracket),
            ']' => Some(Token::RightBracket),
            ';' => Some(Token::Semicolon),
            ',' => Some(Token::Comma),
            '=' => {
                if self.match_char('=') {
                    Some(Token::Equal)
                } else {
                    Some(Token::Assign)
                }
            }
            '<' => {
                if self.match_char('=') {
                    Some(Token::LessEqual)
                } else {
                    Some(Token::LessThan)
                }
            }
            '>' => {
                if self.match_char('=') {
                    Some(Token::GreaterEqual)
                } else {
                    Some(Token::GreaterThan)
                }
            }
            '~' => {
                if self.match_char('=') {
                    Some(Token::NotEqual)
                } else {
                    None
                }
            }
            '.' => {
                if self.match_char('.') {
                    if self.match_char('.') {
                        Some(Token::Ellipsis)
                    } else {
                        Some(Token::DoubleDot)
                    }
                } else {
                    Some(Token::Dot)
                }
            }
            ':' => Some(Token::Colon),
            '"' => self.string(),
            '\'' => self.string(),
            _ => {
                if c.is_ascii_digit() {
                    self.number(c)
                } else if c.is_ascii_alphabetic() || c == '_' {
                    self.identifier(c)
                } else {
                    None
                }
            }
        }
    }

    fn string(&mut self) -> Option<Token> {
        let mut value = String::new();
        while let Some(c) = self.peek() {
            if c == '"' || c == '\'' {
                self.advance();
                return Some(Token::String(value));
            }
            if c == '\\' {
                self.advance();
                if let Some(escaped) = self.peek() {
                    match escaped {
                        'n' => value.push('\n'),
                        't' => value.push('\t'),
                        'r' => value.push('\r'),
                        '\\' => value.push('\\'),
                        '"' => value.push('"'),
                        '\'' => value.push('\''),
                        _ => value.push(escaped),
                    }
                    self.advance();
                }
            } else {
                value.push(c);
                self.advance();
            }
        }
        None
    }

    fn number(&mut self, first: char) -> Option<Token> {
        let mut value = first.to_string();
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() || c == '.' {
                value.push(c);
                self.advance();
            } else {
                break;
            }
        }
        value.parse().ok().map(Token::Number)
    }

    fn identifier(&mut self, first: char) -> Option<Token> {
        let mut value = first.to_string();
        while let Some(c) = self.peek() {
            if c.is_ascii_alphanumeric() || c == '_' {
                value.push(c);
                self.advance();
            } else {
                break;
            }
        }

        if let Some(keyword) = self.keywords.get(&value) {
            Some(keyword.clone())
        } else {
            Some(Token::Identifier(value))
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            match c {
                ' ' | '\r' | '\t' => {
                    self.advance();
                }
                '\n' => {
                    self.line += 1;
                    self.advance();
                }
                '-' => {
                    if self.peek_next() == Some('-') {
                        self.skip_comment();
                    } else {
                        break;
                    }
                }
                _ => break,
            }
        }
    }

    fn skip_comment(&mut self) {
        while let Some(c) = self.peek() {
            if c == '\n' {
                break;
            }
            self.advance();
        }
    }

    fn match_char(&mut self, expected: char) -> bool {
        if let Some(c) = self.peek() {
            if c == expected {
                self.advance();
                return true;
            }
        }
        false
    }

    fn advance(&mut self) -> char {
        let c = self.source.chars().nth(self.position).unwrap_or('\0');
        self.position += 1;
        c
    }

    fn peek(&self) -> Option<char> {
        self.source.chars().nth(self.position)
    }

    fn peek_next(&self) -> Option<char> {
        self.source.chars().nth(self.position + 1)
    }

    fn is_at_end(&self) -> bool {
        self.position >= self.source.len()
    }
}

