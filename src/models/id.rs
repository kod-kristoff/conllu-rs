use std::{fmt, str::FromStr, sync::LazyLock};

use regex::Regex;

#[derive(Clone, Copy, PartialEq)]
pub enum Id {
    Single(i16),
    Range(i16, i16),
    Dot(i16, i16),
}

impl Id {
    fn do_fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Single(id) => f.write_fmt(format_args!("{}", id)),
            Self::Range(from, to) => f.write_fmt(format_args!("{}-{}", from, to)),
            Self::Dot(major, minor) => f.write_fmt(format_args!("{}.{}", major, minor)),
        }
    }
}

impl fmt::Debug for Id {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.do_fmt(f)
    }
}
impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.do_fmt(f)
    }
}

const ID_SINGLE_PATTERN: &str = r"(?:0|[1-9][0-9]*)";
const ID_RANGE_PATTERN: &str = r"[1-9][0-9]*\-[1-9][0-9]*";
const ID_DOT_ID_PATTERN: &str = r"[0-9][0-9]*\.[1-9][0-9]*";
static ID_SINGLE: LazyLock<Regex> = LazyLock::new(|| Regex::new(ID_SINGLE_PATTERN).unwrap());
static ID_RANGE: LazyLock<Regex> = LazyLock::new(|| Regex::new(ID_RANGE_PATTERN).unwrap());
static ID_DOT_ID: LazyLock<Regex> = LazyLock::new(|| Regex::new(ID_DOT_ID_PATTERN).unwrap());

#[derive(Debug)]
pub struct ParseIdError(String);

impl fmt::Display for ParseIdError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Failed to parse as id: {}", self.0)
    }
}

impl std::error::Error for ParseIdError {}

impl FromStr for Id {
    type Err = ParseIdError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        // if value.is_empty() || value == "_" {
        //     return None;
        // }

        if ID_RANGE.is_match(value) {
            let mut ids = value.split('-');
            let from = ids.next().unwrap().parse().unwrap();
            let to = ids.next().unwrap().parse().unwrap();
            Ok(Id::Range(from, to))
        } else if ID_DOT_ID.is_match(value) {
            let mut ids = value.split('.');
            let major = ids.next().unwrap().parse().unwrap();
            let minor = ids.next().unwrap().parse().unwrap();
            Ok(Id::Dot(major, minor))
        } else if ID_SINGLE.is_match(value) {
            match value.parse() {
                Ok(id) => Ok(Id::Single(id)),
                Err(err) => todo!("handle parse error: {}", err),
            }
        } else {
            Err(ParseIdError(format!("Unsupported id form: '{}'", value)))
        }
    }
}
