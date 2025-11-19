use std::fmt;

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
    Single(u8),
    Range(u8, u8),
    Dot(u8, u8),
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
    head: Option<u8>,
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
        head: Option<u8>,
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
