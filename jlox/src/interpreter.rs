use std::collections::HashMap;

use crate::parser::{Expr, LiteralValue, Stmt};
use crate::scanner::{Token, TokenType};

struct Environment {
    values: HashMap<String, Option<LiteralValue>>,
    enclosing: Option<Box<Environment>>,
}

impl Environment {
    fn new() -> Self {
        Self {
            values: HashMap::new(),
            enclosing: None,
        }
    }

    fn from_enclosing(enclosing: Environment) -> Self {
        Self {
            values: HashMap::new(),
            enclosing: Some(Box::new(enclosing)),
        }
    }

    fn define(&mut self, name: String, maybe_value: Option<LiteralValue>) {
        self.values.insert(name, maybe_value);
    }

    fn assign(&mut self, name: &str, value: LiteralValue) -> Result<(), String> {
        if self.values.contains_key(name) {
            self.values.insert(name.to_string(), Some(value));
            return Ok(());
        }

        match self.enclosing.as_mut() {
            Some(enclosing_env) => enclosing_env.assign(name, value),
            None => Err(String::from("Undefined variable '") + &name + "'."),
        }
    }

    fn get(&self, token: &Token) -> Result<LiteralValue, String> {
        match self.values.get(&token.lexeme) {
            Some(val) => match val {
                Some(value) => Ok(value.clone()),
                None => Err(format!("Variable '{}' uninitialized", token.lexeme)),
            },
            None => match &self.enclosing {
                Some(enclosing_env) => enclosing_env.get(token),
                None => Err(String::from("Undefined variable '") + &token.lexeme + "'."),
            },
        }
    }
}

pub struct Interpreter {
    environment: Environment,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            environment: Environment::new(),
        }
    }

    pub fn interpret(&mut self, statements: Vec<&Stmt>) -> Result<(), String> {
        for statement in statements {
            match statement {
                Stmt::Expr(expr) => {
                    self.evaluate_expr(&expr)?;
                }
                Stmt::Print(expr) => {
                    let val = self.evaluate_expr(&expr)?;
                    println!("{}", val);
                }
                Stmt::VarDeclaration(var_declaration) => {
                    let maybe_value = match &var_declaration.initializer {
                        Some(expr) => Some(self.evaluate_expr(expr)?),
                        None => None,
                    };

                    self.environment
                        .define(var_declaration.name.lexeme.clone(), maybe_value);
                }
                Stmt::Block(stmts) => {
                    let prev_env = std::mem::replace(&mut self.environment, Environment::new());

                    // create new inner env
                    self.environment = Environment::from_enclosing(prev_env);
                    let result = self.interpret(stmts.iter().collect());

                    // restore old enclosing env
                    let inner_env = std::mem::replace(&mut self.environment, Environment::new());
                    if let Some(enclosing_env) = inner_env.enclosing {
                        self.environment = *enclosing_env;
                    } else {
                        return Err(String::from(
                            "failed to restore enclosing environment scope",
                        ));
                    }

                    result?;
                }
                Stmt::If {
                    condition,
                    if_branch,
                    else_branch,
                } => {
                    let evaled_cond = self.evaluate_expr(&condition)?;

                    if is_truthy(&evaled_cond) {
                        return self.interpret(vec![if_branch]);
                    } else {
                        match else_branch {
                            Some(else_stmt) => self.interpret(vec![else_stmt])?,
                            None => { /* do nothing */ }
                        }
                    }
                }
                Stmt::While { condition, body } => {
                    while is_truthy(&self.evaluate_expr(&condition)?) {
                        self.interpret(vec![body])?;
                    }
                }
            };
        }

        Ok(())
    }

    pub fn evaluate_expr(&mut self, expr: &Expr) -> Result<LiteralValue, String> {
        match expr {
            Expr::Literal(val) => Ok(val.clone()),
            Expr::Grouping { expression } => self.evaluate_expr(expression),
            Expr::Unary { operator, right } => {
                let literal = self.evaluate_expr(right)?;

                match operator.token_type {
                    TokenType::Minus => match literal {
                        LiteralValue::Number(val) => Ok(LiteralValue::Number(-1.0 * val)),
                        _ => Err(String::from("illegal negation of non-number value")),
                    },
                    TokenType::Bang => Ok(LiteralValue::from_bool(is_truthy(&literal))),
                    _ => Err(String::from("invalid unary operator")),
                }
            }
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                let left_literal = self.evaluate_expr(left)?;
                let right_literal = self.evaluate_expr(right)?;

                match operator.token_type {
                    // arithmetic
                    TokenType::Minus => {
                        let result = parse_number_literals(&vec![left_literal, right_literal])?;
                        Ok(LiteralValue::Number(result[0] - result[1]))
                    }
                    TokenType::Star => {
                        let result = parse_number_literals(&vec![left_literal, right_literal])?;
                        Ok(LiteralValue::Number(result[0] * result[1]))
                    }
                    TokenType::Slash => {
                        let result = parse_number_literals(&vec![left_literal, right_literal])?;

                        if result[1] < f64::EPSILON {
                            return Err(String::from("can NOT divide by zero"));
                        }

                        Ok(LiteralValue::Number(result[0] / result[1]))
                    }
                    TokenType::Plus => {
                        let literals = vec![left_literal, right_literal];

                        if let Ok(result) = parse_number_literals(&literals) {
                            return Ok(LiteralValue::Number(result[0] + result[1]));
                        }

                        if let Ok(result) = parse_string_literals(&literals) {
                            return Ok(LiteralValue::String(String::new() + result[0] + result[1]));
                        }

                        Err(String::from(
                            // TODO: improve this error message
                            "Can NOT perform addition on literals of type {:?} and {:?}",
                        ))
                    }

                    // comparison
                    TokenType::Greater => {
                        let result = parse_number_literals(&vec![left_literal, right_literal])?;
                        Ok(LiteralValue::from_bool(result[0] > result[1]))
                    }
                    TokenType::GreaterEqual => {
                        let result = parse_number_literals(&vec![left_literal, right_literal])?;
                        Ok(LiteralValue::from_bool(result[0] >= result[1]))
                    }
                    TokenType::Less => {
                        let result = parse_number_literals(&vec![left_literal, right_literal])?;
                        Ok(LiteralValue::from_bool(result[0] < result[1]))
                    }
                    TokenType::LessEqual => {
                        let result = parse_number_literals(&vec![left_literal, right_literal])?;
                        Ok(LiteralValue::from_bool(result[0] <= result[1]))
                    }

                    _ => Err(String::from(format!(
                        "Can NOT perform '{:?}' on numbers",
                        operator.token_type
                    ))),
                }
            }
            Expr::Ternary {
                left,
                middle,
                right,
            } => {
                let condition = self.evaluate_expr(left)?;

                match condition {
                    LiteralValue::True => self.evaluate_expr(middle),
                    LiteralValue::False => self.evaluate_expr(right),
                    other => Err(String::from(format!(
                        "Can NOT use non-boolean value, {:?}, in ternary condition",
                        other
                    ))),
                }
            }
            Expr::Exprs(exprs) => {
                let mut iter = exprs.iter();

                let Some(first) = iter.next() else {
                    return Err(String::from("List of expressions is empty"));
                };

                let mut result = self.evaluate_expr(first)?;
                for expr in iter {
                    result = self.evaluate_expr(expr)?;
                }
                return Ok(result);
            }
            Expr::Variable { name } => self.environment.get(name),
            Expr::Assignment { name, value } => {
                let v = self.evaluate_expr(value)?;
                self.environment.assign(&name.lexeme, v.clone())?;
                return Ok(v);
            }
            Expr::Or { left, right } => {
                let result = self.evaluate_expr(left)?;

                if is_truthy(&result) {
                    return Ok(result);
                }

                return self.evaluate_expr(right);
            }
            Expr::And { left, right } => {
                let result = self.evaluate_expr(left)?;

                if is_truthy(&result) {
                    return self.evaluate_expr(right);
                }

                return Ok(result);
            }
        }
    }
}

fn parse_number_literals(literal_values: &[LiteralValue]) -> Result<Vec<f64>, String> {
    let mut result: Vec<f64> = vec![];

    for literal_value in literal_values {
        let LiteralValue::Number(val) = literal_value else {
            return Err(String::from("Found non-number literal"));
        };

        result.push(*val);
    }

    return Ok(result);
}

fn parse_string_literals(literal_values: &[LiteralValue]) -> Result<Vec<&String>, String> {
    let mut result: Vec<&String> = vec![];

    for literal_value in literal_values {
        let LiteralValue::String(val) = literal_value else {
            return Err(String::from("Found non-number literal"));
        };

        result.push(val);
    }

    return Ok(result);
}

fn is_truthy(literal: &LiteralValue) -> bool {
    // everything but false and nil is considered truthy
    match literal {
        LiteralValue::False => false,
        LiteralValue::Nil => false,
        _ => true,
    }
}
