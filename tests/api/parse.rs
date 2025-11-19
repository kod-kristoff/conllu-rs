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

mod TestParse {
    use conllu::parse;
    use rstest::{fixture, rstest};

    use crate::fixtures;

    #[fixture]
    fn data() -> &'static str {
        fixtures::QUICK_FOX
    }

    #[rstest]
    fn test_parse(data: &str) -> eyre::Result<()> {
        let sentences = parse(data.as_bytes())?;
        insta::assert_debug_snapshot!(sentences);
        Ok(())
    }

}

mod TestTrickyCases {
    use std::io;

    use conllu::parse;
    use rstest::rstest;

    use crate::fixtures::TESTCASES;

    #[rstest]
    #[case::arabic(0)]
    #[case::russian(1)]
    #[case::english(2)]
    fn test_parse_and_serialize(#[case] i: usize) -> eyre::Result<()> {
        let testcase = TESTCASES[i];
        let sentences = parse(testcase.as_bytes())?;
        insta::assert_debug_snapshot!(format!("test_parse_and_serialize-case{i}"), sentences);

        let mut wrtr = io::Cursor::new(vec![]);

        sentences[0].serialize(&mut wrtr)?;

        let buf = wrtr.into_inner();
        let actual = String::from_utf8(buf)?;

        assert_eq!(actual, testcase);
        Ok(())
    }
}
