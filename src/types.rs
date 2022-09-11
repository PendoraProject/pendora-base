use std::collections::HashMap;

#[derive(Debug)]
pub struct Project {
    pub(crate) global: Global,
    pub(crate) objects: HashMap<String, Object>,
    pub(crate) methods: HashMap<String, Method>,
}

#[derive(Debug)]
pub struct Global {
    pub(crate) name: String,
    pub(crate) head_route: String,
    pub(crate) shape: ObjectShape,
    pub(crate) methods: Vec<String>,
}

#[derive(Debug)]
pub struct Object {
    pub(crate) name: String,
    pub(crate) shape: ObjectShape,
    pub(crate) methods: Vec<String>,
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
    pub(crate) name: String,
    pub(crate) arguments: MethodArguments,
    pub(crate) route: String,
    pub(crate) request_shape: RequestShape,
    pub(crate) request_type: RequestType,
    pub(crate) return_shape: ReturnShape,
    pub(crate) return_object: String,
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
    UPDATE,
    DELETE,
}
// Option<String> to support parsing aliases
pub type ReturnShape = HashMap<String, Option<String>>;
