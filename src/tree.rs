use std::{fmt, io};

use crate::models::{Id, Metadata, Sentence, Token};

pub trait ToTree {
    fn to_tree(self) -> Result<TokenTree, ToTreeError>;
}

#[derive(Debug)]
pub struct ToTreeError {
    msg: String,
}

impl ToTreeError {
    pub fn with_msg<S: Into<String>>(msg: S) -> ToTreeError {
        Self { msg: msg.into() }
    }
}

impl fmt::Display for ToTreeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl std::error::Error for ToTreeError {}

#[derive(Debug, Clone)]
pub struct TokenTree {
    token: Token,
    children: Vec<TokenTree>,
    metadata: Metadata,
}

impl TokenTree {
    pub fn new(token: Token, children: Vec<TokenTree>, metadata: Metadata) -> TokenTree {
        Self {
            token,
            children,
            metadata,
        }
    }

    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }
    pub fn set_metadata(&mut self, metadata: Metadata) {
        self.metadata = metadata;
    }

    pub fn print_tree<W: io::Write>(&self, out: &mut W, depth: usize) -> Result<(), io::Error> {
        let mut node_repr = String::new();
        node_repr.push_str("form:");
        node_repr.push_str(self.token.form());
        node_repr.push_str(" lemma:");
        node_repr.push_str(self.token.lemma().unwrap_or(""));
        node_repr.push_str(" upos:");
        node_repr.push_str(self.token.upos().unwrap_or(""));
        writeln!(
            out,
            "{}(deprel:{}) {} [{}]",
            " ".repeat(depth * 4),
            self.token.deprel().unwrap_or(""),
            node_repr,
            self.token.id()
        )?;
        for child in &self.children {
            child.print_tree(out, depth + 1)?;
        }
        Ok(())
    }
    pub fn into_sentence(self) -> Sentence {
        fn _to_list(mut token_list: Vec<Token>, children_: Vec<TokenTree>) -> Vec<Token> {
            for child in children_ {
                let TokenTree {
                    token,
                    children,
                    metadata: _,
                } = child;
                token_list.push(token);
                token_list = _to_list(token_list, children);
            }
            token_list
        }
        let Self {
            token,
            children,
            metadata,
        } = self;

        let mut token_list = _to_list(vec![token], children);
        token_list.sort_by(|a, b| match (a.id(), b.id()) {
            (Id::Single(ax), Id::Single(bx)) => ax.cmp(&bx),
            _ => todo!(),
        });
        Sentence::new(token_list, metadata)
    }
}
