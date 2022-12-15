use std::collections::HashMap;

#[derive(Debug)]
pub struct Project {
    pub global: Global,
    pub objects: HashMap<String, Object>,
    pub methods: HashMap<String, Method>,
}

#[derive(Debug)]
pub struct Global {
    pub name: String,
    pub head_route: String,
    pub shape: ObjectShape,
    pub methods: Vec<String>,
}

#[derive(Debug)]
pub struct Object {
    pub name: String,
    pub shape: ObjectShape,
    pub methods: Vec<String>,
}

#[derive(Debug)]
pub enum Type {
    Integer,
    String,
    Boolean,
    NullableInteger,
    NullableString,
    NullableBoolean,
}

pub type ObjectShape = HashMap<String, Type>;

#[derive(Debug)]
pub struct Method {
    pub name: String,
    pub arguments: MethodArguments,
    pub route: String,
    pub request_shape: RequestShape,
    pub request_type: RequestType,
    pub return_shape: ReturnShape,
    pub return_object: String,
}

pub type MethodArguments = HashMap<String, Type>;
pub type RequestShape = HashMap<String, Value>;

#[derive(Debug)]
pub enum Value {
    Global(String),
    Parent(String),
    Argument(String),
}

#[derive(Debug)]
pub enum RequestType {
    GET,
    POST,
    PATCH,
    DELETE,
}
// Option<String> to support parsing aliases
pub type ReturnShape = HashMap<String, Option<String>>;
