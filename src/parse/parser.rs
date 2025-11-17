use std::{
    io::{self, Lines},
    mem,
    sync::LazyLock,
};

use regex::Regex;

use crate::models::{Dict, Id, Sentence, Token};

const DEFAULT_FIELDS: &[&str] = &[
    "id", "form", "lemma", "upos", "xpos", "feats", "head", "deprel", "deps", "misc",
];
// static DEFAULT_METADATA_PARSERS: LazyLock<Dict<&str, Fn(&str, &str) -> (String, String)>> =
//     LazyLock::new(|| {
//         Dict::from_iter(&[
//             ("newpar", |key, value| (key.to_string(), value.to_string())),
//             ("newdoc", |key, value| (key.to_string(), value.to_string())),
//         ])
//     });

pub fn parse_sentences<R: io::BufRead>(in_file: R) -> SentenceIter<R> {
    SentenceIter {
        lines: in_file.lines(),
        buf: Vec::new(),
    }
}

pub(super) fn parse_token_and_metadata(
    metadata_lines: Vec<String>,
    token_lines: Vec<String>,
) -> Sentence {
    let mut tokens = vec![];
    let mut metadata = Dict::new();

    for line in metadata_lines {
        for (key, value) in parse_comment_line(line) {
            if let Some(value) = value {
                metadata.insert(key, value);
            }
        }
    }
    for line in token_lines {
        tokens.push(parse_line(line));
    }
    Sentence::new(tokens, metadata)
}

fn parse_line(line: String) -> Token {
    static TAB_OR_2SPACES: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\t| {2,}").unwrap());
    let mut line_split = TAB_OR_2SPACES.split(&line);
    // if line_split.len() == 1 {
    //     todo!("Error: Invalid line format");
    // }

    let Some(id) = line_split.next().and_then(parse_id_value) else {
        todo!("Empty line");
    };
    let Some(form) = line_split.next().map(ToString::to_string) else {
        todo!("Invalid line format");
    };
    let Some(lemma) = line_split
        .next()
        .map(|s| parse_nullable_value(s).map(ToString::to_string))
    else {
        todo!("Invalid line format");
    };
    let upos = line_split
        .next()
        .and_then(parse_nullable_value)
        .map(ToString::to_string);
    // else {
    //     todo!("Invalid line format");
    // };
    let xpos = line_split
        .next()
        .and_then(parse_nullable_value)
        .map(ToString::to_string);
    let feats = line_split.next().and_then(parse_dict_value);
    let head = line_split.next().and_then(parse_int_value);
    let deprel = line_split
        .next()
        .and_then(parse_nullable_value)
        .map(ToString::to_string);
    let deps = line_split.next().and_then(parse_paired_list_value);
    let misc = line_split.next().and_then(parse_dict_value);

    Token::new(id, form, lemma, upos, xpos, feats, head, deprel, deps, misc)
}
fn parse_comment_line(line: String) -> Vec<(String, Option<String>)> {
    let (key, value) = parse_pair_value(line);
    if key.is_empty() || value.is_none() {
        vec![]
    } else {
        vec![(key, value)]
    }
}

fn parse_pair_value(line: String) -> (String, Option<String>) {
    let mut key_maybe_value = line.splitn(2, '=');
    let key = key_maybe_value.next().unwrap().trim().to_string();
    let value = key_maybe_value.next().map(|s| s.trim().to_string());
    (key, value)
}
pub(super) struct SentenceIter<R: io::BufRead> {
    lines: Lines<R>,
    buf: Vec<String>,
}

impl<R: io::BufRead> Iterator for SentenceIter<R> {
    type Item = Result<Vec<String>, io::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(line) = self.lines.next() {
                let line = match line {
                    Ok(x) => x,
                    Err(err) => return Some(Err(err)),
                };
                if line.trim().is_empty() {
                    if self.buf.is_empty() {
                        continue;
                    }
                    return Some(Ok(mem::take(&mut self.buf)));
                } else {
                    self.buf.push(line);
                }
            } else {
                break;
            }
        }
        if !self.buf.is_empty() {
            return Some(Ok(mem::take(&mut self.buf)));
        }
        None
    }
}
const ID_SINGLE_PATTERN: &str = r"(?:0|[1-9][0-9]*)";
const ID_RANGE_PATTERN: &str = r"[1-9][0-9]*\-[1-9][0-9]*";
const ID_DOT_ID_PATTERN: &str = r"[0-9][0-9]*\.[1-9][0-9]*";
static ID_SINGLE: LazyLock<Regex> = LazyLock::new(|| Regex::new(ID_SINGLE_PATTERN).unwrap());
static ID_RANGE: LazyLock<Regex> = LazyLock::new(|| Regex::new(ID_RANGE_PATTERN).unwrap());
static ID_DOT_ID: LazyLock<Regex> = LazyLock::new(|| Regex::new(ID_DOT_ID_PATTERN).unwrap());

fn parse_id_value(value: &str) -> Option<Id> {
    // if value.is_empty() || value == "_" {
    //     return None;
    // }

    if ID_RANGE.is_match(value) {
        let mut ids = value.split('-');
        let from = ids.next().unwrap().parse().unwrap();
        let to = ids.next().unwrap().parse().unwrap();
        Some(Id::Range(from, to))
    } else if ID_DOT_ID.is_match(value) {
        let mut ids = value.split('.');
        let major = ids.next().unwrap().parse().unwrap();
        let minor = ids.next().unwrap().parse().unwrap();
        Some(Id::Dot(major, minor))
    } else if ID_SINGLE.is_match(value) {
        match value.parse() {
            Ok(id) => Some(Id::Single(id)),
            Err(err) => todo!("handle parse error: {}", err),
        }
    } else {
        None
    }
}

fn parse_nullable_value(value: &str) -> Option<&str> {
    if value.is_empty() || value == "_" {
        None
    } else {
        Some(value)
    }
}

fn parse_dict_value(value: &str) -> Option<Dict<String, String>> {
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

fn parse_int_value(value: &str) -> Option<u8> {
    if value == "_" {
        return None;
    }

    match value.parse() {
        Ok(num) => Some(num),
        Err(err) => todo!("handle err {}", err),
    }
}

fn parse_paired_list_value(value: &str) -> Option<Vec<(String, Id)>> {
    if value == "_" {
        return None;
    }
    let mut list = Vec::new();
    for part in value.split('|') {
        let id_dep = part.split_once(':');
        let Some(id) = id_dep.map(|x| x.0) else {
            todo!("handle empty")
            // return Some(vec![(value.to_string(), None)]);
        };
        let Some(dep) = id_dep.map(|x| x.1) else {
            todo!("handle no : in '{value}'")
            // return Some(vec![(value.to_string(), None)]);
        };
        let Some(id) = parse_id_value(id) else {
            todo!("handle bad id in '{value}'")
        };
        list.push((dep.to_string(), id));
    }
    if list.is_empty() { None } else { Some(list) }
}
