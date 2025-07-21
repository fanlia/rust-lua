use crate::lexer::Token;

#[derive(Debug, Clone)]
pub enum Expr {
    Number(f64),
    String(String),
    Boolean(bool),
    Nil,
    Identifier(String),
    UnaryOp {
        operator: UnaryOperator,
        operand: Box<Expr>,
    },
    BinaryOp {
        left: Box<Expr>,
        operator: BinaryOperator,
        right: Box<Expr>,
    },
    FunctionCall {
        name: String,
        arguments: Vec<Expr>,
    },
    TableAccess {
        table: Box<Expr>,
        key: Box<Expr>,
    },
    TableConstructor {
        fields: Vec<TableField>,
    },
}

#[derive(Debug, Clone)]
pub enum UnaryOperator {
    Not,
    Minus,
    Length,
}

#[derive(Debug, Clone)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Power,
    Concat,
    Equal,
    NotEqual,
    LessThan,
    LessEqual,
    GreaterThan,
    GreaterEqual,
    And,
    Or,
}

#[derive(Debug, Clone)]
pub enum TableField {
    Value(Expr),
    KeyValue(String, Expr),
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Expr(Expr),
    Assignment {
        variables: Vec<String>,
        values: Vec<Expr>,
    },
    LocalAssignment {
        variables: Vec<String>,
        values: Vec<Expr>,
    },
    If {
        condition: Expr,
        then_block: Vec<Stmt>,
        else_if_blocks: Vec<(Expr, Vec<Stmt>)>,
        else_block: Option<Vec<Stmt>>,
    },
    While {
        condition: Expr,
        body: Vec<Stmt>,
    },
    Repeat {
        body: Vec<Stmt>,
        condition: Expr,
    },
    For {
        variable: String,
        start: Expr,
        end: Expr,
        step: Option<Expr>,
        body: Vec<Stmt>,
    },
    Function {
        name: String,
        parameters: Vec<String>,
        body: Vec<Stmt>,
    },
    LocalFunction {
        name: String,
        parameters: Vec<String>,
        body: Vec<Stmt>,
    },
    Return(Option<Vec<Expr>>),
    Break,
}

pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            position: 0,
        }
    }

    pub fn parse(&mut self) -> Vec<Stmt> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            if let Some(stmt) = self.parse_statement() {
                statements.push(stmt);
            }
        }
        statements
    }

    fn parse_statement(&mut self) -> Option<Stmt> {
        if self.match_token(&[Token::If]) {
            self.parse_if()
        } else if self.match_token(&[Token::While]) {
            self.parse_while()
        } else if self.match_token(&[Token::Repeat]) {
            self.parse_repeat()
        } else if self.match_token(&[Token::For]) {
            self.parse_for()
        } else if self.match_token(&[Token::Function]) {
            self.parse_function()
        } else if self.match_token(&[Token::Local]) {
            self.parse_local()
        } else if self.match_token(&[Token::Return]) {
            self.parse_return()
        } else if self.match_token(&[Token::Break]) {
            self.advance();
            Some(Stmt::Break)
        } else {
            let expr = self.parse_expression()?;
            if self.match_token(&[Token::Assign]) {
                self.parse_assignment(expr)
            } else {
                Some(Stmt::Expr(expr))
            }
        }
    }

    fn parse_assignment(&mut self, first: Expr) -> Option<Stmt> {
        let mut variables = vec![match first {
            Expr::Identifier(name) => name,
            _ => return None,
        }];

        while self.match_token(&[Token::Comma]) {
            if let Expr::Identifier(name) = self.parse_expression()? {
                variables.push(name);
            } else {
                return None;
            }
        }

        self.consume(Token::Assign);

        let mut values = Vec::new();
        loop {
            values.push(self.parse_expression()?);
            if !self.match_token(&[Token::Comma]) {
                break;
            }
        }

        Some(Stmt::Assignment { variables, values })
    }

    fn parse_local(&mut self) -> Option<Stmt> {
        if self.match_token(&[Token::Function]) {
            let name = match self.advance()? {
                Token::Identifier(name) => name,
                _ => return None,
            };
            self.consume(Token::LeftParen);
            let parameters = self.parse_parameters()?;
            self.consume(Token::RightParen);
            let body = self.parse_block()?;
            self.consume(Token::End);
            Some(Stmt::LocalFunction {
                name,
                parameters,
                body,
            })
        } else {
            let mut variables = Vec::new();
            loop {
                if let Token::Identifier(name) = self.advance()? {
                    variables.push(name);
                } else {
                    return None;
                }
                if !self.match_token(&[Token::Comma]) {
                    break;
                }
            }

            let mut values = Vec::new();
            if self.match_token(&[Token::Assign]) {
                loop {
                    values.push(self.parse_expression()?);
                    if !self.match_token(&[Token::Comma]) {
                        break;
                    }
                }
            }

            Some(Stmt::LocalAssignment { variables, values })
        }
    }

    fn parse_function(&mut self) -> Option<Stmt> {
        let name = match self.advance()? {
            Token::Identifier(name) => name,
            _ => return None,
        };
        self.consume(Token::LeftParen);
        let parameters = self.parse_parameters()?;
        self.consume(Token::RightParen);
        let body = self.parse_block()?;
        self.consume(Token::End);
        Some(Stmt::Function {
            name,
            parameters,
            body,
        })
    }

    fn parse_parameters(&mut self) -> Option<Vec<String>> {
        let mut parameters = Vec::new();
        if self.check(&Token::RightParen) {
            return Some(parameters);
        }

        loop {
            if let Token::Identifier(name) = self.advance()? {
                parameters.push(name);
            } else {
                return None;
            }
            if !self.match_token(&[Token::Comma]) {
                break;
            }
        }

        Some(parameters)
    }

    fn parse_block(&mut self) -> Option<Vec<Stmt>> {
        let mut statements = Vec::new();
        while !self.check(&Token::End) && !self.check(&Token::Until) && !self.is_at_end() {
            if let Some(stmt) = self.parse_statement() {
                statements.push(stmt);
            }
        }
        Some(statements)
    }

    fn parse_if(&mut self) -> Option<Stmt> {
        let condition = self.parse_expression()?;
        self.consume(Token::Then);
        let then_block = self.parse_block()?;

        let mut else_if_blocks = Vec::new();
        while self.match_token(&[Token::ElseIf]) {
            let condition = self.parse_expression()?;
            self.consume(Token::Then);
            let block = self.parse_block()?;
            else_if_blocks.push((condition, block));
        }

        let else_block = if self.match_token(&[Token::Else]) {
            Some(self.parse_block()?)
        } else {
            None
        };

        self.consume(Token::End);
        Some(Stmt::If {
            condition,
            then_block,
            else_if_blocks,
            else_block,
        })
    }

    fn parse_while(&mut self) -> Option<Stmt> {
        let condition = self.parse_expression()?;
        self.consume(Token::Do);
        let body = self.parse_block()?;
        self.consume(Token::End);
        Some(Stmt::While { condition, body })
    }

    fn parse_repeat(&mut self) -> Option<Stmt> {
        let body = self.parse_block()?;
        self.consume(Token::Until);
        let condition = self.parse_expression()?;
        Some(Stmt::Repeat { body, condition })
    }

    fn parse_for(&mut self) -> Option<Stmt> {
        let variable = match self.advance()? {
            Token::Identifier(name) => name,
            _ => return None,
        };
        self.consume(Token::Assign);
        let start = self.parse_expression()?;
        self.consume(Token::Comma);
        let end = self.parse_expression()?;
        let step = if self.match_token(&[Token::Comma]) {
            Some(self.parse_expression()?)
        } else {
            None
        };
        self.consume(Token::Do);
        let body = self.parse_block()?;
        self.consume(Token::End);
        Some(Stmt::For {
            variable,
            start,
            end,
            step,
            body,
        })
    }

    fn parse_return(&mut self) -> Option<Stmt> {
        if self.check(&Token::End) {
            return Some(Stmt::Return(None));
        }

        let mut values = Vec::new();
        loop {
            values.push(self.parse_expression()?);
            if !self.match_token(&[Token::Comma]) {
                break;
            }
        }

        Some(Stmt::Return(Some(values)))
    }

    fn parse_expression(&mut self) -> Option<Expr> {
        self.parse_binary()
    }

    fn parse_binary(&mut self) -> Option<Expr> {
        let mut left = self.parse_unary()?;

        while let Some(op) = self.match_binary_op() {
            let right = self.parse_unary()?;
            left = Expr::BinaryOp {
                left: Box::new(left),
                operator: op,
                right: Box::new(right),
            };
        }

        Some(left)
    }

    fn parse_unary(&mut self) -> Option<Expr> {
        if let Some(op) = self.match_unary_op() {
            let operand = Box::new(self.parse_unary()?);
            Some(Expr::UnaryOp {
                operator: op,
                operand,
            })
        } else {
            self.parse_primary()
        }
    }

    fn parse_primary(&mut self) -> Option<Expr> {
        match self.advance()? {
            Token::Number(n) => Some(Expr::Number(n)),
            Token::String(s) => Some(Expr::String(s)),
            Token::Boolean(b) => Some(Expr::Boolean(b)),
            Token::Nil => Some(Expr::Nil),
            Token::Identifier(name) => Some(Expr::Identifier(name)),
            Token::LeftParen => {
                let expr = self.parse_expression()?;
                self.consume(Token::RightParen);
                Some(expr)
            }
            _ => None,
        }
    }

    fn match_binary_op(&mut self) -> Option<BinaryOperator> {
        if let Some(token) = self.peek() {
            match token {
                Token::Plus => {
                    self.advance();
                    Some(BinaryOperator::Add)
                }
                Token::Minus => {
                    self.advance();
                    Some(BinaryOperator::Subtract)
                }
                Token::Multiply => {
                    self.advance();
                    Some(BinaryOperator::Multiply)
                }
                Token::Divide => {
                    self.advance();
                    Some(BinaryOperator::Divide)
                }
                Token::Power => {
                    self.advance();
                    Some(BinaryOperator::Power)
                }
                Token::Equal => {
                    self.advance();
                    Some(BinaryOperator::Equal)
                }
                Token::NotEqual => {
                    self.advance();
                    Some(BinaryOperator::NotEqual)
                }
                Token::LessThan => {
                    self.advance();
                    Some(BinaryOperator::LessThan)
                }
                Token::LessEqual => {
                    self.advance();
                    Some(BinaryOperator::LessEqual)
                }
                Token::GreaterThan => {
                    self.advance();
                    Some(BinaryOperator::GreaterThan)
                }
                Token::GreaterEqual => {
                    self.advance();
                    Some(BinaryOperator::GreaterEqual)
                }
                Token::And => {
                    self.advance();
                    Some(BinaryOperator::And)
                }
                Token::Or => {
                    self.advance();
                    Some(BinaryOperator::Or)
                }
                _ => None,
            }
        } else {
            None
        }
    }

    fn match_unary_op(&mut self) -> Option<UnaryOperator> {
        if let Some(token) = self.peek() {
            match token {
                Token::Not => {
                    self.advance();
                    Some(UnaryOperator::Not)
                }
                Token::Minus => {
                    self.advance();
                    Some(UnaryOperator::Minus)
                }
                Token::Length => {
                    self.advance();
                    Some(UnaryOperator::Length)
                }
                _ => None,
            }
        } else {
            None
        }
    }

    fn match_token(&mut self, tokens: &[Token]) -> bool {
        for token in tokens {
            if self.check(token) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn check(&self, token: &Token) -> bool {
        if self.is_at_end() {
            return false;
        }
        &self.tokens[self.position] == token
    }

    fn advance(&mut self) -> Option<Token> {
        if self.is_at_end() {
            return None;
        }
        let token = self.tokens[self.position].clone();
        self.position += 1;
        Some(token)
    }

    fn consume(&mut self, token: Token) {
        if self.check(&token) {
            self.advance();
        }
    }

    fn is_at_end(&self) -> bool {
        self.position >= self.tokens.len() || self.tokens[self.position] == Token::EOF
    }

    fn peek(&self) -> Option<&Token> {
        if self.is_at_end() {
            None
        } else {
            Some(&self.tokens[self.position])
        }
    }
}

