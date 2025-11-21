use std::io;

use crate::{
    models::{Sentence, TokenTree},
    parse::parser::{SentenceIter, parse_sentences, parse_token_and_metadata},
};

mod parser;

pub fn parse<R: io::BufRead>(in_file: R) -> Result<Vec<Sentence>, io::Error> {
    let mut sentences = Vec::new();
    for sentence in parse_incr(in_file) {
        sentences.push(sentence?);
    }
    Ok(sentences)
}

pub fn parse_incr<R: io::BufRead>(in_file: R) -> impl Iterator<Item = Result<Sentence, io::Error>> {
    SentenceGenerator {
        sentences: parse_sentences(in_file),
        global_columns: Vec::new(),
    }
}

pub fn parse_tree<R: io::BufRead>(in_file: R) -> Result<Vec<TokenTree>, io::Error> {
    let mut token_trees = vec![];
    for token_tree in parse_tree_incr(in_file) {
        token_trees.push(token_tree?);
    }
    Ok(token_trees)
}

pub fn parse_tree_incr<R: io::BufRead>(
    in_file: R,
) -> impl Iterator<Item = Result<TokenTree, io::Error>> {
    TokenTreeGenerator {
        sent_iter: SentenceGenerator {
            sentences: parse_sentences(in_file),
            global_columns: vec![],
        },
    }
}

struct SentenceGenerator<R: io::BufRead> {
    sentences: SentenceIter<R>,
    global_columns: Vec<String>,
}

impl<R: io::BufRead> Iterator for SentenceGenerator<R> {
    type Item = Result<Sentence, io::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(sentence) = self.sentences.next() {
            let sentence = match sentence {
                Ok(x) => x,
                Err(err) => return Some(Err(err)),
            };
            let mut curr_metadata = Vec::new();
            let mut curr_sentence = Vec::new();
            for line in sentence {
                //     let line = match line {
                //         Ok(x) => x,
                //         Err(err) => return Some(Err(err)),
                //     };
                if line.starts_with('#') {
                    if line.starts_with("# global.columns = ") {
                        if let Some(gc) = line.split_once('=').map(|x| x.1) {
                            self.global_columns =
                                gc.trim().split(' ').map(ToString::to_string).collect();
                        }
                    }
                    curr_metadata.push(line.strip_prefix("#").unwrap().to_string());
                } else {
                    curr_sentence.push(line);
                }
            }
            return Some(Ok(parse_token_and_metadata(curr_metadata, curr_sentence)));
        }

        None
    }
}

struct TokenTreeGenerator<R: io::BufRead> {
    sent_iter: SentenceGenerator<R>,
}

impl<R: io::BufRead> Iterator for TokenTreeGenerator<R> {
    type Item = Result<TokenTree, io::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(sentence) = self.sent_iter.next() {
            match sentence {
                Ok(sentence) => Some(Ok(sentence.to_tree().unwrap())),
                Err(err) => Some(Err(err)),
            }
        } else {
            None
        }
    }
}
