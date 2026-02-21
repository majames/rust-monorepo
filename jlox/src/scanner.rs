use std::{collections::HashMap, str::Chars};

use crate::utils::report_error;
use itertools::{PeekNth, peek_nth};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TokenType {
    // Single character tokens
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    SemiColon,
    Slash,
    Star,
    QuestionMark,
    Colon,

    // One or two char tokens
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Identifiers
    Identifier,
    String,
    Number,

    // Keywords
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    // End of input
    EOF,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub line: u64,
}

pub fn scan_tokens(source: &str) -> Vec<Token> {
    let keywords_map: HashMap<String, TokenType> = HashMap::from([
        (String::from("class"), TokenType::Class),
        (String::from("and"), TokenType::And),
        (String::from("else"), TokenType::Else),
        (String::from("false"), TokenType::False),
        (String::from("fun"), TokenType::Fun),
        (String::from("for"), TokenType::For),
        (String::from("if"), TokenType::If),
        (String::from("nil"), TokenType::Nil),
        (String::from("or"), TokenType::Or),
        (String::from("print"), TokenType::Print),
        (String::from("return"), TokenType::Return),
        (String::from("super"), TokenType::Super),
        (String::from("this"), TokenType::This),
        (String::from("true"), TokenType::True),
        (String::from("var"), TokenType::Var),
        (String::from("while"), TokenType::While),
    ]);

    let mut tokens: Vec<Token> = Vec::new();
    let mut iter = peek_nth(source.chars());
    let mut line: u64 = 1;

    while let Some(char) = iter.next() {
        let mut lexeme = String::new();
        lexeme.push(char);

        match char {
            // ignore whitespace
            ' ' => {}
            '\r' => {}
            '\t' => {}
            '\n' => {
                line = line + 1;
            }

            // "easy" single char tokens
            '(' => {
                tokens.push(Token {
                    token_type: TokenType::LeftParen,
                    lexeme,
                    line,
                });
            }
            ')' => {
                tokens.push(Token {
                    token_type: TokenType::RightParen,
                    lexeme,
                    line,
                });
            }
            '{' => {
                tokens.push(Token {
                    token_type: TokenType::LeftBrace,
                    lexeme,
                    line,
                });
            }
            '}' => {
                tokens.push(Token {
                    token_type: TokenType::RightBrace,
                    lexeme,
                    line,
                });
            }
            ',' => {
                tokens.push(Token {
                    token_type: TokenType::Comma,
                    lexeme,
                    line,
                });
            }
            '-' => {
                tokens.push(Token {
                    token_type: TokenType::Minus,
                    lexeme,
                    line,
                });
            }
            '+' => {
                tokens.push(Token {
                    token_type: TokenType::Plus,
                    lexeme,
                    line,
                });
            }
            ';' => {
                tokens.push(Token {
                    token_type: TokenType::SemiColon,
                    lexeme,
                    line,
                });
            }
            '*' => {
                tokens.push(Token {
                    token_type: TokenType::Star,
                    lexeme,
                    line,
                });
            }
            '.' => {
                tokens.push(Token {
                    token_type: TokenType::Dot,
                    lexeme,
                    line,
                });
            }
            '?' => {
                tokens.push(Token {
                    token_type: TokenType::QuestionMark,
                    lexeme,
                    line,
                });
            }
            ':' => {
                tokens.push(Token {
                    token_type: TokenType::Colon,
                    lexeme,
                    line,
                });
            }

            // these tokens maybe depend on the next char
            '!' => tokens.push(single_or_two_char_token(
                &mut iter,
                TokenType::Bang,
                TokenType::BangEqual,
                lexeme,
                line,
            )),
            '=' => tokens.push(single_or_two_char_token(
                &mut iter,
                TokenType::Equal,
                TokenType::EqualEqual,
                lexeme,
                line,
            )),
            '>' => tokens.push(single_or_two_char_token(
                &mut iter,
                TokenType::Greater,
                TokenType::GreaterEqual,
                lexeme,
                line,
            )),
            '<' => tokens.push(single_or_two_char_token(
                &mut iter,
                TokenType::Less,
                TokenType::LessEqual,
                lexeme,
                line,
            )),

            // the "/" could be division or the start of a comment line
            '/' => match iter.peek() {
                Some(&next_char) => {
                    if next_char == '/' {
                        // handle comment -- scan to end of the line
                        while let Some(&c) = iter.peek() {
                            if c == '\n' {
                                break;
                            }

                            iter.next();
                        }
                    } else {
                        tokens.push(Token {
                            token_type: TokenType::Slash,
                            lexeme,
                            line,
                        });
                    }
                }
                None => tokens.push(Token {
                    token_type: TokenType::Slash,
                    lexeme: char.to_string(),
                    line,
                }),
            },

            '"' => {
                let mut string_terminated = false;
                lexeme = String::new();

                while let Some(c) = iter.next() {
                    if c == '"' {
                        string_terminated = true;
                        break;
                    }

                    if c == '\n' {
                        line = line + 1;
                    }

                    lexeme.push(c);
                }

                if string_terminated {
                    tokens.push(Token {
                        token_type: TokenType::String,
                        lexeme,
                        line,
                    })
                } else {
                    report_error(line, "", "Unterminated string");
                }
            }

            d if d.is_ascii_digit() => {
                lexeme = d.to_string();

                while let Some(&c) = iter.peek() {
                    if !c.is_ascii_digit() {
                        // no longer a digit
                        break;
                    }

                    lexeme.push(c);
                    iter.next();
                }

                // check if number is followed by '.'
                let Some(&maybe_dot) = iter.peek() else {
                    tokens.push(Token {
                        token_type: TokenType::Number,
                        lexeme,
                        line,
                    });

                    continue;
                };

                // check if number contains trailing decimal
                let Some(&char_after_dot) = iter.peek_nth(1) else {
                    tokens.push(Token {
                        token_type: TokenType::Number,
                        lexeme,
                        line,
                    });

                    continue;
                };

                if maybe_dot == '.' && char_after_dot.is_ascii_digit() {
                    // handle trailing decimal
                    lexeme.push('.');
                    iter.next();

                    while let Some(&c) = iter.peek() {
                        if !c.is_ascii_digit() {
                            // no longer a digit
                            break;
                        }

                        lexeme.push(c);
                        iter.next();
                    }
                }

                tokens.push(Token {
                    token_type: TokenType::Number,
                    lexeme,
                    line,
                });
            }

            char => {
                if char.is_ascii_alphabetic() || char == '_' {
                    // identifier or keyword
                    while let Some(&c) = iter.peek() {
                        if !c.is_ascii_alphanumeric() && c != '_' {
                            break;
                        }

                        lexeme.push(c);
                        iter.next();
                    }

                    let token_type = match keywords_map.get(&lexeme) {
                        Some(token_type) => *token_type,
                        None => TokenType::Identifier,
                    };

                    tokens.push(Token {
                        token_type,
                        lexeme,
                        line,
                    });
                } else {
                    report_error(line, "", &format!("Unexpected character \"{}\"", char))
                }
            }
        }
    }

    // add EOF token
    tokens.push(Token {
        token_type: TokenType::EOF,
        lexeme: String::from(""),
        line,
    });

    return tokens;
}

fn single_or_two_char_token(
    iter: &mut PeekNth<Chars>,
    // peeked_char: Option<&char>,
    one_char_tt: TokenType,
    two_char_tt: TokenType,
    mut lexeme: String,
    line: u64,
) -> Token {
    return match iter.peek() {
        Some(&next_char) => {
            let mut token_type = one_char_tt;

            if next_char == '=' {
                token_type = two_char_tt;
                lexeme.push(next_char);

                // advance the iterator since we've "consumed" this char
                iter.next();
            }

            Token {
                token_type,
                lexeme,
                line,
            }
        }
        None => Token {
            token_type: one_char_tt,
            lexeme,
            line,
        },
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_source() {
        assert_eq!(
            scan_tokens(""),
            vec![Token {
                token_type: TokenType::EOF,
                lexeme: String::from(""),
                line: 1
            }]
        )
    }

    #[test]
    fn single_char_tokens() {
        assert_eq!(
            scan_tokens("( )"),
            vec![
                Token {
                    token_type: TokenType::LeftParen,
                    lexeme: String::from("("),
                    line: 1
                },
                Token {
                    token_type: TokenType::RightParen,
                    lexeme: String::from(")"),
                    line: 1
                },
                Token {
                    token_type: TokenType::EOF,
                    lexeme: String::from(""),
                    line: 1
                }
            ]
        );
    }

    #[test]
    fn one_or_two_char_tokens() {
        assert_eq!(
            scan_tokens("!== >==< <="),
            vec![
                Token {
                    token_type: TokenType::BangEqual,
                    lexeme: String::from("!="),
                    line: 1
                },
                Token {
                    token_type: TokenType::Equal,
                    lexeme: String::from("="),
                    line: 1
                },
                Token {
                    token_type: TokenType::GreaterEqual,
                    lexeme: String::from(">="),
                    line: 1
                },
                Token {
                    token_type: TokenType::Equal,
                    lexeme: String::from("="),
                    line: 1
                },
                Token {
                    token_type: TokenType::Less,
                    lexeme: String::from("<"),
                    line: 1
                },
                Token {
                    token_type: TokenType::LessEqual,
                    lexeme: String::from("<="),
                    line: 1
                },
                Token {
                    token_type: TokenType::EOF,
                    lexeme: String::from(""),
                    line: 1
                }
            ]
        );
    }

    #[test]
    fn strings() {
        assert_eq!(
            scan_tokens("\"some string\""),
            vec![
                Token {
                    token_type: TokenType::String,
                    lexeme: String::from("some string"),
                    line: 1
                },
                Token {
                    token_type: TokenType::EOF,
                    lexeme: String::from(""),
                    line: 1
                }
            ]
        );
    }

    #[test]
    fn tokenizes_numbers() {
        assert_eq!(
            scan_tokens("12345 6789.420 6969."),
            vec![
                Token {
                    token_type: TokenType::Number,
                    lexeme: String::from("12345"),
                    line: 1
                },
                Token {
                    token_type: TokenType::Number,
                    lexeme: String::from("6789.420"),
                    line: 1
                },
                Token {
                    token_type: TokenType::Number,
                    lexeme: String::from("6969"),
                    line: 1
                },
                Token {
                    token_type: TokenType::Dot,
                    lexeme: String::from("."),
                    line: 1
                },
                Token {
                    token_type: TokenType::EOF,
                    lexeme: String::from(""),
                    line: 1
                }
            ]
        );
    }

    #[test]
    fn identifiers_and_keywords() {
        assert_eq!(
            scan_tokens("class return else if some_identifier"),
            vec![
                Token {
                    token_type: TokenType::Class,
                    lexeme: String::from("class"),
                    line: 1
                },
                Token {
                    token_type: TokenType::Return,
                    lexeme: String::from("return"),
                    line: 1
                },
                Token {
                    token_type: TokenType::Else,
                    lexeme: String::from("else"),
                    line: 1
                },
                Token {
                    token_type: TokenType::If,
                    lexeme: String::from("if"),
                    line: 1
                },
                Token {
                    token_type: TokenType::Identifier,
                    lexeme: String::from("some_identifier"),
                    line: 1
                },
                Token {
                    token_type: TokenType::EOF,
                    lexeme: String::from(""),
                    line: 1
                }
            ]
        );
    }

    #[test]
    fn multi_line_source() {
        let s = "
(
      
)
";

        assert_eq!(
            scan_tokens(s),
            vec![
                Token {
                    token_type: TokenType::LeftParen,
                    lexeme: String::from("("),
                    line: 2
                },
                Token {
                    token_type: TokenType::RightParen,
                    lexeme: String::from(")"),
                    line: 4
                },
                Token {
                    token_type: TokenType::EOF,
                    lexeme: String::from(""),
                    line: 5
                }
            ]
        );
    }

    #[test]
    fn complex_case() {
        assert_eq!(
            scan_tokens(
                "() \"some word\" != !
// some comment
/
"
            ),
            vec![
                Token {
                    token_type: TokenType::LeftParen,
                    lexeme: String::from("("),
                    line: 1
                },
                Token {
                    token_type: TokenType::RightParen,
                    lexeme: String::from(")"),
                    line: 1
                },
                Token {
                    token_type: TokenType::String,
                    lexeme: String::from("some word"),
                    line: 1
                },
                Token {
                    token_type: TokenType::BangEqual,
                    lexeme: String::from("!="),
                    line: 1
                },
                Token {
                    token_type: TokenType::Bang,
                    lexeme: String::from("!"),
                    line: 1
                },
                Token {
                    token_type: TokenType::Slash,
                    lexeme: String::from("/"),
                    line: 3
                },
                Token {
                    token_type: TokenType::EOF,
                    lexeme: String::from(""),
                    line: 4
                }
            ]
        );
    }
}
