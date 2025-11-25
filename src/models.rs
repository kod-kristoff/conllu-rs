use core::slice::Iter;
use std::{borrow::Borrow, collections::HashMap, fmt};

pub use crate::models::id::{Id, ParseIdError};
use crate::{de, parse, tree};

mod id;
mod ser;

pub type Dict<K, V> = ordermap::OrderMap<K, V>;

// pub type Metadata = Vec<MetadataOrComment>;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Metadata {
    store: Vec<MetadataOrComment>,
}

impl Metadata {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn is_empty(&self) -> bool {
        self.store.is_empty()
    }
    pub fn push(&mut self, value: MetadataOrComment) {
        self.store.push(value);
    }
    pub fn get<Q>(&self, q: &Q) -> Option<&String>
    where
        String: Borrow<Q> + PartialEq<Q>,
        Q: Eq + ?Sized,
    {
        for val in &self.store {
            if let MetadataOrComment::Metadata { key, value } = val
                && key == q.borrow()
            {
                return Some(value);
            }
        }
        None
    }
    pub fn iter(&self) -> Iter<'_, MetadataOrComment> {
        self.store.iter()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MetadataOrComment {
    Comment(String),
    Metadata { key: String, value: String },
}

#[derive(Debug, Clone)]
pub struct Token {
    id: Id,
    form: String,
    lemma: Option<String>,
    upos: Option<String>,
    xpos: Option<String>,
    feats: Option<Dict<String, String>>,
    head: Option<i16>,
    deprel: Option<String>,
    deps: Option<Vec<(String, Id)>>,
    misc: Option<Dict<String, String>>,
}

impl Token {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: Id,
        form: String,
        lemma: Option<String>,
        upos: Option<String>,
        xpos: Option<String>,
        feats: Option<Dict<String, String>>,
        head: Option<i16>,
        deprel: Option<String>,
        deps: Option<Vec<(String, Id)>>,
        misc: Option<Dict<String, String>>,
    ) -> Token {
        Self {
            id,
            form,
            lemma,
            upos,
            xpos,
            feats,
            head,
            deprel,
            deps,
            misc,
        }
    }

    pub fn id(&self) -> Id {
        self.id
    }
    pub fn form(&self) -> &str {
        self.form.as_str()
    }
    pub fn lemma(&self) -> Option<&str> {
        self.lemma.as_deref()
    }
    pub fn upos(&self) -> Option<&str> {
        self.upos.as_deref()
    }
    pub fn xpos(&self) -> Option<&str> {
        self.xpos.as_deref()
    }
    pub fn head(&self) -> Option<i16> {
        self.head
    }
    pub fn deprel(&self) -> Option<&str> {
        self.deprel.as_deref()
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if !self.form.is_empty() {
            f.write_str(&self.form)
        } else {
            f.write_fmt(format_args!("id={}", self.id))
        }
    }
}

#[derive(Debug)]
pub struct Sentence {
    tokens: Vec<Token>,
    metadata: Metadata,
}

impl Sentence {
    pub fn new(tokens: Vec<Token>, metadata: Metadata) -> Sentence {
        Self { tokens, metadata }
    }

    pub fn tokens(&self) -> &[Token] {
        self.tokens.as_slice()
    }

    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    fn create_tree(head_to_token_map: &HashMap<i16, Vec<Token>>, id: i16) -> Vec<tree::TokenTree> {
        let Some(children) = head_to_token_map.get(&id) else {
            return vec![];
        };
        let mut token_trees = vec![];
        for child in children {
            let Id::Single(child_id) = child.id else {
                todo!()
            };
            token_trees.push(tree::TokenTree::new(
                child.clone(),
                Self::create_tree(head_to_token_map, child_id),
                Metadata::new(),
            ));
        }
        token_trees
    }
    fn head_to_token(tokens: Vec<Token>) -> Result<HashMap<i16, Vec<Token>>, tree::ToTreeError> {
        if tokens.is_empty() {
            return Err(tree::ToTreeError::with_msg(
                "Can't parse tree, need at least one token.",
            ));
        }
        if matches!(tokens[0].id, Id::Single(_)) && tokens[0].head.is_none() {
            return Err(tree::ToTreeError::with_msg(
                "Can't parse tree, missing 'head' field.",
            ));
        }

        let mut head_indexed = HashMap::new();
        for token in tokens {
            if matches!(token.id, Id::Dot(_, _) | Id::Range(_, _)) {
                continue;
            }
            let Some(head) = token.head else {
                continue;
            };
            if head < 0 {
                continue;
            }
            head_indexed
                .entry(head)
                .or_insert_with(Vec::new)
                .push(token);
        }
        if !head_indexed.contains_key(&0) {
            Err(tree::ToTreeError::with_msg(
                "Found no head node, can't build tree",
            ))
        } else {
            Ok(head_indexed)
        }
    }
}

impl tree::ToTree for Sentence {
    fn to_tree(self) -> Result<tree::TokenTree, tree::ToTreeError> {
        let Self { tokens, metadata } = self;
        let mut head_indexed = Self::head_to_token(tokens)?;
        let mut roots = if head_indexed[&0].len() > 1 {
            head_indexed.insert(
                -1,
                vec![Token::new(
                    Id::Single(0),
                    "_".to_string(),
                    None,
                    None,
                    None,
                    None,
                    None,
                    Some("root".to_string()),
                    None,
                    None,
                )],
            );
            Self::create_tree(&head_indexed, -1)
        } else {
            Self::create_tree(&head_indexed, 0)
        };
        let mut root = roots.swap_remove(0);
        root.set_metadata(metadata);
        Ok(root)
    }
}

impl fmt::Display for Sentence {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("Sentence<")?;
        let mut write_comma = false;
        for token in &self.tokens {
            if write_comma {
                f.write_str(", ")?;
            } else {
                write_comma = true;
            }
            f.write_fmt(format_args!("{}", token))?;
        }
        if !self.metadata.is_empty() {
            f.write_str(", metdata={{")?;
            let mut write_comma = false;
            for meta_or_comm in self.metadata.iter() {
                if let MetadataOrComment::Metadata { key, value } = meta_or_comm {
                    if write_comma {
                        f.write_str(", ")?;
                    } else {
                        write_comma = true;
                    }
                    f.write_fmt(format_args!("{}=\"{:?}\"", key, value))?;
                }
            }
            f.write_str("}}")?;
        }
        f.write_str(">")?;
        Ok(())
    }
}

pub struct SentenceBuilder {
    metadata: Metadata,
    tokens: Vec<Token>,
}

impl de::Deserialize for Sentence {
    type Builder = SentenceBuilder;
    fn builder() -> Self::Builder {
        SentenceBuilder {
            metadata: Metadata::new(),
            tokens: Vec::new(),
        }
    }
}

impl de::SentenceBuilder for SentenceBuilder {
    type Builder = TokenBuilder;
    type Value = Sentence;
    type Token = Token;

    fn add_meta(&mut self, key: &str, value: &str) -> Result<(), de::DeError> {
        self.metadata.push(MetadataOrComment::Metadata {
            key: key.into(),
            value: value.into(),
        });
        Ok(())
    }
    fn add_comment(&mut self, comment: &str) -> Result<(), de::DeError> {
        self.metadata
            .push(MetadataOrComment::Comment(comment.into()));
        Ok(())
    }

    fn token_start(&self) -> TokenBuilder {
        TokenBuilder::default()
    }

    fn add_token(&mut self, token: Token) -> Result<(), de::DeError> {
        self.tokens.push(token);
        Ok(())
    }
    fn finish(self) -> Option<Sentence> {
        let Self { metadata, tokens } = self;
        Some(Sentence { tokens, metadata })
    }
}

#[derive(Debug, Default)]
pub struct TokenBuilder {
    id: Option<Id>,
    form: Option<String>,
    lemma: Option<String>,
    upos: Option<String>,
    xpos: Option<String>,
    feats: Option<Dict<String, String>>,
    head: Option<i16>,
    deprel: Option<String>,
    deps: Option<Vec<(String, Id)>>,
    misc: Option<Dict<String, String>>,
}

impl de::TokenBuilder for TokenBuilder {
    type Value = Token;

    fn add_field(&mut self, key: &str, value: &str) -> Result<(), de::DeError> {
        match key {
            "id" => {
                self.id = Some(
                    value
                        .parse()
                        .map_err(|err| de::DeError::invalid_value("id", err))?,
                );
            }
            "form" => {
                if value.is_empty() {
                    return Err(de::DeError::invalid_value("form", "Cannot be empty"));
                }
                self.form = Some(value.to_string());
            }
            "lemma" => self.lemma = parse::parse_nullable_value(value).map(ToString::to_string),
            "upos" => self.upos = parse::parse_nullable_value(value).map(ToString::to_string),
            "xpos" => self.xpos = parse::parse_nullable_value(value).map(ToString::to_string),
            "feats" => self.feats = parse::parse_dict_value(value),
            "head" => {
                self.head = parse::parse_from_str_nullable(value)
                    .map_err(|err| de::DeError::invalid_value("head", err))?;
            }
            "deprel" => self.deprel = parse::parse_nullable_value(value).map(ToString::to_string),
            "deps" => {
                self.deps = parse::parse_paired_list_value(value)
                    .map_err(|err| de::DeError::invalid_value("deps", err))?;
            }
            "misc" => self.misc = parse::parse_dict_value(value),
            &_ => {
                return Err(de::DeError::unknown_field(
                    key,
                    &[
                        "id", "form", "lemma", "upos", "xpos", "feats", "head", "deprel", "deps",
                        "misc",
                    ],
                ));
            }
        }
        Ok(())
    }
    fn finish(self) -> Result<Token, de::DeError> {
        let Self {
            id,
            form,
            lemma,
            upos,
            xpos,
            feats,
            head,
            deprel,
            deps,
            misc,
        } = self;
        let Some(id) = id else {
            return Err(de::DeError::missing_field("id"));
        };
        let Some(form) = form else {
            return Err(de::DeError::missing_field("form"));
        };
        Ok(Token::new(
            id, form, lemma, upos, xpos, feats, head, deprel, deps, misc,
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::{de::TokenBuilder as _, tree::ToTree};

    use super::*;

    fn mk_token(id: Id, form: &str) -> Token {
        Token {
            id,
            form: form.to_string(),
            lemma: None,
            upos: None,
            xpos: None,
            feats: None,
            head: None,
            deprel: None,
            deps: None,
            misc: None,
        }
    }

    #[test]
    fn token_builder_unknown_field() {
        let mut builder = TokenBuilder::default();
        let res = builder.add_field("unknown", "field").unwrap_err();

        insta::assert_snapshot!(res);
    }
    mod to_tree_failures {

        use crate::tree::ToTree;

        use super::*;

        #[test]
        fn empty_sentence_fails() {
            let empty = Sentence::new(Vec::new(), Metadata::new());

            let res = empty.to_tree().unwrap_err();

            insta::assert_snapshot!(res);
        }

        #[test]
        fn missing_head_fails() {
            let sent = Sentence::new(vec![mk_token(Id::Single(1), "form")], Metadata::new());

            let res = sent.to_tree().unwrap_err();

            insta::assert_snapshot!(res);
        }
        #[test]
        fn no_root_fails() {
            let sent = Sentence::new(
                vec![Token::new(
                    Id::Single(1),
                    "form".into(),
                    None,
                    None,
                    None,
                    None,
                    Some(2),
                    None,
                    None,
                    None,
                )],
                Metadata::new(),
            );

            let res = sent.to_tree().unwrap_err();

            insta::assert_snapshot!(res);
        }
    }

    #[test]
    fn sentence_with_2_roots_builds_tree() {
        let sent = Sentence::new(
            vec![
                Token::new(
                    Id::Single(1),
                    "form1".into(),
                    None,
                    None,
                    None,
                    None,
                    Some(0),
                    Some("root".into()),
                    None,
                    None,
                ),
                Token::new(
                    Id::Single(2),
                    "form2".into(),
                    None,
                    None,
                    None,
                    None,
                    Some(0),
                    Some("root".into()),
                    None,
                    None,
                ),
            ],
            Metadata::new(),
        );

        let res = sent.to_tree().unwrap();

        insta::assert_debug_snapshot!(res);
    }
}
