use std::io;

use crate::models::{MetadataOrComment, Sentence, Token};

impl Sentence {
    pub fn serialize(&self, out: &mut impl io::Write) -> io::Result<()> {
        for meta_or_comm in self.metadata.iter() {
            match meta_or_comm {
                MetadataOrComment::Comment(comm) => writeln!(out, "# {}", comm)?,
                MetadataOrComment::Metadata { key, value } => {
                    writeln!(out, "# {} = {}", key, value)?
                }
            }
        }
        for token in &self.tokens {
            token.serialize(out)?;
        }
        Ok(())
    }
}

impl Token {
    pub fn serialize(&self, out: &mut impl io::Write) -> io::Result<()> {
        write!(out, "{}\t{}\t", self.id, self.form)?;
        write!(out, "{}\t", self.lemma.as_deref().unwrap_or("_"))?;
        write!(out, "{}\t", self.upos.as_deref().unwrap_or("_"))?;
        write!(out, "{}\t", self.xpos.as_deref().unwrap_or("_"))?;
        if let Some(feats) = &self.feats {
            let mut write_bar = false;
            for (feat, value) in feats {
                if write_bar {
                    write!(out, "|")?;
                } else {
                    write_bar = true;
                }
                write!(out, "{}={}", feat, value)?;
            }
        } else {
            write!(out, "_")?;
        }
        write!(out, "\t")?;
        if let Some(head) = self.head {
            write!(out, "{}\t", head)?;
        } else {
            write!(out, "_\t")?;
        }
        write!(out, "{}\t", self.deprel.as_deref().unwrap_or("_"))?;
        if let Some(deps) = &self.deps {
            let mut write_bar = false;
            for (rel, head) in deps {
                if write_bar {
                    write!(out, "|")?;
                } else {
                    write_bar = true;
                }
                write!(out, "{}:{}", head, rel)?;
            }
        } else {
            write!(out, "_")?;
        }
        write!(out, "\t")?;
        if let Some(misc) = &self.misc {
            let mut write_bar = false;
            for (key, value) in misc {
                if write_bar {
                    write!(out, "|")?;
                } else {
                    write_bar = true;
                }
                write!(out, "{}={}", key, value)?;
            }
        } else {
            write!(out, "_")?;
        }
        writeln!(out)?;
        Ok(())
    }
}
