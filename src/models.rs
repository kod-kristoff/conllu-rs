use std::{collections::HashMap, fmt, io};

mod ser;

pub type Dict<K, V> = ordermap::OrderMap<K, V>;

pub type Metadata = Vec<MetadataOrComment>;

#[derive(Debug, Clone, PartialEq)]
pub enum MetadataOrComment {
    Comment(String),
    Metadata { key: String, value: String },
}

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
    pub fn repr(&self) -> Repr<'_> {
        Repr(self)
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

pub struct Repr<'a>(&'a Token);

impl<'a> fmt::Display for Repr<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("{\n")?;
        writeln!(f, "    'id': {},", self.0.id)?;
        writeln!(f, "    'form': '{}',", self.0.form)?;
        if let Some(lemma) = &self.0.lemma {
            writeln!(f, "    'lemma': '{}',", lemma)?;
        }
        if let Some(upos) = &self.0.upos {
            writeln!(f, "    'upos': '{}',", upos)?;
        }
        if let Some(xpos) = &self.0.xpos {
            writeln!(f, "    'xpos': '{}',", xpos)?;
        }
        if let Some(feats) = &self.0.feats {
            writeln!(f, "    'feats': {:?},", feats)?;
        }
        if let Some(head) = &self.0.head {
            writeln!(f, "    'head': {},", head)?;
        }
        if let Some(deprel) = &self.0.deprel {
            writeln!(f, "    'deprel': '{}',", deprel)?;
        }
        if let Some(deps) = &self.0.deps {
            writeln!(f, "    'deps': {},", DisplayDeps(deps))?;
        }
        if let Some(misc) = &self.0.misc {
            writeln!(f, "    'misc': {:?},", misc)?;
        }
        f.write_str("  }\n")?;
        Ok(())
    }
}

pub struct DisplayDeps<'a>(&'a [(String, Id)]);

impl<'a> fmt::Display for DisplayDeps<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("[")?;
        let mut write_comma = false;
        for dep in self.0 {
            if write_comma {
                f.write_str(", ")?;
            } else {
                write_comma = true;
            }
            write!(f, "('{}', {})", dep.0, dep.1)?;
        }
        f.write_str("]")?;
        Ok(())
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

    pub fn to_tree(self) -> Result<TokenTree, ToTreeError> {
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
        root.metadata = metadata;
        Ok(root)
    }
    fn create_tree(head_to_token_map: &HashMap<i16, Vec<Token>>, id: i16) -> Vec<TokenTree> {
        let Some(children) = head_to_token_map.get(&id) else {
            return vec![];
        };
        let mut token_trees = vec![];
        for child in children {
            let Id::Single(child_id) = child.id else {
                todo!()
            };
            token_trees.push(TokenTree {
                token: child.clone(),
                children: Self::create_tree(head_to_token_map, child_id),
                metadata: Metadata::new(),
            });
        }
        token_trees
    }
    fn head_to_token(tokens: Vec<Token>) -> Result<HashMap<i16, Vec<Token>>, ToTreeError> {
        if tokens.is_empty() {
            return Err(ToTreeError::with_msg(
                "Can't parse tree, need at least one token.",
            ));
        }
        if matches!(tokens[0].id, Id::Single(_)) && tokens[0].head.is_none() {
            return Err(ToTreeError::with_msg(
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
            Err(ToTreeError::with_msg(
                "Found no head node, can't build tree",
            ))
        } else {
            Ok(head_indexed)
        }
    }
}

#[derive(Debug)]
pub struct ToTreeError {
    msg: String,
}

impl ToTreeError {
    fn with_msg<S: Into<String>>(msg: S) -> ToTreeError {
        Self { msg: msg.into() }
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

    pub fn print_tree<W: io::Write>(&self, out: &mut W, depth: usize) -> Result<(), io::Error> {
        let mut node_repr = String::new();
        node_repr.push_str("form:");
        node_repr.push_str(&self.token.form);
        node_repr.push_str(" lemma:");
        node_repr.push_str(self.token.lemma.as_deref().unwrap_or(""));
        node_repr.push_str(" upos:");
        node_repr.push_str(self.token.upos.as_deref().unwrap_or(""));
        writeln!(
            out,
            "{}(deprel:{}) {} [{}]",
            " ".repeat(depth * 4),
            self.token.deprel.as_deref().unwrap_or(""),
            node_repr,
            self.token.id
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
        Sentence {
            tokens: token_list,
            metadata,
        }
    }
}
