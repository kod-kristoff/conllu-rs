use std::{fmt, str::FromStr};

use crate::models::{Dict, Id, ParseIdError};

pub fn parse_nullable_value(value: &str) -> Option<&str> {
    if value.is_empty() || value == "_" {
        None
    } else {
        Some(value)
    }
}

pub fn parse_dict_value(value: &str) -> Option<Dict<String, String>> {
    parse_nullable_value(value)?;
    let mut dict = Dict::new();

    for part in value.split('|') {
        let mut key_value = part.split('=');
        let Some(key) = key_value.next() else {
            continue;
        };
        let value = key_value
            .next()
            .and_then(parse_nullable_value)
            .unwrap_or("")
            .to_string();
        dict.insert(key.to_string(), value);
    }
    if dict.is_empty() { None } else { Some(dict) }
}

pub fn parse_from_str_nullable<F: FromStr>(value: &str) -> Result<Option<F>, F::Err> {
    if value == "_" {
        return Ok(None);
    }

    match value.parse() {
        Ok(v) => Ok(Some(v)),
        Err(err) => Err(err),
    }
}

#[derive(Debug)]
pub enum ParsePairedListError {
    MissingColon(String),
    ParseId(ParseIdError),
}

impl From<ParseIdError> for ParsePairedListError {
    fn from(value: ParseIdError) -> Self {
        Self::ParseId(value)
    }
}

impl fmt::Display for ParsePairedListError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingColon(val) => write!(f, "Missing `:` in '{}'", val),
            Self::ParseId(_err) => f.write_str("Failed to parse id"),
        }
    }
}

impl std::error::Error for ParsePairedListError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ParseId(err) => Some(err),
            _ => None,
        }
    }
}

pub fn parse_paired_list_value(
    value: &str,
) -> Result<Option<Vec<(String, Id)>>, ParsePairedListError> {
    if value == "_" {
        return Ok(None);
    }
    let mut list = Vec::new();
    for part in value.split('|') {
        let Some((id, dep)) = part.split_once(':') else {
            return Err(ParsePairedListError::MissingColon(value.into()));
        };

        list.push((dep.to_string(), id.parse()?));
    }
    Ok(if list.is_empty() { None } else { Some(list) })
}

#[cfg(test)]
mod tests {
    use super::*;
    mod parse_paired_list_value {
        use std::error::Error;

        use rstest::rstest;

        use super::*;
        #[rstest]
        #[case::no_colon("no_colon")]
        #[case::bad_id("k:nsubj")]
        fn malformed_value_returns_error(#[case] v: &str) {
            let actual = parse_paired_list_value(v).unwrap_err();
            insta::assert_snapshot!(
                format!("malformed_value_returns_error-{}", v.replace(':', "_")),
                actual
            );
            insta::assert_snapshot!(
                format!(
                    "malformed_value_returns_error-{}-source",
                    v.replace(':', "_")
                ),
                actual.source().map(ToString::to_string).unwrap_or_default()
            );
        }
    }
}
