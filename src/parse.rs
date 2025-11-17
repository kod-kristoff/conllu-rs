use std::{
    io::{self, Lines},
    mem,
};

use crate::{
    models::Sentence,
    parse::parser::{SentenceIter, parse_sentences, parse_token_and_metadata},
};

mod parser;

pub fn parse_incr<R: io::BufRead>(in_file: R) -> impl Iterator<Item = Result<Sentence, io::Error>> {
    SentenceGenerator {
        sentences: parse_sentences(in_file),
        global_columns: Vec::new(),
    }
}

struct SentenceGenerator<R: io::BufRead> {
    sentences: SentenceIter<R>,
    global_columns: Vec<String>,
}

impl<R: io::BufRead> Iterator for SentenceGenerator<R> {
    type Item = Result<Sentence, io::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
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
            } else {
                break;
            }
        }
        None
    }
}

// pub fn parse_sentences<R: io::BufRead>(in_file: R) -> SentenceIter<R> {
//     SentenceIter {
//         lines: in_file.lines(),
//         buf: Vec::new(),
//     }
// }
//
// fn parse_token_and_metadata()
//
// struct SentenceIter<R: io::BufRead> {
//     lines: Lines<R>,
//     buf: Vec<String>,
// }
//
// impl<R: io::BufRead> Iterator for SentenceIter<R> {
//     type Item = Result<Vec<String>, io::Error>;
//
//     fn next(&mut self) -> Option<Self::Item> {
//         loop {
//             if let Some(line) = self.lines.next() {
//                 let line = match line {
//                     Ok(x) => x,
//                     Err(err) => return Some(Err(err)),
//                 };
//                 if line.trim().is_empty() {
//                     if self.buf.is_empty() {
//                         continue;
//                     }
//                     return Some(Ok(mem::take(&mut self.buf)));
//                 } else {
//                     self.buf.push(line);
//                 }
//             } else {
//                 break;
//             }
//         }
//         if !self.buf.is_empty() {
//             return Some(Ok(mem::take(&mut self.buf)));
//         }
//         None
//     }
// }
