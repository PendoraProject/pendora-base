use std::iter::Peekable;

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Integer(u32),
    Boolean(bool),
    StringLiteral(String),
    Word(String),
    Encapsulator(char),
    Split(char),
}

pub fn tokenise(input: String) -> Vec<Token> {
    let mut cursor = input.chars().peekable();
    let mut result: Vec<Token> = Vec::new();

    while let Some(&c) = cursor.peek() {
        match c {
            'A'..='Z' | 'a'..='z' => {
                let mut word = String::new();
                loop {
                    let ch = cursor.peek().unwrap();
                    match ch {
                        'A'..='Z' | 'a'..='z' | '_' | '-' | '?' | '.' => {
                            word.push(*ch);
                            cursor.next();
                        }
                        _ => break,
                    }
                }
                match word.as_str() {
                    "True" | "true" => result.push(Token::Boolean(true)),
                    "False" | "false" => result.push(Token::Boolean(false)),
                    _ => result.push(Token::Word(word)),
                }
            }
            '"' => {
                let mut str_lit = String::new();
                cursor.next();
                loop {
                    let ch = cursor.peek().unwrap();
                    match ch {
                        '"' => {
                            cursor.next();
                            break;
                        }
                        _ => {
                            str_lit.push(*ch);
                            cursor.next();
                        }
                    }
                }
                result.push(Token::StringLiteral(str_lit))
            }
            '0'..='9' => {
                cursor.next();
                let n = get_number(c, &mut cursor);
                result.push(Token::Integer(n));
            }
            '(' | ')' | '{' | '}' | '<' | '>' | '[' | ']' => {
                result.push(Token::Encapsulator(c));
                cursor.next();
            }
            ':' | ',' | ';' => {
                result.push(Token::Split(c));
                cursor.next();
            }
            _ => {
                cursor.next();
            }
        }
    }

    result
}

fn get_number<T: Iterator<Item = char>>(c: char, iter: &mut Peekable<T>) -> u32 {
    let mut number = c
        .to_string()
        .parse::<u32>()
        .expect("The caller should have passed a digit.");
    while let Some(Ok(digit)) = iter.peek().map(|c| c.to_string().parse::<u32>()) {
        number = number * 10 + digit;
        iter.next();
    }
    number
}
