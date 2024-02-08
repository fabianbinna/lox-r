use std::{collections::VecDeque, rc::Rc};
use crate::{token::{Token, TokenType}, expr::{Expr, ExprType}, error::{report, error}, object::Object, stmt::Stmt};

pub fn parse(tokens: Vec<Token>) -> Vec<Stmt> {
    let mut parser = Parser::new(tokens);
    let mut statements = Vec::new();

    while !parser.is_at_end() {
        statements.push(parser.declaration());
    }

    statements
}

struct Parser {
    tokens: VecDeque<Token>
}

impl Parser {

    fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens: tokens.into()
        }
    }

    fn declaration(&mut self) -> Stmt {
        match self.peek().token_type {
            TokenType::Class => {
                self.advance();
                self.class_declaration()
            },
            TokenType::Fun => {
                self.advance();
                self.function("function")
            },
            TokenType::Var => {
                self.consume(&TokenType::Var);
                self.var_declaration()
            },
            _ => self.statement()
        }
        // TODO: synchronize on error
    }

    fn class_declaration(&mut self) -> Stmt {
        let name = self.consume(&TokenType::Identifier).expect("Expect class name.");

        let superclass = if self.consume(&TokenType::Less).is_some() {
            Some(Expr::new(ExprType::Variable(Box::new(self.consume(&TokenType::Identifier).expect("Expect superclass name.")))))
        } else {
            None
        };

        self.consume(&TokenType::LeftBrace).expect("Expect '{' before class body.");

        let mut methods = vec![];
        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            methods.push(self.function("method"));
        }

        self.consume(&TokenType::RightBrace).expect("Expect '}' after class body");
        Stmt::Class(Box::from(name), Box::from(superclass), methods)

    }

    fn function(&mut self, kind: &str) -> Stmt {
        let name = self.consume(&TokenType::Identifier).expect(format!("Expect {} name.", kind).as_str());
        self.consume(&TokenType::LeftParen).expect(format!("Expect '(' after {} name.", kind).as_str());

        let mut parameters = Vec::new();
        if !self.check(&TokenType::RightParen) {
            loop {
                if parameters.len() >= 255 {
                    error(self.peek().line, "Expect parameter name.".to_string());
                }

                parameters.push(self.consume(&TokenType::Identifier).expect("Expect parameter name."));

                if !self.check(&TokenType::Comma) {
                    break;
                } else {
                    self.advance();
                }
            }
        }
        
        self.consume(&TokenType::RightParen).expect("Expect ')' after parameters.");
        self.consume(&TokenType::LeftBrace).expect(format!("Expect '{{' before {} body.", kind).as_str());
        let body = self.block();
        Stmt::Function(Box::from(name), parameters, Rc::new(body))
    }

    fn var_declaration(&mut self) -> Stmt {
        let name = self.consume(&TokenType::Identifier).expect("Expect variable name.");

        let initializer = if self.check(&TokenType::Equal) {
            self.advance();
            self.expression()
        } else {
            Expr::new(ExprType::Literal(Object::Nil))
        };

        self.consume(&TokenType::Semicolon).expect("Expect ';' after variable declaration.");
        Stmt::Var(Box::new(name), Box::new(initializer))
    }

    fn statement(&mut self) -> Stmt {
        match self.peek().token_type {
            TokenType::For => {
                self.advance();
                self.for_statement()
            },
            TokenType::If => {
                self.advance();
                self.if_statement()
            },
            TokenType::Print => {
                self.consume(&TokenType::Print);
                self.print_statement()
            },
            TokenType::Return => {
                self.advance();
                self.return_statement()
            },
            TokenType::While => {
                self.advance();
                self.while_statement()
            },
            TokenType::LeftBrace => {
                self.advance();
                Stmt::Block(self.block())
            },
            _ => self.expressions_statement()
        }
    }

    fn for_statement(&mut self) -> Stmt {
        self.consume(&TokenType::LeftParen).expect("Expect '(' after 'for'.");
        let initializer = match self.peek().token_type {
            TokenType::Semicolon => {
                self.advance(); 
                None
            },
            TokenType::Var => {
                self.advance(); 
                Some(self.var_declaration())
            },
            _ => Some(self.expressions_statement())
        };

        let condition = if !(self.peek().token_type == TokenType::Semicolon) {
            self.expression()
        } else {
            self.advance();
            Expr::new(ExprType::Literal(Object::Boolean(true)))
        };

        self.consume(&TokenType::Semicolon).expect("Expect ';' after loop condition.");

        let increment = if !(self.peek().token_type == TokenType::RightParen) {
            Some(self.expression())
        } else {
            self.advance();
            None
        };
        self.consume(&TokenType::RightParen).expect("Expect ')' after 'for' clause.");

        let mut body = self.statement();

        if increment.is_some() {
            body = Stmt::Block(vec![body, Stmt::Expression(Box::from(increment.unwrap()))]);
        }

        body = Stmt::While(Box::from(condition), Box::from(body));

        if initializer.is_some() {
            body = Stmt::Block(vec![initializer.unwrap(), body]);
        }

        body
    }

    fn if_statement(&mut self) -> Stmt {
        self.consume(&TokenType::LeftParen).expect("Expect '(' after 'if'.");
        let condition = self.expression();
        self.consume(&TokenType::RightParen).expect("Expect ')' after if condition.");

        let then_branch = self.statement();
        let else_branch = if self.check(&TokenType::Else) {
            self.advance();
            Some(self.statement())
        } else {
            None
        };

        Stmt::If(Box::from(condition), Box::from(then_branch), Box::from(else_branch))
    }

    fn while_statement(&mut self) -> Stmt {
        self.consume(&TokenType::LeftParen).expect("Expect '(' after 'while'.");
        let condition = self.expression();
        self.consume(&TokenType::RightParen).expect("Expect ')' after condition.");
        let body = self.statement();

        Stmt::While(Box::from(condition), Box::from(body))
    }

    fn block(&mut self) -> Vec<Stmt> {
        let mut statements = Vec::new();

        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            statements.push(self.declaration());
        }

        self.consume(&TokenType::RightBrace).expect("Expepect '}' after block.");
        statements
    }

    fn print_statement(&mut self) -> Stmt {
        let expr = self.expression();
        self.consume(&TokenType::Semicolon).expect("Expect ';' after value.");
        Stmt::Print(Box::new(expr))
    }

    fn return_statement(&mut self) -> Stmt {
        let value = if !self.check(&TokenType::Semicolon) {
            Some(self.expression())
        } else {
            None
        };

        self.consume(&TokenType::Semicolon).expect("Expect ';' after return value.");
        Stmt::Return(Box::from(value))
    }

    fn expressions_statement(&mut self) -> Stmt {
        let expr = self.expression();
        self.consume(&TokenType::Semicolon).expect("Expect ';' after expression.");
        Stmt::Expression(Box::new(expr))
    }

    fn expression(&mut self) -> Expr {
        self.assignment()
    }

    fn assignment(&mut self) -> Expr {
        let expr = self.or();

        if self.matching(&[TokenType::Equal]) {
            let equals = self.advance();
            let value = self.assignment();

            return match expr.expr_type {
                ExprType::Variable(name) => Expr::new(ExprType::Assign(name, Box::from(value))),
                ExprType::Get(expr, name) => Expr::new(ExprType::Set(expr, name, Box::from(value))),
                _ => panic!("{}, Invalid assign target.", equals.lexeme)
            }
        }

        expr
    }

    fn or(&mut self) -> Expr {
        let mut expr = self.and();

        while self.check(&TokenType::Or) {
            let operator = self.consume(&TokenType::Or).unwrap();
            let right = self.and();
            expr = Expr::new(ExprType::Logical(Box::from(expr), Box::from(operator), Box::from(right)));
        }

        expr
    }

    fn and(&mut self) -> Expr {
        let mut expr = self.equality();

        while self.check(&TokenType::And) {
            let operator = self.consume(&TokenType::And).unwrap();
            let right = self.equality();
            expr = Expr::new(ExprType::Logical(Box::from(expr), Box::from(operator), Box::from(right)));
        }

        expr
    }

    fn equality(&mut self) -> Expr {
        let mut expr = self.comparison();

        while self.matching(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator = self.advance();
            let right = self.comparison();
            expr = Expr::new(ExprType::Binary(Box::from(expr), Box::from(operator), Box::from(right)));
        }

        expr
    }

    fn comparison(&mut self) -> Expr {
        let mut expr = self.term();

        while self.matching(&[TokenType::Greater, TokenType::GreaterEqual, TokenType::Less, TokenType::LessEqual]) {
            let operator = self.advance();
            let right = self.term();
            expr = Expr::new(ExprType::Binary(Box::from(expr), Box::from(operator), Box::from(right)));
        }

        expr
    }

    fn term(&mut self) -> Expr {
        let mut expr = self.factor();

        while self.matching(&[TokenType::Minus, TokenType::Plus]) {
            let operator = self.advance();
            let right = self.factor();
            expr = Expr::new(ExprType::Binary(Box::from(expr), Box::from(operator), Box::from(right)));
        }

        expr
    }

    fn factor(&mut self) -> Expr {
        let mut expr = self.unary();

        while self.matching(&[TokenType::Slash, TokenType::Star]) {
            let operator = self.advance();
            let right = self.unary();
            expr = Expr::new(ExprType::Binary(Box::from(expr), Box::from(operator), Box::from(right)));
        }

        expr
    }

    fn unary(&mut self) -> Expr {
        if self.matching(&[TokenType::Bang, TokenType::Minus]) {
            let operator = self.advance();
            let right = self.unary();
            return Expr::new(ExprType::Unary(Box::from(operator), Box::from(right)));
        }

        return self.call();
    }

    fn call(&mut self) -> Expr {
        let mut expr = self.primary();

        loop {
            if self.check(&TokenType::LeftParen) {
                self.advance();
                expr = self.finish_call(expr);
            } else if self.check(&TokenType::Dot) {
                self.advance();
                let name = self.consume(&TokenType::Identifier).expect("Expect property name after '.'.");
                expr = Expr::new(ExprType::Get(Box::from(expr), Box::from(name)));
            } else {
                break;
            }
        }

        expr
    }

    fn finish_call(&mut self, callee: Expr) -> Expr {
        let mut arguments = Vec::new();
        if !self.check(&TokenType::RightParen) {
            loop {
                if arguments.len() >= 255 {
                    error(self.peek().line, "Can't have more than 255 arguments.".to_string());
                }

                arguments.push(self.expression());
                if !self.check(&TokenType::Comma) {
                    break;
                }
                self.advance();
            }
        }

        let paren = self.consume(&TokenType::RightParen)
            .expect("Expect ')' after arguments.");

        Expr::new(ExprType::Call(Box::from(callee), Box::from(paren), arguments))
    }

    fn primary(&mut self) -> Expr {
        match self.peek().token_type {
            TokenType::False => {
                self.advance();
                Expr::new(ExprType::Literal(Object::Boolean(false)))
            },
            TokenType::True => {
                self.advance();
                Expr::new(ExprType::Literal(Object::Boolean(true)))
            },
            TokenType::Nil => {
                self.advance();
                Expr::new(ExprType::Literal(Object::Nil))
            },
            TokenType::Number => {
                let token = self.advance();
                Expr::new(ExprType::Literal(Object::Number(token.lexeme.parse::<f64>().unwrap())))
            },
            TokenType::String => {
                let token = self.advance();
                Expr::new(ExprType::Literal(Object::String(token.lexeme)))
            },
            TokenType::LeftParen => {
                self.advance();
                let expr = self.expression();
                self.consume(&TokenType::RightParen).expect("Expected ')' after expression.");
                Expr::new(ExprType::Grouping(Box::from(expr)))
            },
            TokenType::Super => {
                let keyword = self.advance();
                self.consume(&TokenType::Dot).expect("Expect '.' after 'super'");
                let method = self.consume(&TokenType::Identifier).expect("Expect superclass method name.");
                Expr::new(ExprType::Super(Box::from(keyword), Box::from(method)))
            },
            TokenType::This => {
                Expr::new(ExprType::This(Box::new(self.advance())))
            },
            TokenType::Identifier => {
                let variable = self.advance();
                Expr::new(ExprType::Variable(Box::from(variable)))
            }
            _ => {
                report(self.peek().line, format!("at {}", self.peek().lexeme), String::from("Expected expression."));
                panic!("Expected expression.");
            }
        }
    }

    fn matching(&self, token_types: &[TokenType]) -> bool {
        for token_type in token_types {
            if self.check(token_type) {
                return true;
            }
        }
        
        false
    }

    fn consume(&mut self, token_type: &TokenType) -> Option<Token> {
        match self.check(token_type) {
            true => Some(self.advance()),
            false => None
        }
    }

    fn advance(&mut self) -> Token {
        self.tokens.pop_front().unwrap()
    }

    fn check(&self, token_type: &TokenType) -> bool {
        &self.peek().token_type == token_type
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::Eof
    }

    fn peek(&self) -> &Token {
        match self.tokens.get(0) {
            Some(token) => token,
            None => panic!("No more tokens")
        }
    }

}