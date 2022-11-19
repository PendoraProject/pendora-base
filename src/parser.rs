use std::{collections::HashMap, fs::read_to_string};
use walkdir::WalkDir;

use crate::{errors::*, token::*, types::*};

pub fn parse_project(root: &str) -> Result<Project, ParserError> {
    let mut global: Option<Global> = None;
    let mut objects: HashMap<String, Object> = HashMap::new();
    let mut methods: HashMap<String, Method> = HashMap::new();

    for entry in WalkDir::new(root) {
        let entry = entry.unwrap();
        let path = entry.path();
        let file_name = entry.file_name().to_str().unwrap();

        if entry.file_type().is_file() && file_name.ends_with(".pendora") {
            println!("{}", path.display());

            let input = read_to_string(path).unwrap();

            let tokens = tokenise(input);

            match &tokens[0] {
                Token::Word(w) => match w.as_str() {
                    "Global" => global = Some(parse_global(tokens)?),
                    "Method" => {
                        let method = parse_method(tokens)?;
                        let method_name = &method.name;
                        methods.insert(method_name.to_string(), method);
                    }
                    "Object" => {
                        let object = parse_object(tokens)?;
                        let object_name = &object.name;
                        objects.insert(object_name.to_string(), object);
                    }
                    _ => {
                        return Err(ParserError::InvalidSymbolBody {
                            location: ParserErrorLocation::Project {
                                file_name: path.display().to_string(),
                            },
                            incorrect_symbol: Token::Word(w.to_string()),
                            valid_symbols: vec![
                                "Global".to_string(),
                                "Object".to_string(),
                                "Method".to_string(),
                            ],
                        })
                    }
                },
                _ => {
                    let incorrect = tokens[0].clone();
                    return Err(ParserError::MisplacedSymbol {
                        location: ParserErrorLocation::Project {
                            file_name: path.display().to_string(),
                        },
                        incorrect_symbol: incorrect,
                        correct_symbol: Token::Word(String::new()),
                    });
                }
            }
        }
    }

    let true_global = match global {
        Some(g) => g,
        None => {
            return Err(ParserError::FieldNotExistent {
                location: ParserErrorLocation::Project {
                    file_name: String::from("entire project"),
                },
                missing_field: String::from("Global"),
            })
        }
    };

    Ok(Project {
        global: true_global,
        objects,
        methods,
    })
}

pub fn parse_method(input: Vec<Token>) -> Result<Method, ParserError> {
    let name: String;
    let arguments: MethodArguments;
    let route: String;
    let request_shape: RequestShape;
    let request_type: RequestType;
    let return_shape: ReturnShape;
    let return_object: String;

    let mut cursor = input.into_iter().peekable();
    if cursor.peek().unwrap() != &Token::Word(String::from("Method")) {
        return Err(ParserError::InvalidSymbolBody {
            location: ParserErrorLocation::Method,
            incorrect_symbol: cursor.peek().unwrap().clone(),
            valid_symbols: vec!["Method".to_string()],
        });
    }
    cursor.next();

    // destructure name
    match cursor.peek().unwrap().clone() {
        Token::Word(w) => name = w,
        _ => {
            return Err(ParserError::MisplacedSymbol {
                location: ParserErrorLocation::Method,
                incorrect_symbol: cursor.peek().unwrap().clone(),
                correct_symbol: Token::Word(String::from("method_name")),
            })
        }
    }
    cursor.next();

    if cursor.peek().unwrap() != &Token::Encapsulator('(') {
        return Err(ParserError::PoorClosure {
            location: ParserErrorLocation::Method,
            incorrect_encap: cursor.peek().unwrap().clone(),
            correct_encap: Token::Encapsulator('('),
        });
    }
    cursor.next();

    let mut arg_internal: Vec<Token> = Vec::new();
    loop {
        let t = cursor.peek().unwrap();
        match t {
            Token::Encapsulator(')') => {
                cursor.next();
                break;
            }
            _ => {
                arg_internal.push(t.clone());
                cursor.next();
            }
        }
    }
    arguments = parse_method_arguments(arg_internal)?;

    if cursor.peek().unwrap() != &Token::Encapsulator('{') {
        return Err(ParserError::PoorClosure {
            location: ParserErrorLocation::Method,
            incorrect_encap: cursor.peek().unwrap().clone(),
            correct_encap: Token::Encapsulator('{'),
        });
    }
    cursor.next();

    let mut method_internal: Vec<Token> = Vec::new();
    loop {
        let t = cursor.peek().unwrap();
        match t {
            Token::Split(';') => {
                method_internal.remove(method_internal.len() - 1);
                cursor.next();
                break;
            }
            _ => {
                method_internal.push(t.clone());
                cursor.next();
            }
        }
    }

    let internal = parse_method_internal(method_internal)?;

    route = internal.route;
    request_shape = internal.request_shape;
    request_type = internal.request_type;
    return_object = internal.return_object;
    return_shape = internal.return_shape;

    Ok(Method {
        name,
        arguments,
        route,
        request_shape,
        request_type,
        return_object,
        return_shape,
    })
}

fn parse_method_arguments(input: Vec<Token>) -> Result<MethodArguments, ParserError> {
    let mut result = MethodArguments::new();
    for chunk in input.chunks(3) {
        let arg_name: String;
        let arg_type: Type;
        match chunk.len() {
            3 => {
                match &chunk[0] {
                    Token::Word(w) => arg_type = parse_type(w.to_string()).unwrap(),
                    _ => {
                        return Err(ParserError::MisplacedSymbol {
                            location: ParserErrorLocation::MethodArguments,
                            incorrect_symbol: chunk[0].clone(),
                            correct_symbol: Token::Word(String::from("argument_type")),
                        });
                    }
                }
                match &chunk[1] {
                    Token::Word(w) => arg_name = w.to_string(),
                    _ => {
                        return Err(ParserError::MisplacedSymbol {
                            location: ParserErrorLocation::MethodArguments,
                            incorrect_symbol: chunk[1].clone(),
                            correct_symbol: Token::Word(String::from("argument_name")),
                        });
                    }
                }
                match &chunk[2] {
                    Token::Split(',') => {}
                    _ => {
                        return Err(ParserError::MisplacedSymbol {
                            location: ParserErrorLocation::MethodArguments,
                            incorrect_symbol: chunk[2].clone(),
                            correct_symbol: Token::Split(','),
                        })
                    }
                }
            }
            2 => {
                match &chunk[0] {
                    Token::Word(w) => arg_type = parse_type(w.to_string()).unwrap(),
                    _ => {
                        return Err(ParserError::MisplacedSymbol {
                            location: ParserErrorLocation::MethodArguments,
                            incorrect_symbol: chunk[0].clone(),
                            correct_symbol: Token::Word(String::from("argument_type")),
                        });
                    }
                }
                match &chunk[1] {
                    Token::Word(w) => arg_name = w.to_string(),
                    _ => {
                        return Err(ParserError::MisplacedSymbol {
                            location: ParserErrorLocation::MethodArguments,
                            incorrect_symbol: chunk[1].clone(),
                            correct_symbol: Token::Word(String::from("argument_name")),
                        });
                    }
                }
            }
            _ => {
                return Err(ParserError::BadLength {
                    location: ParserErrorLocation::MethodArguments,
                    incorrect_length: chunk.len(),
                    valid_lengths: vec![3, 2],
                });
            }
        }
        result.insert(arg_name, arg_type);
    }
    Ok(result)
}

#[derive(Debug)]
struct MethodInternal {
    route: String,
    request_shape: RequestShape,
    request_type: RequestType,
    return_shape: ReturnShape,
    return_object: String,
}

fn parse_method_internal(input: Vec<Token>) -> Result<MethodInternal, ParserError> {
    let mut route: String = String::new();
    let mut request_shape: RequestShape = RequestShape::new();
    let mut request_type: RequestType = RequestType::GET;
    let mut return_shape: ReturnShape = ReturnShape::new();
    let mut return_object: String = String::new();

    let mut cursor = input.into_iter().peekable();

    match cursor.peek().unwrap() {
        Token::Word(_) => {}
        _ => {
            return Err(ParserError::MisplacedSymbol {
                location: ParserErrorLocation::MethodInternal,
                incorrect_symbol: cursor.peek().unwrap().to_owned(),
                correct_symbol: Token::Word(String::from("")),
            });
        }
    }

    while let Some(Token::Word(w)) = cursor.peek() {
        match w.as_str() {
            "route" => {
                cursor.next();
                if cursor.peek().unwrap() != &Token::Encapsulator('(') {
                    return Err(ParserError::PoorClosure {
                        location: ParserErrorLocation::MethodInternal,
                        incorrect_encap: cursor.peek().unwrap().to_owned(),
                        correct_encap: Token::Encapsulator('('),
                    });
                }
                cursor.next();
                let mut route_internal: Vec<Token> = Vec::new();
                loop {
                    let t = cursor.peek().unwrap();
                    match t {
                        Token::Encapsulator(')') => {
                            cursor.next();
                            break;
                        }
                        _ => {
                            route_internal.push(t.clone());
                            cursor.next();
                        }
                    }
                }
                match &route_internal[0] {
                    Token::StringLiteral(str_lit) => route = str_lit.to_string(),
                    _ => {
                        return Err(ParserError::MisplacedSymbol {
                            location: ParserErrorLocation::MethodInternal,
                            incorrect_symbol: route_internal[0].clone(),
                            correct_symbol: Token::StringLiteral(String::from("route")),
                        })
                    }
                }
            }
            "request" => {
                cursor.next();
                if cursor.peek().unwrap() != &Token::Encapsulator('<') {
                    return Err(ParserError::PoorClosure {
                        location: ParserErrorLocation::MethodInternal,
                        incorrect_encap: cursor.peek().unwrap().to_owned(),
                        correct_encap: Token::Encapsulator('<'),
                    });
                }
                cursor.next();
                let mut request_type_internal: Vec<Token> = Vec::new();
                loop {
                    let t = cursor.peek().unwrap();
                    match t {
                        Token::Encapsulator('>') => {
                            cursor.next();
                            break;
                        }
                        _ => {
                            request_type_internal.push(t.clone());
                            cursor.next();
                        }
                    }
                }

                match &request_type_internal[0] {
                    Token::Word(w) => request_type = parse_request_type(w.to_string())?,
                    _ => {
                        return Err(ParserError::MisplacedSymbol {
                            location: ParserErrorLocation::MethodInternal,
                            incorrect_symbol: request_type_internal[0].to_owned(),
                            correct_symbol: Token::Word(String::from("request_type")),
                        })
                    }
                }

                if cursor.peek().unwrap() != &Token::Encapsulator('(') {
                    return Err(ParserError::PoorClosure {
                        location: ParserErrorLocation::MethodInternal,
                        incorrect_encap: cursor.peek().unwrap().to_owned(),
                        correct_encap: Token::Encapsulator('('),
                    });
                }
                cursor.next();
                let mut request_shape_internal: Vec<Token> = Vec::new();
                loop {
                    let t = cursor.peek().unwrap();
                    match t {
                        Token::Encapsulator(')') => {
                            cursor.next();
                            break;
                        }
                        _ => {
                            request_shape_internal.push(t.clone());
                            cursor.next();
                        }
                    }
                }
                request_shape = parse_request_shape(request_shape_internal)?;
            }
            "return" => {
                cursor.next();
                if cursor.peek().unwrap() != &Token::Encapsulator('<') {
                    return Err(ParserError::PoorClosure {
                        location: ParserErrorLocation::MethodInternal,
                        incorrect_encap: cursor.peek().unwrap().to_owned(),
                        correct_encap: Token::Encapsulator('('),
                    });
                }
                cursor.next();
                let mut return_object_internal: Vec<Token> = Vec::new();
                loop {
                    let t = cursor.peek().unwrap();
                    match t {
                        Token::Encapsulator('>') => {
                            cursor.next();
                            break;
                        }
                        _ => {
                            return_object_internal.push(t.clone());
                            cursor.next();
                        }
                    }
                }
                match &return_object_internal[0] {
                    Token::Word(w) => return_object = w.to_string(),
                    _ => {
                        return Err(ParserError::MisplacedSymbol {
                            location: ParserErrorLocation::MethodInternal,
                            incorrect_symbol: return_object_internal[0].to_owned(),
                            correct_symbol: Token::Word(String::from("return_object")),
                        })
                    }
                }

                if cursor.peek().unwrap() != &Token::Encapsulator('(') {
                    return Err(ParserError::PoorClosure {
                        location: ParserErrorLocation::MethodInternal,
                        incorrect_encap: cursor.peek().unwrap().to_owned(),
                        correct_encap: Token::Encapsulator('('),
                    });
                };
                cursor.next();
                let mut return_shape_internal: Vec<Token> = Vec::new();
                loop {
                    let t = cursor.peek().unwrap();
                    match t {
                        Token::Encapsulator(')') => {
                            cursor.next();
                            break;
                        }
                        _ => {
                            return_shape_internal.push(t.clone());
                            cursor.next();
                        }
                    }
                }
                return_shape = parse_return_shape(return_shape_internal)?;
            }
            _ => {
                return Err(ParserError::InvalidSymbolBody {
                    location: ParserErrorLocation::MethodInternal,
                    incorrect_symbol: cursor.peek().unwrap().to_owned(),
                    valid_symbols: vec![
                        "route".to_string(),
                        "request".to_string(),
                        "return".to_string(),
                    ],
                });
            }
        }
    }

    Ok(MethodInternal {
        route,
        request_shape,
        request_type,
        return_shape,
        return_object,
    })
}

fn parse_type(input: String) -> Result<Type, ParserError> {
    match input.as_str() {
        "int" | "Integer" => Ok(Type::Integer),
        "bool" | "Boolean" => Ok(Type::Boolean),
        "str" | "String" => Ok(Type::String),
        "int?" | "Integer?" => Ok(Type::NullableInteger),
        "bool?" | "Boolean?" => Ok(Type::NullableBoolean),
        "str?" | "String?" => Ok(Type::NullableString),
        _ => Err(ParserError::InvalidSymbolBody {
            location: ParserErrorLocation::Type,
            incorrect_symbol: Token::Word(input),
            valid_symbols: vec![
                "int".to_string(),
                "bool".to_string(),
                "str".to_string(),
                "Integer".to_string(),
                "Boolean".to_string(),
                "String".to_string(),
            ],
        }),
    }
}

fn parse_request_type(input: String) -> Result<RequestType, ParserError> {
    match input.as_str() {
        "GET" => Ok(RequestType::GET),
        "POST" => Ok(RequestType::POST),
        "PATCH" => Ok(RequestType::PATCH),
        "DELETE" => Ok(RequestType::DELETE),
        _ => Err(ParserError::InvalidSymbolBody {
            location: ParserErrorLocation::RequestType,
            incorrect_symbol: Token::Word(input),
            valid_symbols: vec![
                "GET".to_string(),
                "POST".to_string(),
                "PATCH".to_string(),
                "DELETE".to_string(),
            ],
        }),
    }
}

fn parse_request_shape(input: Vec<Token>) -> Result<RequestShape, ParserError> {
    let mut cursor = input.into_iter().peekable();
    let mut result = RequestShape::new();

    if cursor.peek().unwrap() != &Token::Encapsulator('{') {
        return Err(ParserError::PoorClosure {
            location: ParserErrorLocation::RequestShape,
            incorrect_encap: cursor.peek().unwrap().to_owned(),
            correct_encap: Token::Encapsulator('{'),
        });
    }
    cursor.next();
    let mut request_shape_hashmap: Vec<Token> = Vec::new();
    loop {
        let t = cursor.peek().unwrap();
        match t {
            Token::Encapsulator('}') => {
                cursor.next();
                break;
            }
            _ => {
                request_shape_hashmap.push(t.clone());
                cursor.next();
            }
        }
    }

    for chunk in request_shape_hashmap.chunks(4) {
        let key_name: String;
        let value: Value;
        match chunk.len() {
            4 => {
                match &chunk[0] {
                    Token::Word(w) => key_name = w.to_string(),
                    _ => {
                        return Err(ParserError::MisplacedSymbol {
                            location: ParserErrorLocation::RequestShape,
                            incorrect_symbol: chunk[0].to_owned(),
                            correct_symbol: Token::Word(String::from("param_name")),
                        })
                    }
                }
                match &chunk[1] {
                    Token::Split(':') => {}
                    _ => {
                        return Err(ParserError::MisplacedSymbol {
                            location: ParserErrorLocation::RequestShape,
                            incorrect_symbol: chunk[1].to_owned(),
                            correct_symbol: Token::Split(':'),
                        })
                    }
                }
                match &chunk[2] {
                    Token::Word(w) => value = parse_method_shape_value(w.to_string()),
                    _ => {
                        return Err(ParserError::MisplacedSymbol {
                            location: ParserErrorLocation::RequestShape,
                            incorrect_symbol: chunk[2].to_owned(),
                            correct_symbol: Token::Word(String::from("param_value")),
                        })
                    }
                }
                match &chunk[3] {
                    Token::Split(',') => {}
                    _ => {
                        return Err(ParserError::MisplacedSymbol {
                            location: ParserErrorLocation::RequestShape,
                            incorrect_symbol: chunk[3].to_owned(),
                            correct_symbol: Token::Split(','),
                        })
                    }
                }
            }
            3 => {
                match &chunk[0] {
                    Token::Word(w) => key_name = w.to_string(),
                    _ => {
                        return Err(ParserError::MisplacedSymbol {
                            location: ParserErrorLocation::RequestShape,
                            incorrect_symbol: chunk[0].to_owned(),
                            correct_symbol: Token::Word(String::from("param_name")),
                        })
                    }
                }
                match &chunk[1] {
                    Token::Split(':') => {}
                    _ => {
                        return Err(ParserError::MisplacedSymbol {
                            location: ParserErrorLocation::RequestShape,
                            incorrect_symbol: chunk[1].to_owned(),
                            correct_symbol: Token::Split(':'),
                        })
                    }
                }
                match &chunk[2] {
                    Token::Word(w) => value = parse_method_shape_value(w.to_string()),
                    _ => {
                        return Err(ParserError::MisplacedSymbol {
                            location: ParserErrorLocation::RequestShape,
                            incorrect_symbol: chunk[2].to_owned(),
                            correct_symbol: Token::Word(String::from("param_value")),
                        })
                    }
                }
            }
            _ => {
                return Err(ParserError::BadLength {
                    location: ParserErrorLocation::RequestShape,
                    incorrect_length: chunk.len(),
                    valid_lengths: vec![4, 3],
                })
            }
        }
        result.insert(key_name, value);
    }

    Ok(result)
}

fn parse_method_shape_value(input: String) -> Value {
    if input.starts_with("GLOBAL.") {
        let val = input.strip_prefix("GLOBAL.");
        return Value::Global(val.unwrap().to_string());
    } else if input.starts_with("PARENT.") {
        let val = input.strip_prefix("PARENT.");
        return Value::Parent(val.unwrap().to_string());
    } else {
        return Value::Argument(input);
    }
}

fn parse_return_shape(input: Vec<Token>) -> Result<ReturnShape, ParserError> {
    let mut cursor = input.into_iter().peekable();
    let mut result = ReturnShape::new();

    if cursor.peek().unwrap() != &Token::Encapsulator('{') {
        return Err(ParserError::PoorClosure {
            location: ParserErrorLocation::ReturnShape,
            incorrect_encap: cursor.peek().unwrap().to_owned(),
            correct_encap: Token::Encapsulator('{'),
        });
    }
    cursor.next();
    let mut return_shape_hashmap: Vec<Token> = Vec::new();
    loop {
        let t = cursor.peek().unwrap();
        match t {
            Token::Encapsulator('}') => {
                cursor.next();
                break;
            }
            _ => {
                return_shape_hashmap.push(t.clone());
                cursor.next();
            }
        }
    }
    for chunk in return_shape_hashmap.split(|t| match t {
        Token::Split(',') => true,
        _ => false,
    }) {
        match chunk.len() {
            1 => match &chunk[0] {
                Token::Word(w) => {
                    result.insert(w.to_string(), None);
                }
                _ => {
                    return Err(ParserError::MisplacedSymbol {
                        location: ParserErrorLocation::ReturnShape,
                        incorrect_symbol: chunk[0].to_owned(),
                        correct_symbol: Token::Word(String::from("return_value")),
                    })
                }
            },
            3 => {
                let value: String;
                let value_alias: String;
                match &chunk[0] {
                    Token::Word(w) => value = w.to_string(),
                    _ => {
                        return Err(ParserError::MisplacedSymbol {
                            location: ParserErrorLocation::ReturnShape,
                            incorrect_symbol: chunk[0].to_owned(),
                            correct_symbol: Token::Word(String::from("return_value")),
                        })
                    }
                }
                match &chunk[1] {
                    Token::Split(':') => {}
                    _ => {
                        return Err(ParserError::MisplacedSymbol {
                            location: ParserErrorLocation::ReturnShape,
                            incorrect_symbol: chunk[1].to_owned(),
                            correct_symbol: Token::Split(':'),
                        })
                    }
                }
                match &chunk[2] {
                    Token::StringLiteral(str_lit) => value_alias = str_lit.to_string(),
                    _ => {
                        return Err(ParserError::MisplacedSymbol {
                            location: ParserErrorLocation::ReturnShape,
                            incorrect_symbol: chunk[2].to_owned(),
                            correct_symbol: Token::StringLiteral(String::from(
                                "return_value_alias",
                            )),
                        })
                    }
                }

                result.insert(value, Some(value_alias));
            }
            0 => {}
            _ => {
                return Err(ParserError::BadLength {
                    location: ParserErrorLocation::ReturnShape,
                    incorrect_length: chunk.len(),
                    valid_lengths: vec![3, 1, 0],
                })
            }
        }
    }

    Ok(result)
}

pub fn parse_object(input: Vec<Token>) -> Result<Object, ParserError> {
    let name: String;
    let mut shape: ObjectShape = ObjectShape::new();
    let mut methods: Vec<String> = Vec::new();

    let mut cursor = input.into_iter().peekable();
    if cursor.peek().unwrap() != &Token::Word(String::from("Object")) {
        return Err(ParserError::InvalidSymbolBody {
            location: ParserErrorLocation::Object,
            incorrect_symbol: cursor.peek().unwrap().to_owned(),
            valid_symbols: vec!["Object".to_string()],
        });
    }
    cursor.next();

    match cursor.peek().unwrap() {
        Token::Word(w) => name = w.to_owned(),
        _ => {
            return Err(ParserError::MisplacedSymbol {
                location: ParserErrorLocation::Object,
                incorrect_symbol: cursor.peek().unwrap().to_owned(),
                correct_symbol: Token::Word(String::from("object_name")),
            })
        }
    }
    cursor.next();

    if cursor.peek().unwrap() != &Token::Encapsulator('{') {
        return Err(ParserError::PoorClosure {
            location: ParserErrorLocation::Object,
            incorrect_encap: cursor.peek().unwrap().to_owned(),
            correct_encap: Token::Encapsulator('{'),
        });
    }
    cursor.next();
    let mut internal: Vec<Token> = Vec::new();
    loop {
        let t = cursor.peek().unwrap();
        match t {
            Token::Split(';') => {
                internal.remove(internal.len() - 1);
                cursor.next();
                break;
            }
            _ => {
                internal.push(t.clone());
                cursor.next();
            }
        }
    }

    let mut internal_cursor = internal.into_iter().peekable();
    match internal_cursor.peek().unwrap() {
        Token::Word(_) => {}
        _ => {
            return Err(ParserError::MisplacedSymbol {
                location: ParserErrorLocation::Object,
                incorrect_symbol: internal_cursor.peek().unwrap().to_owned(),
                correct_symbol: Token::Word(String::from("function")),
            });
        }
    }
    internal_cursor.next();

    while let Some(Token::Word(w)) = internal_cursor.peek() {
        match w.as_str() {
            "shape" => {
                internal_cursor.next();
                if internal_cursor.peek().unwrap() != &Token::Encapsulator('(') {
                    return Err(ParserError::PoorClosure {
                        location: ParserErrorLocation::Object,
                        incorrect_encap: internal_cursor.peek().unwrap().to_owned(),
                        correct_encap: Token::Encapsulator('('),
                    });
                }
                internal_cursor.next();
                let mut shape_internal: Vec<Token> = Vec::new();
                loop {
                    let t = internal_cursor.peek().unwrap();
                    match t {
                        Token::Encapsulator(')') => {
                            internal_cursor.next();
                            break;
                        }
                        _ => {
                            shape_internal.push(t.clone());
                            internal_cursor.next();
                        }
                    }
                }
                shape = parse_object_shape(shape_internal)?;
            }
            "methods" => {
                internal_cursor.next();
                if internal_cursor.peek().unwrap() != &Token::Encapsulator('(') {
                    return Err(ParserError::PoorClosure {
                        location: ParserErrorLocation::Object,
                        incorrect_encap: internal_cursor.peek().unwrap().to_owned(),
                        correct_encap: Token::Encapsulator('('),
                    });
                }
                internal_cursor.next();
                let mut methods_internal: Vec<Token> = Vec::new();
                loop {
                    let t = internal_cursor.peek().unwrap();
                    match t {
                        Token::Encapsulator(')') => {
                            internal_cursor.next();
                            break;
                        }
                        _ => {
                            methods_internal.push(t.clone());
                            internal_cursor.next();
                        }
                    }
                }
                methods = parse_object_methods(methods_internal)?;
            }
            _ => {
                return Err(ParserError::InvalidSymbolBody {
                    location: ParserErrorLocation::Object,
                    incorrect_symbol: internal_cursor.peek().unwrap().to_owned(),
                    valid_symbols: vec!["shape".to_string(), "methods".to_string()],
                })
            }
        }
    }

    Ok(Object {
        name,
        shape,
        methods,
    })
}

fn parse_object_methods(input: Vec<Token>) -> Result<Vec<String>, ParserError> {
    let mut cursor = input.into_iter().peekable();
    let mut result: Vec<String> = Vec::new();

    if cursor.peek().unwrap() != &Token::Encapsulator('[') {
        return Err(ParserError::PoorClosure {
            location: ParserErrorLocation::ObjectMethods,
            incorrect_encap: cursor.peek().unwrap().to_owned(),
            correct_encap: Token::Encapsulator('['),
        });
    }
    cursor.next();
    let mut object_methods_internal: Vec<Token> = Vec::new();
    loop {
        let t = cursor.peek().unwrap();
        match t {
            Token::Encapsulator(']') => {
                cursor.next();
                break;
            }
            _ => {
                object_methods_internal.push(t.clone());
                cursor.next();
            }
        }
    }

    for chunk in object_methods_internal.chunks(2) {
        match chunk.len() {
            2 => {
                match &chunk[0] {
                    Token::Word(w) => result.push(w.to_string()),
                    _ => {
                        return Err(ParserError::MisplacedSymbol {
                            location: ParserErrorLocation::ObjectMethods,
                            incorrect_symbol: chunk[0].to_owned(),
                            correct_symbol: Token::Word(String::from("method_name")),
                        })
                    }
                }
                match &chunk[1] {
                    Token::Split(',') => {}
                    _ => {
                        return Err(ParserError::MisplacedSymbol {
                            location: ParserErrorLocation::ObjectMethods,
                            incorrect_symbol: chunk[1].to_owned(),
                            correct_symbol: Token::Split(','),
                        })
                    }
                }
            }
            1 => match &chunk[0] {
                Token::Word(w) => result.push(w.to_string()),
                _ => {
                    return Err(ParserError::MisplacedSymbol {
                        location: ParserErrorLocation::ObjectMethods,
                        incorrect_symbol: chunk[0].to_owned(),
                        correct_symbol: Token::Word(String::from("method_name")),
                    })
                }
            },
            _ => {
                return Err(ParserError::BadLength {
                    location: ParserErrorLocation::ObjectMethods,
                    incorrect_length: chunk.len(),
                    valid_lengths: vec![2, 1],
                })
            }
        }
    }

    Ok(result)
}

fn parse_object_shape(input: Vec<Token>) -> Result<ObjectShape, ParserError> {
    let mut cursor = input.into_iter().peekable();
    let mut result = ObjectShape::new();

    if cursor.peek().unwrap() != &Token::Encapsulator('{') {
        return Err(ParserError::PoorClosure {
            location: ParserErrorLocation::ObjectShape,
            incorrect_encap: cursor.peek().unwrap().to_owned(),
            correct_encap: Token::Encapsulator('{'),
        });
    }
    cursor.next();
    let mut object_shape_hashmap: Vec<Token> = Vec::new();
    loop {
        let t = cursor.peek().unwrap();
        match t {
            Token::Encapsulator('}') => {
                cursor.next();
                break;
            }
            _ => {
                object_shape_hashmap.push(t.clone());
                cursor.next();
            }
        }
    }

    for chunk in object_shape_hashmap.chunks(4) {
        let name: String;
        let val_type: Type;
        match chunk.len() {
            4 => {
                match &chunk[0] {
                    Token::Word(w) => name = w.to_string(),
                    _ => {
                        return Err(ParserError::MisplacedSymbol {
                            location: ParserErrorLocation::ObjectShape,
                            incorrect_symbol: chunk[0].to_owned(),
                            correct_symbol: Token::Word(String::from("object_shape_name")),
                        })
                    }
                }
                match &chunk[1] {
                    Token::Split(':') => {}
                    _ => {
                        return Err(ParserError::MisplacedSymbol {
                            location: ParserErrorLocation::ObjectShape,
                            incorrect_symbol: chunk[1].to_owned(),
                            correct_symbol: Token::Split(':'),
                        })
                    }
                }
                match &chunk[2] {
                    Token::Word(w) => val_type = parse_type(w.to_string())?,
                    _ => {
                        return Err(ParserError::MisplacedSymbol {
                            location: ParserErrorLocation::ObjectShape,
                            incorrect_symbol: chunk[2].to_owned(),
                            correct_symbol: Token::Word(String::from("object_shape_type")),
                        })
                    }
                }
                match &chunk[3] {
                    Token::Split(',') => {}
                    _ => {
                        return Err(ParserError::MisplacedSymbol {
                            location: ParserErrorLocation::ObjectShape,
                            incorrect_symbol: chunk[3].to_owned(),
                            correct_symbol: Token::Split(','),
                        })
                    }
                }
            }
            3 => {
                match &chunk[0] {
                    Token::Word(w) => name = w.to_string(),
                    _ => {
                        return Err(ParserError::MisplacedSymbol {
                            location: ParserErrorLocation::ObjectShape,
                            incorrect_symbol: chunk[0].to_owned(),
                            correct_symbol: Token::Word(String::from("object_shape_name")),
                        })
                    }
                }
                match &chunk[1] {
                    Token::Split(':') => {}
                    _ => {
                        return Err(ParserError::MisplacedSymbol {
                            location: ParserErrorLocation::ObjectShape,
                            incorrect_symbol: chunk[1].to_owned(),
                            correct_symbol: Token::Split(':'),
                        })
                    }
                }
                match &chunk[2] {
                    Token::Word(w) => val_type = parse_type(w.to_string())?,
                    _ => {
                        return Err(ParserError::MisplacedSymbol {
                            location: ParserErrorLocation::ObjectShape,
                            incorrect_symbol: chunk[2].to_owned(),
                            correct_symbol: Token::Word(String::from("object_shape_type")),
                        })
                    }
                }
            }
            _ => {
                return Err(ParserError::BadLength {
                    location: ParserErrorLocation::ObjectShape,
                    incorrect_length: chunk.len(),
                    valid_lengths: vec![4, 3],
                })
            }
        }
        result.insert(name, val_type);
    }

    Ok(result)
}

pub fn parse_global(input: Vec<Token>) -> Result<Global, ParserError> {
    let name: String;
    let mut head_route: String = String::new();
    let mut shape: ObjectShape = ObjectShape::new();
    let mut methods: Vec<String> = Vec::new();

    let mut cursor = input.into_iter().peekable();
    if cursor.peek().unwrap() != &Token::Word(String::from("Global")) {
        return Err(ParserError::InvalidSymbolBody {
            location: ParserErrorLocation::Global,
            incorrect_symbol: cursor.peek().unwrap().to_owned(),
            valid_symbols: vec!["Global".to_string()],
        });
    }
    cursor.next();

    match cursor.peek().unwrap() {
        Token::Word(w) => name = w.to_owned(),
        _ => {
            return Err(ParserError::MisplacedSymbol {
                location: ParserErrorLocation::Global,
                incorrect_symbol: cursor.peek().unwrap().to_owned(),
                correct_symbol: Token::Word(String::from("global_object_name")),
            })
        }
    }
    cursor.next();

    if cursor.peek().unwrap() != &Token::Encapsulator('{') {
        return Err(ParserError::PoorClosure {
            location: ParserErrorLocation::Global,
            incorrect_encap: cursor.peek().unwrap().to_owned(),
            correct_encap: Token::Encapsulator('{'),
        });
    }
    cursor.next();
    let mut internal: Vec<Token> = Vec::new();
    loop {
        let t = cursor.peek().unwrap();
        match t {
            Token::Split(';') => {
                internal.remove(internal.len() - 1);
                cursor.next();
                break;
            }
            _ => {
                internal.push(t.clone());
                cursor.next();
            }
        }
    }

    let mut internal_cursor = internal.into_iter().peekable();
    match internal_cursor.peek().unwrap() {
        Token::Word(_) => {}
        _ => {
            return Err(ParserError::MisplacedSymbol {
                location: ParserErrorLocation::Global,
                incorrect_symbol: internal_cursor.peek().unwrap().to_owned(),
                correct_symbol: Token::Word(String::from("function")),
            });
        }
    }

    while let Some(Token::Word(w)) = internal_cursor.peek() {
        match w.as_str() {
            "headRoute" => {
                internal_cursor.next();
                if internal_cursor.peek().unwrap() != &Token::Encapsulator('(') {
                    return Err(ParserError::PoorClosure {
                        location: ParserErrorLocation::Global,
                        incorrect_encap: internal_cursor.peek().unwrap().to_owned(),
                        correct_encap: Token::Encapsulator('('),
                    });
                }
                internal_cursor.next();
                let mut route_internal: Vec<Token> = Vec::new();
                loop {
                    let t = internal_cursor.peek().unwrap();
                    match t {
                        Token::Encapsulator(')') => {
                            internal_cursor.next();
                            break;
                        }
                        _ => {
                            route_internal.push(t.clone());
                            internal_cursor.next();
                        }
                    }
                }
                match &route_internal[0] {
                    Token::StringLiteral(str_lit) => head_route = str_lit.to_string(),
                    _ => {
                        return Err(ParserError::MisplacedSymbol {
                            location: ParserErrorLocation::Global,
                            incorrect_symbol: route_internal[0].to_owned(),
                            correct_symbol: Token::StringLiteral(String::from("head_route")),
                        })
                    }
                }
            }
            "shape" => {
                internal_cursor.next();
                if internal_cursor.peek().unwrap() != &Token::Encapsulator('(') {
                    return Err(ParserError::PoorClosure {
                        location: ParserErrorLocation::Global,
                        incorrect_encap: internal_cursor.peek().unwrap().to_owned(),
                        correct_encap: Token::Encapsulator('('),
                    });
                }
                internal_cursor.next();
                let mut shape_internal: Vec<Token> = Vec::new();
                loop {
                    let t = internal_cursor.peek().unwrap();
                    match t {
                        Token::Encapsulator(')') => {
                            internal_cursor.next();
                            break;
                        }
                        _ => {
                            shape_internal.push(t.clone());
                            internal_cursor.next();
                        }
                    }
                }
                shape = parse_object_shape(shape_internal)?;
            }
            "methods" => {
                internal_cursor.next();
                if internal_cursor.peek().unwrap() != &Token::Encapsulator('(') {
                    return Err(ParserError::PoorClosure {
                        location: ParserErrorLocation::Global,
                        incorrect_encap: internal_cursor.peek().unwrap().to_owned(),
                        correct_encap: Token::Encapsulator('('),
                    });
                }
                internal_cursor.next();
                let mut methods_internal: Vec<Token> = Vec::new();
                loop {
                    let t = internal_cursor.peek().unwrap();
                    match t {
                        Token::Encapsulator(')') => {
                            internal_cursor.next();
                            break;
                        }
                        _ => {
                            methods_internal.push(t.clone());
                            internal_cursor.next();
                        }
                    }
                }
                methods = parse_object_methods(methods_internal)?;
            }
            _ => {
                return Err(ParserError::InvalidSymbolBody {
                    location: ParserErrorLocation::Global,
                    incorrect_symbol: internal_cursor.peek().unwrap().to_owned(),
                    valid_symbols: vec![
                        "headRoute".to_string(),
                        "methods".to_string(),
                        "shape".to_string(),
                    ],
                })
            }
        }
    }

    Ok(Global {
        name,
        head_route,
        shape,
        methods,
    })
}
