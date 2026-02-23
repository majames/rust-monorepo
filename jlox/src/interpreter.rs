use crate::parser::{Expr, LiteralValue, Stmt};
use crate::scanner::TokenType;

pub fn interpret(statements: Vec<Stmt>) -> Result<(), String> {
    for statement in statements {
        match statement {
            Stmt::Expr(expr) => {
                evaluate(&expr)?;
            }
            Stmt::Print(expr) => {
                let val = evaluate(&expr)?;
                println!("{}", val);
            }
        };
    }

    Ok(())
}

fn evaluate(expr: &Expr) -> Result<LiteralValue, String> {
    match expr {
        Expr::Literal(val) => Ok(val.clone()),
        Expr::Grouping { expression } => evaluate(expression),
        Expr::Unary { operator, right } => {
            let literal = evaluate(right)?;

            match operator.token_type {
                TokenType::Minus => match literal {
                    LiteralValue::Number(val) => Ok(LiteralValue::Number(-1.0 * val)),
                    _ => Err(String::from("illegal negation of non-number value")),
                },
                TokenType::Bang => Ok(is_truthy(literal)),
                _ => Err(String::from("invalid unary operator")),
            }
        }
        Expr::Binary {
            left,
            operator,
            right,
        } => {
            let left_literal = evaluate(left)?;
            let right_literal = evaluate(right)?;

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
            let condition = evaluate(left)?;

            match condition {
                LiteralValue::True => evaluate(middle),
                LiteralValue::False => evaluate(right),
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

            let mut result = evaluate(first)?;
            for expr in iter {
                result = evaluate(expr)?;
            }
            return Ok(result);
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

fn is_truthy(literal: LiteralValue) -> LiteralValue {
    // everything but false and nil is considered truthy
    match literal {
        LiteralValue::False => LiteralValue::False,
        LiteralValue::Nil => LiteralValue::False,
        _ => LiteralValue::True,
    }
}
