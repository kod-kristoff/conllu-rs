use std::{
    fmt::{self, Display},
    io::{self},
};

pub trait Deserialize: Sized {
    type Builder: SentenceBuilder<Value = Self>;
    fn builder() -> Self::Builder;
}

pub trait SentenceBuilder {
    type Value;
    type Token;
    type Builder: TokenBuilder<Value = Self::Token>;

    fn add_meta(&mut self, key: &str, value: &str) -> Result<(), DeError>;
    fn add_comment(&mut self, comment: &str) -> Result<(), DeError> {
        let _ = comment;
        Ok(())
    }
    fn token_start(&self) -> Self::Builder;
    fn add_token(&mut self, token: Self::Token) -> Result<(), DeError>;

    fn finish(self) -> Option<Self::Value>;
}

pub trait TokenBuilder {
    type Value;

    fn add_field(&mut self, field: &str, value: &str) -> Result<(), DeError>;
    fn finish(self) -> Result<Self::Value, DeError>;
}

#[derive(Debug)]
pub enum DeError {
    InvalidValue(String, String),
    Io(io::Error),
    MissingField(String),
    UnexpectedEof(String),
    UnknownField(String),
}

impl DeError {
    pub fn missing_field(s: &str) -> Self {
        Self::MissingField(s.into())
    }
    pub fn invalid_value<S: Display>(field: &str, msg: S) -> Self {
        Self::InvalidValue(field.into(), msg.to_string())
    }
    pub fn unknown_field(field: &str, expected: &[&str]) -> Self {
        Self::UnknownField(format!(
            "unknown field `{}`, expected: {}",
            field,
            OneOf { names: expected }
        ))
    }
}
impl fmt::Display for DeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::InvalidValue(field, msg) => {
                write!(f, "Invalid value for field '{}': {}", field, msg)
            }
            Self::Io(_) => f.write_str("IO error"),
            Self::MissingField(field) => write!(f, "Missing field '{}'", field),
            Self::UnexpectedEof(msg) => f.write_fmt(format_args!("Unexpected Eof: {}", msg)),
            Self::UnknownField(msg) => f.write_str(msg),
        }
    }
}

impl std::error::Error for DeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(err) => Some(err),
            _ => None,
        }
    }
}

struct OneOf<'a> {
    names: &'a [&'a str],
}

impl<'a> fmt::Display for OneOf<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.names.len() {
            0 => f.write_str("has no fields"),
            1 => write!(f, "`{}`", self.names[0]),
            2 => write!(f, "`{}` or `{}`", self.names[0], self.names[1]),
            _ => {
                write!(f, "one of ")?;
                for (i, alt) in self.names.iter().enumerate() {
                    if i > 0 {
                        f.write_str(", ")?;
                    }
                    write!(f, "`{}`", alt)?;
                }
                Ok(())
            }
        }
    }
}
