// Two sets of grammar rules for Lox programming language are defined below -- the second set of rules is
// stratified to REMOVE ambiguity (e.g. multiplication binds more tightly than addition)
//
// program        → statement* EOF ;
// statement      → expr_stmt
//                | print_stmt ;
// expr_stmt      → expression ";" ;
// print_stmt     → "print" expression ";" ;
//
// expression     → (literal
//                | unary
//                | binary
//                | grouping)
//                ("," expression)?   ;
// literal        → NUMBER | STRING | "true" | "false" | "nil" ;
// grouping       → "(" expression ")" ;
// unary          → ( "-" | "!" ) expression ;
// binary         → expression operator expression ;
// ternary        → expression "?" expression ":" expression ;
// operator       → "==" | "!=" | "<" | "<=" | ">" | ">="
//                | "+"  | "-"  | "*" | "/" ;
//
// Rules with precedence (higher precedence at bottom):
// expression     → ternary ("," ternary)* ;
// ternary        → equality ("?" ternary ":" ternary)? ;
// equality       → comparison ( ( "!=" | "==" ) comparison )* ;
// comparison     → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
// term           → factor ( ( "-" | "+" ) factor )* ;
// factor         → unary ( ( "/" | "*" ) unary )* ;
// unary          → ( "!" | "-" ) unary
//                | primary ;
// primary        → NUMBER | STRING | "true" | "false" | "nil"
//                | "(" expression ")" ;

use std::fmt::{self};

use crate::scanner::{Token, TokenType};

#[derive(Debug, PartialEq, Clone)]
pub enum LiteralValue {
    Number(f64),
    String(String),
    False,
    True,
    Nil,
}

impl LiteralValue {
    pub fn from_bool(b: bool) -> Self {
        if b { Self::True } else { Self::False }
    }
}

impl fmt::Display for LiteralValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LiteralValue::Number(n) => write!(f, "{}", n),
            LiteralValue::String(s) => write!(f, "\"{}\"", s),
            LiteralValue::False => write!(f, "false"),
            LiteralValue::True => write!(f, "true"),
            LiteralValue::Nil => write!(f, "nil"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Expr {
    Literal(LiteralValue),
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Ternary {
        left: Box<Expr>,
        middle: Box<Expr>,
        right: Box<Expr>,
    },
    Grouping {
        expression: Box<Expr>,
    },
    Exprs(Vec<Expr>),
}

pub enum Stmt {
    Expr(Expr),
    Print(Expr),
}

//// Visitor for walking Expression AST ////

// Visitor interface for traversing the AST
pub trait Visitor {
    fn enter_literal(&mut self, _val: &LiteralValue) {}
    fn exit_literal(&mut self, _val: &LiteralValue) {}

    fn enter_binary(&mut self, _left: &Expr, _operator: &Token, _right: &Expr) {}
    fn inter_binary(&mut self, _left: &Expr, _operator: &Token, _right: &Expr) {}
    fn exit_binary(&mut self, _left: &Expr, _operator: &Token, _right: &Expr) {}

    fn enter_ternary(&mut self, _left: &Expr, _middle: &Expr, _right: &Expr) {}
    fn after_ternary_guard(&mut self, _left: &Expr, _middle: &Expr, _right: &Expr) {}
    fn after_ternary_first(&mut self, _left: &Expr, _middle: &Expr, _right: &Expr) {}
    fn exit_ternary(&mut self, _left: &Expr, _middle: &Expr, _right: &Expr) {}

    fn enter_unary(&mut self, _operator: &Token, _right: &Expr) {}
    fn exit_unary(&mut self, _operator: &Token, _right: &Expr) {}

    fn enter_grouping(&mut self, _expr: &Expr) {}
    fn exit_grouping(&mut self, _expr: &Expr) {}

    fn enter_exprs(&mut self, _exprs: &Vec<Expr>) {}
    fn inter_exprs(&mut self, _exprs: &Vec<Expr>, _index: usize) {}
    fn exit_exprs(&mut self, _exprs: &Vec<Expr>) {}
}

// Traverses the expression calling visitor on each node
pub fn walk_expression(expr: &Expr, visitor: &mut impl Visitor) {
    match expr {
        Expr::Literal(val) => {
            visitor.enter_literal(val);
            visitor.exit_literal(val);
        }
        Expr::Grouping { expression } => {
            visitor.enter_grouping(expr);
            walk_expression(expression, visitor);
            visitor.exit_grouping(expr);
        }
        Expr::Unary { operator, right } => {
            visitor.enter_unary(operator, right);
            walk_expression(right, visitor);
            visitor.exit_unary(operator, right);
        }
        Expr::Binary {
            left,
            operator,
            right,
        } => {
            visitor.enter_binary(left, operator, right);
            walk_expression(left, visitor);
            visitor.inter_binary(left, operator, right);
            walk_expression(right, visitor);
            visitor.exit_binary(left, operator, right);
        }
        Expr::Ternary {
            left,
            middle,
            right,
        } => {
            visitor.enter_ternary(left, middle, right);
            walk_expression(left, visitor);
            visitor.after_ternary_guard(left, middle, right);
            walk_expression(middle, visitor);
            visitor.after_ternary_first(left, middle, right);
            walk_expression(right, visitor);
            visitor.exit_ternary(left, middle, right);
        }
        Expr::Exprs(exprs) => {
            visitor.enter_exprs(exprs);

            for (i, expr) in exprs.iter().enumerate() {
                match expr {
                    Expr::Literal(val) => {
                        visitor.enter_literal(val);
                        visitor.exit_literal(val);
                    }
                    Expr::Grouping { expression } => {
                        visitor.enter_grouping(expression);
                        walk_expression(expression, visitor);
                        visitor.exit_grouping(expression);
                    }
                    Expr::Unary { operator, right } => {
                        visitor.enter_unary(operator, right);
                        walk_expression(right, visitor);
                        visitor.exit_unary(operator, right);
                    }
                    Expr::Binary {
                        left,
                        operator,
                        right,
                    } => {
                        visitor.enter_binary(left, operator, right);
                        walk_expression(left, visitor);
                        visitor.inter_binary(left, operator, right);
                        walk_expression(right, visitor);
                        visitor.exit_binary(left, operator, right);
                    }
                    Expr::Ternary {
                        left,
                        middle,
                        right,
                    } => {
                        visitor.enter_ternary(left, middle, right);
                        walk_expression(left, visitor);
                        visitor.after_ternary_guard(left, middle, right);
                        walk_expression(middle, visitor);
                        visitor.after_ternary_first(left, middle, right);
                        walk_expression(right, visitor);
                        visitor.exit_ternary(left, middle, right);
                    }
                    Expr::Exprs(exprs) => {
                        for expr in exprs {
                            walk_expression(expr, visitor);
                        }
                    }
                }

                visitor.inter_exprs(exprs, i);
            }

            visitor.exit_exprs(exprs);
        }
    }
}

pub struct AstPrinter {
    pub printed_str: String,
}

impl Visitor for AstPrinter {
    fn enter_literal(&mut self, val: &LiteralValue) {
        let s = match val {
            LiteralValue::Number(num) => &num.to_string(),
            LiteralValue::String(s) => &(String::from("\"") + s + "\""),
            LiteralValue::False => "false",
            LiteralValue::True => "true",
            LiteralValue::Nil => "nil",
        };

        self.printed_str.push_str(s);
    }
    fn exit_literal(&mut self, _: &LiteralValue) -> () {}

    fn enter_grouping(&mut self, _: &Expr) {
        self.printed_str.push_str("(group ");
    }
    fn exit_grouping(&mut self, _: &Expr) {
        self.printed_str.push_str(")");
    }

    fn enter_unary(&mut self, operator: &Token, _: &Expr) {
        self.printed_str.push_str(&format!("({} ", operator.lexeme));
    }
    fn exit_unary(&mut self, _operator: &Token, _right: &Expr) {
        self.printed_str.push_str(")");
    }

    fn enter_binary(&mut self, _left: &Expr, operator: &Token, _right: &Expr) {
        self.printed_str.push_str(&format!("({} ", operator.lexeme));
    }
    fn inter_binary(&mut self, _left: &Expr, _operator: &Token, _right: &Expr) {
        self.printed_str.push_str(" ");
    }
    fn exit_binary(&mut self, _left: &Expr, _operator: &Token, _right: &Expr) {
        self.printed_str.push_str(")");
    }

    fn enter_ternary(&mut self, _left: &Expr, _middle: &Expr, _right: &Expr) {
        self.printed_str.push_str("(");
    }
    fn after_ternary_guard(&mut self, _left: &Expr, _middle: &Expr, _right: &Expr) {
        self.printed_str.push_str(" ? ");
    }
    fn after_ternary_first(&mut self, _left: &Expr, _middle: &Expr, _right: &Expr) {
        self.printed_str.push_str(" : ");
    }
    fn exit_ternary(&mut self, _left: &Expr, _middle: &Expr, _right: &Expr) {
        self.printed_str.push_str(")");
    }

    fn enter_exprs(&mut self, _exprs: &Vec<Expr>) {
        self.printed_str.push_str("{expressions ");
    }
    fn inter_exprs(&mut self, exprs: &Vec<Expr>, index: usize) {
        if index < exprs.len() - 1 {
            self.printed_str.push(',');
        }
    }
    fn exit_exprs(&mut self, _exprs: &Vec<Expr>) {
        self.printed_str.push('}');
    }
}

#[cfg(test)]
mod ast_printer_tests {
    use super::*;

    #[test]
    fn literals() {
        let mut ast_visitor = AstPrinter {
            printed_str: String::new(),
        };
        walk_expression(
            &Expr::Literal(LiteralValue::Number(12345.0)),
            &mut ast_visitor,
        );
        assert_eq!(ast_visitor.printed_str, "12345");

        ast_visitor.printed_str = String::new();
        walk_expression(
            &Expr::Literal(LiteralValue::String(String::from("some string"))),
            &mut ast_visitor,
        );
        assert_eq!(ast_visitor.printed_str, "\"some string\"");
    }

    #[test]
    fn unary_expression() {
        let unary_expression = &Expr::Unary {
            operator: Token {
                token_type: crate::scanner::TokenType::Bang,
                lexeme: String::from("!"),
                line: 1,
            },
            right: Box::new(Expr::Literal(LiteralValue::Number(12345.0))),
        };

        let mut ast_visitor = AstPrinter {
            printed_str: String::new(),
        };
        walk_expression(unary_expression, &mut ast_visitor);

        assert_eq!(ast_visitor.printed_str, "(! 12345)");
    }

    #[test]
    fn complex_expression() {
        let expr = &Expr::Binary {
            left: Box::new(Expr::Unary {
                operator: Token {
                    token_type: crate::scanner::TokenType::Minus,
                    lexeme: String::from("-"),
                    line: 1,
                },
                right: Box::new(Expr::Literal(LiteralValue::Number(123.0))),
            }),
            operator: Token {
                token_type: crate::scanner::TokenType::Star,
                lexeme: String::from("*"),
                line: 1,
            },
            right: Box::new(Expr::Grouping {
                expression: Box::new(Expr::Literal(LiteralValue::Number(45.67))),
            }),
        };

        let mut ast_visitor = AstPrinter {
            printed_str: String::new(),
        };
        walk_expression(expr, &mut ast_visitor);
        assert_eq!(ast_visitor.printed_str, "(* (- 123) (group 45.67))");
    }
}

//// end AST walking ////

//// Parse tokens into an expression AST ////
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>, String> {
        let mut statements: Vec<Stmt> = Vec::new();

        while !self.is_at_end() {
            let stmt = self.statement()?;
            statements.push(stmt);
        }

        Ok(statements)
    }

    fn statement(&mut self) -> Result<Stmt, String> {
        if self.peek().token_type == TokenType::Print {
            return self.print_statement();
        }

        self.expression_statement()
    }

    fn print_statement(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume the print statement
        let expr = self.expression()?;

        // check terminated with ';'
        match self.advance().token_type {
            TokenType::SemiColon => Ok(Stmt::Print(expr)),
            _ => Err(String::from("Expect ';' after expression.")),
        }
    }

    fn expression_statement(&mut self) -> Result<Stmt, String> {
        let expr = self.expression()?;

        // check terminated with ';'
        match self.advance().token_type {
            TokenType::SemiColon => Ok(Stmt::Expr(expr)),
            _ => Err(String::from("Expect ';' after expression.")),
        }
    }

    fn expression(&mut self) -> Result<Expr, String> {
        let head = self.ternary()?;
        let mut exprs = vec![head];

        while matches!(self.peek().token_type, TokenType::Comma) {
            self.advance();
            let expr = self.ternary()?;
            exprs.push(expr);
        }

        if exprs.len() == 1 {
            return Ok(exprs.pop().unwrap());
        }

        return Ok(Expr::Exprs(exprs));
    }

    fn ternary(&mut self) -> Result<Expr, String> {
        let left = self.equality()?;

        if self.peek().token_type != TokenType::QuestionMark {
            return Ok(left);
        }

        self.advance();
        let middle = self.ternary()?;

        if self.peek().token_type != TokenType::Colon {
            return Err(String::from("ternary missing : operator"));
        }

        self.advance();
        let right = self.ternary()?;

        return Ok(Expr::Ternary {
            left: Box::new(left),
            middle: Box::new(middle),
            right: Box::new(right),
        });
    }

    fn equality(&mut self) -> Result<Expr, String> {
        let mut expr = self.comparison()?;

        while matches!(
            self.peek().token_type,
            TokenType::EqualEqual | TokenType::BangEqual
        ) {
            let operator = self.advance().clone();
            let right = self.comparison()?;

            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        return Ok(expr);
    }

    fn comparison(&mut self) -> Result<Expr, String> {
        let mut expr = self.term()?;

        while matches!(
            self.peek().token_type,
            TokenType::Greater | TokenType::GreaterEqual | TokenType::Less | TokenType::LessEqual
        ) {
            let operator = self.advance().clone();
            let right = self.term()?;

            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        return Ok(expr);
    }

    fn term(&mut self) -> Result<Expr, String> {
        let mut expr = self.factor()?;

        while matches!(self.peek().token_type, TokenType::Plus | TokenType::Minus) {
            let operator = self.advance().clone();
            let right = self.factor()?;

            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        return Ok(expr);
    }

    fn factor(&mut self) -> Result<Expr, String> {
        let mut expr = self.unary()?;

        while matches!(self.peek().token_type, TokenType::Star | TokenType::Slash) {
            let operator = self.advance().clone();
            let right = self.unary()?;

            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        return Ok(expr);
    }

    fn unary(&mut self) -> Result<Expr, String> {
        if matches!(self.peek().token_type, TokenType::Bang | TokenType::Minus) {
            let operator = self.advance().clone();
            let right = self.unary()?;

            return Ok(Expr::Unary {
                operator,
                right: Box::new(right),
            });
        }

        return self.primary();
    }

    fn primary(&mut self) -> Result<Expr, String> {
        let token = self.peek();

        let result = match token.token_type {
            TokenType::False => {
                self.advance();
                Ok(Expr::Literal(LiteralValue::False))
            }
            TokenType::True => {
                self.advance();
                Ok(Expr::Literal(LiteralValue::True))
            }
            TokenType::Nil => {
                self.advance();
                Ok(Expr::Literal(LiteralValue::Nil))
            }
            TokenType::String => {
                let token = self.advance();
                Ok(Expr::Literal(LiteralValue::String(token.lexeme.clone())))
            }
            TokenType::Number => {
                let token = self.advance();
                match token.lexeme.parse::<f64>() {
                    Ok(val) => Ok(Expr::Literal(LiteralValue::Number(val))),
                    Err(_) => Err(String::from("Failed to parse literal number")),
                }
            }

            TokenType::LeftParen => {
                self.advance();
                let expr = self.expression()?;
                let maybe_right_paren = self.advance();

                if matches!(maybe_right_paren.token_type, TokenType::RightParen) {
                    return Ok(Expr::Grouping {
                        expression: Box::new(expr),
                    });
                }

                return Err(String::from("failed to find closing right paren"));
            }
            _ => Err(String::from("parse_primary() passed a non-literal Token!")),
        };

        return result;
    }

    // Parser panic mode error recovery
    // TODO: use me!
    fn synchronize(&mut self) {
        self.advance();

        // discard tokens until we hit a new statement
        while !self.is_at_end() {
            if self.previous().token_type == TokenType::SemiColon {
                return;
            }

            match self.peek().token_type {
                TokenType::Class => return,
                TokenType::Fun => return,
                TokenType::Var => return,
                TokenType::For => return,
                TokenType::If => return,
                TokenType::While => return,
                TokenType::Print => return,
                TokenType::Return => return,
                _ => (),
            }

            self.advance();
        }
    }

    /// Consume the current token (advances current index) -- returns the last token if stream is ended
    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current = self.current + 1;
        }

        return self.previous();
    }

    fn is_at_end(&self) -> bool {
        return self.peek().token_type == TokenType::EOF;
    }

    fn peek(&self) -> &Token {
        return &self.tokens[self.current];
    }

    fn previous(&self) -> &Token {
        return &self.tokens[self.current - 1];
    }
}

#[cfg(test)]
mod parser_tests {
    use super::*;

    #[test]
    fn empty_tokens() {
        let mut parser = Parser::new(vec![Token {
            token_type: TokenType::EOF,
            lexeme: String::from(""),
            line: 0,
        }]);

        assert!(parser.parse().is_ok());
    }

    #[test]
    fn simple_literals() -> Result<(), String> {
        let mut number_parser = Parser::new(vec![
            Token {
                token_type: TokenType::Number,
                lexeme: String::from("12345"),
                line: 0,
            },
            Token {
                token_type: TokenType::EOF,
                lexeme: String::from(""),
                line: 0,
            },
        ]);

        let mut ast = number_parser.expression()?;
        assert_eq!(ast, Expr::Literal(LiteralValue::Number(12345.0)));

        let mut string_parser = Parser::new(vec![
            Token {
                token_type: TokenType::String,
                lexeme: String::from("some_string"),
                line: 0,
            },
            Token {
                token_type: TokenType::EOF,
                lexeme: String::from(""),
                line: 0,
            },
        ]);

        ast = string_parser.expression()?;
        assert_eq!(
            ast,
            Expr::Literal(LiteralValue::String(String::from("some_string")))
        );

        return Ok(());
    }

    #[test]
    fn arithemtic() -> Result<(), String> {
        let mut ast_visitor = AstPrinter {
            printed_str: String::new(),
        };

        // 1 - 6 / 3
        let mut parser = Parser::new(vec![
            Token {
                token_type: TokenType::Number,
                lexeme: String::from("1"),
                line: 0,
            },
            Token {
                token_type: TokenType::Minus,
                lexeme: String::from("-"),
                line: 0,
            },
            Token {
                token_type: TokenType::Number,
                lexeme: String::from("6"),
                line: 0,
            },
            Token {
                token_type: TokenType::Slash,
                lexeme: String::from("/"),
                line: 0,
            },
            Token {
                token_type: TokenType::Number,
                lexeme: String::from("3"),
                line: 0,
            },
            Token {
                token_type: TokenType::EOF,
                lexeme: String::from(""),
                line: 0,
            },
        ]);

        // parse list of tokens into AST
        let ast = parser.expression()?;

        // check the result using the AST visitor
        walk_expression(&ast, &mut ast_visitor);
        assert_eq!(ast_visitor.printed_str, "(- 1 (/ 6 3))");

        return Ok(());
    }

    #[test]
    fn grouping() -> Result<(), String> {
        let mut ast_visitor = AstPrinter {
            printed_str: String::new(),
        };

        let mut parser = Parser::new(vec![
            Token {
                token_type: TokenType::LeftParen,
                lexeme: String::from("("),
                line: 0,
            },
            Token {
                token_type: TokenType::Number,
                lexeme: String::from("1"),
                line: 0,
            },
            Token {
                token_type: TokenType::Plus,
                lexeme: String::from("+"),
                line: 0,
            },
            Token {
                token_type: TokenType::Number,
                lexeme: String::from("2"),
                line: 0,
            },
            Token {
                token_type: TokenType::RightParen,
                lexeme: String::from(")"),
                line: 0,
            },
            Token {
                token_type: TokenType::EOF,
                lexeme: String::new(),
                line: 0,
            },
        ]);

        let ast = parser.expression()?;

        // check the result using the AST visitor
        walk_expression(&ast, &mut ast_visitor);
        assert_eq!(ast_visitor.printed_str, "(group (+ 1 2))");

        return Ok(());
    }

    #[test]
    fn commas() -> Result<(), String> {
        let mut ast_visitor = AstPrinter {
            printed_str: String::new(),
        };

        let mut parser = Parser::new(vec![
            Token {
                token_type: TokenType::Number,
                lexeme: String::from("1"),
                line: 0,
            },
            Token {
                token_type: TokenType::Minus,
                lexeme: String::from("+"),
                line: 0,
            },
            Token {
                token_type: TokenType::Number,
                lexeme: String::from("2"),
                line: 0,
            },
            Token {
                token_type: TokenType::Comma,
                lexeme: String::from(","),
                line: 0,
            },
            Token {
                token_type: TokenType::LeftParen,
                lexeme: String::from("("),
                line: 0,
            },
            Token {
                token_type: TokenType::Number,
                lexeme: String::from("3"),
                line: 0,
            },
            Token {
                token_type: TokenType::Minus,
                lexeme: String::from("+"),
                line: 0,
            },
            Token {
                token_type: TokenType::Number,
                lexeme: String::from("4"),
                line: 0,
            },
            Token {
                token_type: TokenType::RightParen,
                lexeme: String::from(")"),
                line: 0,
            },
            Token {
                token_type: TokenType::EOF,
                lexeme: String::from(""),
                line: 0,
            },
        ]);

        let ast = parser.expression()?;

        // check the result using the AST visitor
        walk_expression(&ast, &mut ast_visitor);
        assert_eq!(
            ast_visitor.printed_str,
            "{expressions (+ 1 2),(group (+ 3 4))}"
        );

        return Ok(());
    }

    #[test]
    fn ternary() -> Result<(), String> {
        let mut ast_visitor = AstPrinter {
            printed_str: String::new(),
        };

        let mut parser = Parser::new(vec![
            Token {
                token_type: TokenType::True,
                lexeme: String::from("true"),
                line: 0,
            },
            Token {
                token_type: TokenType::QuestionMark,
                lexeme: String::from("?"),
                line: 0,
            },
            Token {
                token_type: TokenType::Number,
                lexeme: String::from("1"),
                line: 0,
            },
            Token {
                token_type: TokenType::Plus,
                lexeme: String::from("+"),
                line: 0,
            },
            Token {
                token_type: TokenType::Number,
                lexeme: String::from("2"),
                line: 0,
            },
            Token {
                token_type: TokenType::Colon,
                lexeme: String::from(":"),
                line: 0,
            },
            Token {
                token_type: TokenType::Number,
                lexeme: String::from("3"),
                line: 0,
            },
            Token {
                token_type: TokenType::EOF,
                lexeme: String::from(""),
                line: 0,
            },
        ]);

        let ast = parser.expression()?;

        // check the result using the AST visitor
        walk_expression(&ast, &mut ast_visitor);
        assert_eq!(ast_visitor.printed_str, "(true ? (+ 1 2) : 3)");

        return Ok(());
    }

    #[test]
    fn multiple_ternarys() -> Result<(), String> {
        let mut ast_visitor = AstPrinter {
            printed_str: String::new(),
        };

        let mut parser = Parser::new(vec![
            Token {
                token_type: TokenType::True,
                lexeme: String::from("false"),
                line: 0,
            },
            Token {
                token_type: TokenType::QuestionMark,
                lexeme: String::from("?"),
                line: 0,
            },
            Token {
                token_type: TokenType::String,
                lexeme: String::from("first"),
                line: 0,
            },
            Token {
                token_type: TokenType::Colon,
                lexeme: String::from(":"),
                line: 0,
            },
            Token {
                token_type: TokenType::True,
                lexeme: String::from("true"),
                line: 0,
            },
            Token {
                token_type: TokenType::QuestionMark,
                lexeme: String::from("?"),
                line: 0,
            },
            Token {
                token_type: TokenType::String,
                lexeme: String::from("second"),
                line: 0,
            },
            Token {
                token_type: TokenType::Colon,
                lexeme: String::from(":"),
                line: 0,
            },
            Token {
                token_type: TokenType::String,
                lexeme: String::from("third"),
                line: 0,
            },
            Token {
                token_type: TokenType::EOF,
                lexeme: String::from(""),
                line: 0,
            },
        ]);

        let ast = parser.expression()?;

        // check the result using the AST visitor
        walk_expression(&ast, &mut ast_visitor);
        assert_eq!(
            ast_visitor.printed_str,
            "(true ? \"first\" : (true ? \"second\" : \"third\"))"
        );

        return Ok(());
    }
}

//// end AST parsing ////
