use std::fmt;
use std::io::{self, Lines};
use std::marker::PhantomData;

use crate::de::{self, SentenceBuilder, TokenBuilder};
pub use crate::parse::parser::{
    parse_dict_value, parse_from_str_nullable, parse_nullable_value, parse_paired_list_value,
};
use crate::tree;

pub(crate) mod parser;

const DEFAULT_FIELDS: &[&str] = &[
    "id", "form", "lemma", "upos", "xpos", "feats", "head", "deprel", "deps", "misc",
];

pub fn parse<S, R>(in_file: R) -> Result<Vec<S>, de::DeError>
where
    S: de::Deserialize,
    R: io::BufRead,
{
    let mut sentences = Vec::new();
    for sentence in parse_incr(in_file) {
        sentences.push(sentence?);
    }
    Ok(sentences)
}

pub fn parse_incr<S, R>(in_file: R) -> impl Iterator<Item = Result<S, de::DeError>>
where
    S: de::Deserialize,
    R: io::BufRead,
{
    SentenceGenerator::new(in_file)
}
pub fn parse_tree<S, R>(in_file: R) -> Result<Vec<tree::TokenTree>, ParseTreeError>
where
    S: de::Deserialize + tree::ToTree,
    R: io::BufRead,
{
    let mut token_trees = vec![];
    for token_tree in parse_tree_incr::<S, R>(in_file) {
        token_trees.push(token_tree?);
    }
    Ok(token_trees)
}

pub fn parse_tree_incr<S, R>(
    in_file: R,
) -> impl Iterator<Item = Result<tree::TokenTree, ParseTreeError>>
where
    S: de::Deserialize + tree::ToTree,
    R: io::BufRead,
{
    TokenTreeGenerator {
        sent_iter: SentenceGenerator::<S, R>::new(in_file),
    }
}

#[derive(Debug)]
pub enum ParseTreeError {
    TreeError(tree::ToTreeError),
    DeError(de::DeError),
}

impl fmt::Display for ParseTreeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::DeError(_) => f.write_str("Deserialize error"),
            Self::TreeError(_) => f.write_str("Failed to convert to tree"),
        }
    }
}

impl std::error::Error for ParseTreeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::DeError(err) => Some(err),
            Self::TreeError(err) => Some(err),
        }
    }
}

impl From<de::DeError> for ParseTreeError {
    fn from(value: de::DeError) -> Self {
        ParseTreeError::DeError(value)
    }
}

impl From<tree::ToTreeError> for ParseTreeError {
    fn from(value: tree::ToTreeError) -> Self {
        ParseTreeError::TreeError(value)
    }
}

struct SentenceGenerator<S, R: io::BufRead> {
    lines: Lines<R>,
    // global_columns: Vec<String>,
    _phantom: PhantomData<S>,
}

impl<S, R> SentenceGenerator<S, R>
where
    R: io::BufRead,
{
    pub fn new(in_file: R) -> Self {
        Self {
            lines: in_file.lines(),
            _phantom: PhantomData,
        }
    }
}
impl<S, R: io::BufRead> Iterator for SentenceGenerator<S, R>
where
    S: de::Deserialize,
{
    type Item = Result<S, de::DeError>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut line = match self.lines.next()? {
            Err(err) => return Some(Err(de::DeError::Io(err))),
            Ok(x) => x,
        };

        if line.trim().is_empty() {
            line = match self.lines.next()? {
                Err(err) => return Some(Err(de::DeError::Io(err))),
                Ok(x) => x,
            };
        }
        dbg!(&line);
        let mut builder = S::builder();
        while let Some(meta_or_comm) = line.strip_prefix('#') {
            if let Some((key, value)) = meta_or_comm.split_once('=') {
                if let Err(err) = builder.add_meta(key.trim(), value.trim()) {
                    return Some(Err(err));
                }
            } else if let Err(err) = builder.add_comment(meta_or_comm.trim()) {
                return Some(Err(err));
            }
            line = match self.lines.next() {
                None => {
                    return Some(Err(de::DeError::UnexpectedEof(
                        "Sentence without tokens".to_string(),
                    )));
                }
                Some(Err(err)) => return Some(Err(de::DeError::Io(err))),
                Some(Ok(x)) => x,
            };
        }
        while !line.trim().is_empty() {
            let mut token = builder.token_start();
            for (key, value) in DEFAULT_FIELDS.iter().zip(line.split('\t')) {
                if let Err(err) = token.add_field(key, value) {
                    return Some(Err(err));
                }
            }
            let token = match token.finish() {
                Ok(x) => x,
                Err(err) => return Some(Err(err)),
            };
            if let Err(err) = builder.add_token(token) {
                return Some(Err(err));
            }

            line = match self.lines.next() {
                None => break,
                Some(Err(err)) => return Some(Err(de::DeError::Io(err))),
                Some(Ok(x)) => x,
            };
        }
        if let Some(sentence) = builder.finish() {
            return Some(Ok(sentence));
        }

        None
    }
}

struct TokenTreeGenerator<S, R: io::BufRead> {
    sent_iter: SentenceGenerator<S, R>,
}

impl<S, R> Iterator for TokenTreeGenerator<S, R>
where
    S: de::Deserialize + tree::ToTree,
    R: io::BufRead,
{
    type Item = Result<tree::TokenTree, ParseTreeError>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(sentence) = self.sent_iter.next() {
            match sentence {
                Ok(sentence) => match sentence.to_tree() {
                    Ok(tree) => Some(Ok(tree)),
                    Err(err) => Some(Err(err.into())),
                },
                Err(err) => Some(Err(err.into())),
            }
        } else {
            None
        }
    }
}
