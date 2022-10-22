use crate::token::Token;
use std::fmt;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("Wrong symbol found while parsing {location}. Expected {correct_symbol:?} but found {incorrect_symbol:?}.")]
    MisplacedSymbol {
        location: ParserErrorLocation,
        incorrect_symbol: Token,
        correct_symbol: Token,
    },
    #[error("Invalid symbol content found while parsing {location}. Found {incorrect_symbol:?} but expected {valid_symbols:?}.")]
    InvalidSymbolBody {
        location: ParserErrorLocation,
        incorrect_symbol: Token,
        valid_symbols: Vec<String>,
    },
    #[error("Incorrectly sized chunk found while parsing {location}. Expected length(s) {valid_lengths:?} but found length {incorrect_length}.")]
    BadLength {
        location: ParserErrorLocation,
        incorrect_length: usize,
        valid_lengths: Vec<usize>,
    },
    #[error("Incomplete or poor closure found while parsing {location}. Expected {correct_encap:?} but found {incorrect_encap:?}")]
    PoorClosure {
        location: ParserErrorLocation,
        incorrect_encap: Token,
        correct_encap: Token,
    },
    #[error("Required field ({missing_field}) unable to be found while parsing {location}.")]
    FieldNotExistent {
        location: ParserErrorLocation,
        missing_field: String,
    },
}

#[derive(Debug)]
pub enum ParserErrorLocation {
    Project { file_name: String },
    Global,
    Object,
    Method,
    MethodInternal,
    MethodArguments,
    Type,
    RequestType,
    RequestShape,
    MethodShapeValue,
    ReturnShape,
    ObjectMethods,
    ObjectShape,
}

impl fmt::Display for ParserErrorLocation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let expanded_loc = match self.to_owned() {
            ParserErrorLocation::Project { file_name } => {
                format!("a project ({file_name})")
            }
            ParserErrorLocation::Global => "a global object".to_string(),
            ParserErrorLocation::Object => "an object".to_string(),
            ParserErrorLocation::Method => "a method".to_string(),
            ParserErrorLocation::MethodInternal => "a method's internal".to_string(),
            ParserErrorLocation::MethodArguments => "a method's arguments".to_string(),
            ParserErrorLocation::Type => "a Pendora type".to_string(),
            ParserErrorLocation::RequestType => "a HTTP request type".to_string(),
            ParserErrorLocation::RequestShape => "a method's request shape".to_string(),
            ParserErrorLocation::MethodShapeValue => "a method shape value".to_string(),
            ParserErrorLocation::ReturnShape => "a method's shape".to_string(),
            ParserErrorLocation::ObjectMethods => "an object's methods list".to_string(),
            ParserErrorLocation::ObjectShape => "an object's shape".to_string(),
        };
        write!(f, "{}", expanded_loc)
    }
}
