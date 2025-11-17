use std::{fs::File, io};

use conllu::{models::Sentence, parse, parse_incr};
use rstest::rstest;

#[rstest]
#[case("assets/test_cases/empty-node.conllu")]
#[case("assets/test_cases/en_ewt-ud-test_excerp.conllu")]
#[case("assets/test_cases/long-token-to-text.conllu")]
#[case("assets/test_cases/multiword.conllu")]
#[case("assets/test_cases/paragraph-and-document.conllu")]
#[case("assets/test_cases/paragraph-in-sentence.conllu")]
#[case("assets/test_cases/space-after-no.conllu")]
fn test_parse_incr(#[case] case_path: &str) -> eyre::Result<()> {
    let file = File::open(case_path)?;
    let rdr = io::BufReader::new(file);
    let sentences = parse(rdr)?;

    insta::assert_debug_snapshot!(format!("test_parse_incr-{case_path}"), sentences);
    Ok(())
}
