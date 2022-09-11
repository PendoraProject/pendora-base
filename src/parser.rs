use std::{collections::HashMap, fs::read_to_string};

use walkdir::WalkDir;

use crate::{
    token::{tokenise, Token},
    types::*,
};

pub fn parse_project(root: &str) -> Result<Project, String> {
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

            let tokens = tokenise(input).unwrap();
            let tokens_cp = tokens.clone();

            match &tokens_cp[0] {
                Token::Word(w) => match w.as_str() {
                    "Global" => global = Some(parse_global(tokens).unwrap()),
                    "Method" => {
                        let method = parse_method(tokens).unwrap();
                        let method_name = &method.name;
                        methods.insert(method_name.to_string(), method);
                    }
                    "Object" => {
                        let object = parse_object(tokens).unwrap();
                        let object_name = &object.name;
                        objects.insert(object_name.to_string(), object);
                    }
                    _ => {
                        return Err(String::from(
                            "Invalid syntax (parse_project first token invalid word)",
                        ))
                    }
                },
                _ => {
                    return Err(String::from(
                        "Invalid syntax (parse_project first token not word)",
                    ))
                }
            }
        }
    }

    let true_global = global.unwrap();

    Ok(Project {
        global: true_global,
        objects,
        methods,
    })
}

pub fn parse_method(input: Vec<Token>) -> Result<Method, String> {
    let name: String;
    let arguments: MethodArguments;
    let route: String;
    let request_shape: RequestShape;
    let request_type: RequestType;
    let return_shape: ReturnShape;
    let return_object: String;

    let mut cursor = input.into_iter().peekable();
    if cursor.next().unwrap() != Token::Word(String::from("Method")) {
        return Err(String::from("Type mismatch"));
    }

    // destructure name
    match cursor.next().unwrap() {
        Token::Word(w) => name = w,
        _ => return Err(String::from("Invalid syntax")),
    }

    if cursor.next().unwrap() != Token::Encapsulator('(') {
        return Err(String::from("Invalid syntax"));
    }
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
    arguments = parse_method_arguments(arg_internal).unwrap();

    if cursor.next().unwrap() != Token::Encapsulator('{') {
        return Err(String::from("Invalid syntax"));
    }
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

    let internal = parse_method_internal(method_internal).unwrap();

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

fn parse_method_arguments(input: Vec<Token>) -> Result<MethodArguments, String> {
    let mut result = MethodArguments::new();
    for chunk in input.chunks(3) {
        let arg_name: String;
        let arg_type: Type;
        match chunk.len() {
            3 => {
                match &chunk[0] {
                    Token::Word(w) => arg_type = parse_type(w.to_string()).unwrap(),
                    _ => {
                        return Err(String::from(
                            "Invalid syntax (parse_method_arguments, chunk.len: 3)",
                        ))
                    }
                }
                match &chunk[1] {
                    Token::Word(w) => arg_name = w.to_string(),
                    _ => {
                        return Err(String::from(
                            "Invalid syntax (parse_method_arguments, chunk.len: 3)",
                        ))
                    }
                }
                match &chunk[2] {
                    Token::Split(',') => {}
                    _ => {
                        return Err(String::from(
                            "Invalid syntax (parse_method_arguments, chunk.len: 3)",
                        ))
                    }
                }
            }
            2 => {
                match &chunk[0] {
                    Token::Word(w) => arg_type = parse_type(w.to_string()).unwrap(),
                    _ => {
                        return Err(String::from(
                            "Invalid syntax (parse_method_arguments, chunk.len: 2)",
                        ))
                    }
                }
                match &chunk[1] {
                    Token::Word(w) => arg_name = w.to_string(),
                    _ => {
                        return Err(String::from(
                            "Invalid syntax (parse_method_arguments, chunk.len: 2)",
                        ))
                    }
                }
            }
            _ => return Err(String::from("Length mismatch")),
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

fn parse_method_internal(input: Vec<Token>) -> Result<MethodInternal, String> {
    let mut route: String = String::new();
    let mut request_shape: RequestShape = RequestShape::new();
    let mut request_type: RequestType = RequestType::GET;
    let mut return_shape: ReturnShape = ReturnShape::new();
    let mut return_object: String = String::new();

    let mut cursor = input.into_iter().peekable();

    match cursor.peek().unwrap() {
        Token::Word(_) => {}
        _ => {
            return Err(String::from("Type mismatch"));
        }
    }

    while let Some(Token::Word(w)) = cursor.next() {
        match w.as_str() {
            "route" => {
                if cursor.next().unwrap() != Token::Encapsulator('(') {
                    return Err(String::from(
                        "Invalid syntax (parse_method_internal route encap check)",
                    ));
                }
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
                        return Err(String::from(
                            "Invalid syntax (parse_method_internal route not a string literal)",
                        ))
                    }
                }
            }
            "request" => {
                if cursor.next().unwrap() != Token::Encapsulator('<') {
                    return Err(String::from(
                        "Invalid syntax (parse_method_internal request encap check)",
                    ));
                }
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
                    Token::Word(w) => request_type = parse_request_type(w.to_string()).unwrap(),
                    _ => {
                        return Err(String::from(
                            "Invalid syntax (parse_method_internal request not word)",
                        ))
                    }
                }

                if cursor.next().unwrap() != Token::Encapsulator('(') {
                    return Err(String::from(
                        "Invalid syntax (parse_method_internal request encap check)",
                    ));
                }
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
                request_shape = parse_request_shape(request_shape_internal).unwrap();
            }
            "return" => {
                if cursor.next().unwrap() != Token::Encapsulator('<') {
                    return Err(String::from(
                        "Invalid syntax (parse_method_internal return encap check)",
                    ));
                }
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
                        return Err(String::from(
                            "Invalid syntax (parse_method_internal return not word)",
                        ))
                    }
                }

                if cursor.next().unwrap() != Token::Encapsulator('(') {
                    return Err(String::from(
                        "Invalid syntax (parse_method_internal return encap check)",
                    ));
                }
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
                return_shape = parse_return_shape(return_shape_internal).unwrap();
            }
            _ => {
                return Err(String::from(
                    "Invalid syntax (parse_method_internal invalid function)",
                ))
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

fn parse_type(input: String) -> Result<Type, String> {
    match input.as_str() {
        "int" | "Integer" => Ok(Type::Integer),
        "bool" | "Boolean" => Ok(Type::Boolean),
        "str" | "String" => Ok(Type::String),
        "int?" | "Integer?" => Ok(Type::NullableInteger),
        "bool?" | "Boolean?" => Ok(Type::NullableBoolean),
        "str?" | "String?" => Ok(Type::NullableString),
        _ => Err(String::from("Invalid type (parse_type)")),
    }
}

fn parse_request_type(input: String) -> Result<RequestType, String> {
    match input.as_str() {
        "GET" => Ok(RequestType::GET),
        "POST" => Ok(RequestType::POST),
        "UPDATE" => Ok(RequestType::UPDATE),
        "DELETE" => Ok(RequestType::DELETE),
        _ => Err(String::from("Invalid type (parse_request_type)")),
    }
}

fn parse_request_shape(input: Vec<Token>) -> Result<RequestShape, String> {
    let mut cursor = input.into_iter().peekable();
    let mut result = RequestShape::new();

    if cursor.next().unwrap() != Token::Encapsulator('{') {
        return Err(String::from(
            "Invalid syntax (parse_request_shape encap check)",
        ));
    }
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
                        return Err(String::from(
                            "Invalid syntax (parse_request_shape 4 not word 1)",
                        ))
                    }
                }
                match &chunk[1] {
                    Token::Split(':') => {}
                    _ => {
                        return Err(String::from(
                            "Invalid syntax (parse_request_shape 4 not split ':')",
                        ))
                    }
                }
                match &chunk[2] {
                    Token::Word(w) => value = parse_method_shape_value(w.to_string()),
                    _ => {
                        return Err(String::from(
                            "Invalid syntax (parse_request_shape 4 not word 2)",
                        ))
                    }
                }
                match &chunk[3] {
                    Token::Split(',') => {}
                    _ => {
                        return Err(String::from(
                            "Invalid syntax (parse_request_shape 4 not split ',')",
                        ))
                    }
                }
            }
            3 => {
                match &chunk[0] {
                    Token::Word(w) => key_name = w.to_string(),
                    _ => {
                        return Err(String::from(
                            "Invalid syntax (parse_request_shape 3 not word 1)",
                        ))
                    }
                }
                match &chunk[1] {
                    Token::Split(':') => {}
                    _ => {
                        return Err(String::from(
                            "Invalid syntax (parse_request_shape 3 not split ':')",
                        ))
                    }
                }
                match &chunk[2] {
                    Token::Word(w) => value = parse_method_shape_value(w.to_string()),
                    _ => {
                        return Err(String::from(
                            "Invalid syntax (parse_request_shape 3 not word 2)",
                        ))
                    }
                }
            }
            _ => {
                return Err(String::from(
                    "Invalid syntax (parse_request_shape chunk length)",
                ))
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

fn parse_return_shape(input: Vec<Token>) -> Result<ReturnShape, String> {
    let mut cursor = input.into_iter().peekable();
    let mut result = ReturnShape::new();

    if cursor.next().unwrap() != Token::Encapsulator('{') {
        return Err(String::from(
            "Invalid syntax (parse_return_shape encap check)",
        ));
    }
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
                    return Err(String::from(
                        "Invalid syntax (parse_return_shape 1 not word)",
                    ))
                }
            },
            3 => {
                let value: String;
                let value_alias: String;
                match &chunk[0] {
                    Token::Word(w) => value = w.to_string(),
                    _ => {
                        return Err(String::from(
                            "Invalid syntax (parse_return_shape 3 not word 1)",
                        ))
                    }
                }
                match &chunk[1] {
                    Token::Split(':') => {}
                    _ => {
                        return Err(String::from(
                            "Invalid syntax (parse_return_shape 3 not split ':')",
                        ))
                    }
                }
                match &chunk[2] {
                    Token::StringLiteral(str_lit) => value_alias = str_lit.to_string(),
                    _ => {
                        return Err(String::from(
                            "Invalid syntax (parse_return_shape 3 not word 2)",
                        ))
                    }
                }

                result.insert(value, Some(value_alias));
            }
            0 => {}
            _ => {
                return Err(String::from(
                    "Invalid syntax (parse_return_shape bad chunk length)",
                ))
            }
        }
    }

    Ok(result)
}

pub fn parse_object(input: Vec<Token>) -> Result<Object, String> {
    let name: String;
    let mut shape: ObjectShape = ObjectShape::new();
    let mut methods: Vec<String> = Vec::new();

    let mut cursor = input.into_iter().peekable();
    if cursor.next().unwrap() != Token::Word(String::from("Object")) {
        return Err(String::from("Type mismatch"));
    }

    match cursor.next().unwrap() {
        Token::Word(w) => name = w,
        _ => return Err(String::from("Invalid syntax")),
    }

    if cursor.next().unwrap() != Token::Encapsulator('{') {
        return Err(String::from("Invalid syntax"));
    }
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
            return Err(String::from("Type mismatch"));
        }
    }

    while let Some(Token::Word(w)) = internal_cursor.next() {
        match w.as_str() {
            "shape" => {
                if internal_cursor.next().unwrap() != Token::Encapsulator('(') {
                    return Err(String::from(
                        "Invalid syntax (parse_object shape encap check)",
                    ));
                }
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
                shape = parse_object_shape(shape_internal).unwrap();
            }
            "methods" => {
                if internal_cursor.next().unwrap() != Token::Encapsulator('(') {
                    return Err(String::from(
                        "Invalid syntax (parse_object methods encap check)",
                    ));
                }
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
                methods = parse_object_methods(methods_internal).unwrap();
            }
            _ => {
                return Err(String::from(
                    "Invalid syntax (parse_object invalid function)",
                ))
            }
        }
    }

    Ok(Object {
        name,
        shape,
        methods,
    })
}

fn parse_object_methods(input: Vec<Token>) -> Result<Vec<String>, String> {
    let mut cursor = input.into_iter().peekable();
    let mut result: Vec<String> = Vec::new();

    if cursor.next().unwrap() != Token::Encapsulator('[') {
        return Err(String::from(
            "Invalid syntax (parse_object_methods encap check)",
        ));
    }
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
                        return Err(String::from(
                            "Invalid syntax (parse_object_methods 2 not word)",
                        ))
                    }
                }
                match &chunk[1] {
                    Token::Split(',') => {}
                    _ => {
                        return Err(String::from(
                            "Invalid syntax (parse_object_methods 2 not split ',')",
                        ))
                    }
                }
            }
            1 => match &chunk[0] {
                Token::Word(w) => result.push(w.to_string()),
                _ => {
                    return Err(String::from(
                        "Invalid syntax (parse_object_methods 1 not word)",
                    ))
                }
            },
            _ => {
                return Err(String::from(
                    "Invalid syntax (parse_object_methods bad length)",
                ))
            }
        }
    }

    Ok(result)
}

fn parse_object_shape(input: Vec<Token>) -> Result<ObjectShape, String> {
    let mut cursor = input.into_iter().peekable();
    let mut result = ObjectShape::new();

    if cursor.next().unwrap() != Token::Encapsulator('{') {
        return Err(String::from(
            "Invalid syntax (parse_object_shape encap check)",
        ));
    }
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
                        return Err(String::from(
                            "Invalid syntax (parse_object_shape 4 not word 1)",
                        ))
                    }
                }
                match &chunk[1] {
                    Token::Split(':') => {}
                    _ => {
                        return Err(String::from(
                            "Invalid syntax (parse_object_shape 4 not split ':')",
                        ))
                    }
                }
                match &chunk[2] {
                    Token::Word(w) => val_type = parse_type(w.to_string()).unwrap(),
                    _ => {
                        return Err(String::from(
                            "Invalid syntax (parse_object_shape 4 not word 2)",
                        ))
                    }
                }
                match &chunk[3] {
                    Token::Split(',') => {}
                    _ => {
                        return Err(String::from(
                            "Invalid syntax (parse_object_shape 4 not split ',')",
                        ))
                    }
                }
            }
            3 => {
                match &chunk[0] {
                    Token::Word(w) => name = w.to_string(),
                    _ => {
                        return Err(String::from(
                            "Invalid syntax (parse_object_shape 3 not word 1)",
                        ))
                    }
                }
                match &chunk[1] {
                    Token::Split(':') => {}
                    _ => {
                        return Err(String::from(
                            "Invalid syntax (parse_object_shape 3 not split ':')",
                        ))
                    }
                }
                match &chunk[2] {
                    Token::Word(w) => val_type = parse_type(w.to_string()).unwrap(),
                    _ => {
                        return Err(String::from(
                            "Invalid syntax (parse_object_shape 3 not word 2)",
                        ))
                    }
                }
            }
            _ => {
                return Err(String::from(
                    "Invalid syntax (parse_object_shape chunk length)",
                ))
            }
        }
        result.insert(name, val_type);
    }

    Ok(result)
}

pub fn parse_global(input: Vec<Token>) -> Result<Global, String> {
    let name: String;
    let mut head_route: String = String::new();
    let mut shape: ObjectShape = ObjectShape::new();
    let mut methods: Vec<String> = Vec::new();

    let mut cursor = input.into_iter().peekable();
    if cursor.next().unwrap() != Token::Word(String::from("Global")) {
        return Err(String::from("Type mismatch"));
    }

    match cursor.next().unwrap() {
        Token::Word(w) => name = w,
        _ => return Err(String::from("Invalid syntax 1 ")),
    }

    if cursor.next().unwrap() != Token::Encapsulator('{') {
        return Err(String::from("Invalid syntax 2"));
    }
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
            return Err(String::from("Type mismatch"));
        }
    }

    while let Some(Token::Word(w)) = internal_cursor.next() {
        match w.as_str() {
            "headRoute" => {
                if internal_cursor.next().unwrap() != Token::Encapsulator('(') {
                    return Err(String::from(
                        "Invalid syntax (parse_method_internal route encap check)",
                    ));
                }
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
                        return Err(String::from(
                            "Invalid syntax (parse_global head_route not a string literal)",
                        ))
                    }
                }
            }
            "shape" => {
                if internal_cursor.next().unwrap() != Token::Encapsulator('(') {
                    return Err(String::from(
                        "Invalid syntax (parse_object shape encap check)",
                    ));
                }
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
                shape = parse_object_shape(shape_internal).unwrap();
            }
            "methods" => {
                if internal_cursor.next().unwrap() != Token::Encapsulator('(') {
                    return Err(String::from(
                        "Invalid syntax (parse_object methods encap check)",
                    ));
                }
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
                methods = parse_object_methods(methods_internal).unwrap();
            }
            _ => {
                return Err(String::from(
                    "Invalid syntax (parse_object invalid function)",
                ))
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
