use std::{fs::File, io};

use conllu::{models::Sentence, parse};
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
    let sentences: Vec<Sentence> = parse(rdr)?;

    insta::assert_debug_snapshot!(format!("test_parse_incr-{case_path}"), sentences);
    Ok(())
}

#[rstest]
fn empty_source_files() -> eyre::Result<()> {
    let res: Result<Vec<Sentence>, conllu::de::DeError> = parse("".as_bytes());

    insta::assert_debug_snapshot!(res);
    Ok(())
}
mod failures {
    use conllu::{models::Sentence, parse_tree};
    use rstest::rstest;

    #[rstest]
    #[case::no_tokens("# sent_id = 1")]
    #[case::no_form("# sent_id = 2\n1")]
    #[case::bad_id("# sent_id = 3\nk")]
    fn bad_source_files(#[case] data: &str) -> eyre::Result<()> {
        let res = parse_tree::<Sentence, _>(data.as_bytes());

        insta::assert_debug_snapshot!(
            format!("failures_bad_source_files-data-{}", data.replace('\n', "_")),
            res
        );
        Ok(())
    }
}

#[allow(non_snake_case)]
mod TestParse {
    use conllu::{models::Sentence, parse, parse_tree};
    use rstest::{fixture, rstest};

    use crate::fixtures;

    #[fixture]
    fn data() -> &'static str {
        fixtures::QUICK_FOX
    }

    #[rstest]
    fn test_parse(data: &str) -> eyre::Result<()> {
        let sentences: Vec<Sentence> = parse(data.as_bytes())?;
        insta::assert_debug_snapshot!(sentences);
        assert!(sentences[0].metadata().get("text").is_some());
        Ok(())
    }

    #[rstest]
    fn test_parse_tree(data: &str) -> eyre::Result<()> {
        println!("data={}", data);
        let sentences = parse_tree::<Sentence, _>(data.as_bytes())?;
        insta::assert_debug_snapshot!(sentences);

        let root = &sentences[0];
        println!("root={:?}", &root);
        let mut buf = Vec::new();

        root.print_tree(&mut buf, 0)?;

        let printed_tree = String::from_utf8(buf)?;
        insta::assert_snapshot!(printed_tree);
        Ok(())
    }
}

#[allow(non_snake_case)]
mod TestTrickyCases {
    use std::io::{self, Cursor};

    use conllu::{
        models::{Id, Sentence},
        parse, parse_tree,
    };
    use rstest::rstest;

    use crate::fixtures::TESTCASES;

    #[rstest]
    #[case::arabic(0)]
    #[case::russian(1)]
    #[case::english(2)]
    fn test_parse_and_serialize(#[case] i: usize) -> eyre::Result<()> {
        let testcase = TESTCASES[i];
        let sentences: Vec<Sentence> = parse(testcase.as_bytes())?;
        insta::assert_debug_snapshot!(format!("test_parse_and_serialize-case{i}"), sentences);

        let mut wrtr = io::Cursor::new(vec![]);

        sentences[0].serialize(&mut wrtr)?;

        let buf = wrtr.into_inner();
        let actual = String::from_utf8(buf)?;

        assert_eq!(actual, testcase);
        Ok(())
    }

    #[rstest]
    #[case::arabic(0)]
    #[case::russian(1)]
    #[case::english(2)]
    fn test_parse_tree_and_serialize(#[case] i: usize) -> eyre::Result<()> {
        let testcase = TESTCASES[i];
        let data: Vec<Sentence> = parse(testcase.as_bytes())?;
        let tokens = data[0]
            .tokens()
            .iter()
            .filter(|t| matches!(t.id(), Id::Single(_)))
            .cloned()
            .collect();

        let testcase_without_range_and_elided = Sentence::new(tokens, data[0].metadata().clone());
        let actual = parse_tree::<Sentence, _>(testcase.as_bytes())?[0].clone();
        let mut buf = Cursor::new(Vec::new());
        let actual = actual.into_sentence();
        actual.serialize(&mut buf)?;
        let actual_serialized = String::from_utf8(buf.into_inner())?;

        let mut buf = Cursor::new(Vec::new());
        testcase_without_range_and_elided.serialize(&mut buf)?;
        let expected = String::from_utf8(buf.into_inner())?;
        assert_eq!(actual_serialized, expected);
        Ok(())
    }
}
//
// @pytest.mark.integration
// mod TestParseCoNLLUPlus {
//     def test_parse_conllu_plus(self):
//         # Note: global.columns affects both sentences
//         data = dedent("""\
//             # global.columns = ID FORM UPOS HEAD DEPREL MISC PARSEME:MWE
//             # source_sent_id = conllu http://hdl.handle.net/11234/1-2837 UD_German-GSD/de_gsd-ud-train.conllu
//             # sent_id = train-s16
//             # text = Der CDU-Politiker strebt
//             1\tDer\tDET\t2\tdet\t_\t*
//             2\tCDU\tPROPN\t4\tcompound\tSpaceAfter=No\t*
//             3\t-\tPUNCT\t2\tpunct\tSpaceAfter=No\t*
//             4\tPolitiker\tNOUN\t5\tnsubj\t_\t*
//             5\tstrebt\tVERB\t0\troot\t_\t2:VPC.full
//
//             # source_sent_id = conllu http://hdl.handle.net/11234/1-2837 UD_German-GSD/de_gsd-ud-train.conllu
//             # sent_id = train-s17
//             # text = Der ortsüblichen Vergleichsmiete orientieren.
//             1\tDer\tDET\t19\tdet\t_\t*
//             2\tortsüblichen\tADJ\t19\tamod\t_\t*
//             3\tVergleichsmiete\tNOUN\t20\tobl\t_\t*
//             4\torientieren\tVERB\t8\tacl\tSpaceAfter=No\t1
//             5\t.\tPUNCT\t5\tpunct\t_\t*
//         """)
//
//         sentences = parse(data)
//
//         self.assertEqual(
//             [
//                 {"form": token["form"], "parseme:mwe": token["parseme:mwe"]}
//                 for token in sentences[0]
//             ],
//             [
//                 {"form": "Der", "parseme:mwe": "*"},
//                 {"form": "CDU", "parseme:mwe": "*"},
//                 {"form": "-", "parseme:mwe": "*"},
//                 {"form": "Politiker", "parseme:mwe": "*"},
//                 {"form": "strebt", "parseme:mwe": "2:VPC.full"},
//             ]
//         )
//         self.assertEqual(sentences[0].metadata, {
//             "global.columns": "ID FORM UPOS HEAD DEPREL MISC PARSEME:MWE",
//             "source_sent_id": "conllu http://hdl.handle.net/11234/1-2837 UD_German-GSD/de_gsd-ud-train.conllu",
//             "sent_id": "train-s16",
//             "text": "Der CDU-Politiker strebt",
//         })
//         self.assertEqual(
//             [
//                 {"form": token["form"], "parseme:mwe": token["parseme:mwe"]}
//                 for token in sentences[1]
//             ],
//             [
//                 {"form": "Der", "parseme:mwe": "*"},
//                 {"form": "ortsüblichen", "parseme:mwe": "*"},
//                 {"form": "Vergleichsmiete", "parseme:mwe": "*"},
//                 {"form": "orientieren", "parseme:mwe": "1"},
//                 {"form": ".", "parseme:mwe": "*"},
//             ]
//         )
//         self.assertEqual(sentences[1].metadata, {
//             "source_sent_id": "conllu http://hdl.handle.net/11234/1-2837 UD_German-GSD/de_gsd-ud-train.conllu",
//             "sent_id": "train-s17",
//             "text": "Der ortsüblichen Vergleichsmiete orientieren.",
//         })
//
//
// @pytest.mark.integration
// mod TestParseCoNLL2009 {
//     def test_parse_CoNLL2009_1(self):
//         data = dedent("""\
//             #\tid\tform\tlemma\tplemma\tpos\tppos\tfeats\tpfeats\thead\tphead\tdeprel\tpdeprel\tfillpred\tpred\tapreds
//             1\tZ\tz\tz\tR\tR\tSubPOS=R|Cas=2\tSubPOS=R|Cas=2\t10\t10\tAuxP\tAuxP\t_\t_\t_\t_\t_\t_\t_\t_\t_\t_\t_\t_\t_\t_\t_\t_\t_\t_
//             2\ttéto\ttento\ttento\tP\tP\tSubPOS=D|Gen=F|Num=S|Cas=2\tSubPOS=D|Gen=F|Num=S|Cas=2\t3\t3\tAtr\tAtr\tY\ttento\t_\tRSTR\t_\t_\t_\t_\t_\t_\t_\t_\t_\t_\t_\t_\t_\t_
//             3\tknihy\tkniha\tkniha\tN\tN\tSubPOS=N|Gen=F|Num=S|Cas=2|Neg=A\tSubPOS=N|Gen=F|Num=S|Cas=2|Neg=A\t1\t1\tAdv\tAdv\tY\tkniha\t_\t_\t_\t_\t_\t_\t_\tDIR1\t_\t_\t_\t_\t_\t_\t_\t_
//
//         """)
//
//         sentences = parse(
//             data,
//             fields=(
//                 'id', 'form', 'lemma', 'plemma', 'pos', 'ppos', 'feats', 'pfeats',
//                 'head', 'phead', 'deprel', 'pdeprel', 'fillpred', 'pred', 'apreds'
//             ),
//             field_parsers={
//                 "pfeats": lambda line, i: parse_dict_value(line[i]),
//                 "phead": lambda line, i: parse_int_value(line[i]),
//                 "apreds": lambda line, i: [
//                     apred_field if apred_field != "_" else None
//                     for apred_field in line[i:len(line)]
//                 ],
//             },
//         )
//         self.assertEqual(
//             sentences[0][2],
//             Token([
//                 ('id', 3),
//                 ('form', 'knihy'),
//                 ('lemma', 'kniha'),
//                 ('plemma', 'kniha'),
//                 ('pos', 'N'),
//                 ('ppos', 'N'),
//                 ('feats', Token([
//                     ('SubPOS', 'N'),
//                     ('Gen', 'F'),
//                     ('Num', 'S'),
//                     ('Cas', '2'),
//                     ('Neg', 'A')
//                 ])),
//                 ('pfeats', Token([
//                     ('SubPOS', 'N'),
//                     ('Gen', 'F'),
//                     ('Num', 'S'),
//                     ('Cas', '2'),
//                     ('Neg', 'A')
//                 ])),
//                 ('head', 1),
//                 ('phead', 1),
//                 ('deprel', 'Adv'),
//                 ('pdeprel', 'Adv'),
//                 ('fillpred', 'Y'),
//                 ('pred', 'kniha'),
//                 ('apreds', [
//                     None, None, None, None, None, None, None, 'DIR1',
//                     None, None, None, None, None, None, None, None
//                 ])
//             ])
//         )
//
//     def test_parse_CoNLL2009_2(self):
//         data = dedent("""\
//             #\tid='1'-document_id='36:1047'-span='1'
//             1\t+\t+\tPunc\tPunc\t_\t0\tROOT\t_\t_
//             2\tIn\tin\tr\tr\tr|-|-|-|-|-|-|-|-\t5\tAuxP\t_\t_
//             3\tDei\tDeus\tn\tPropn\tn|-|s|-|-|-|m|g|-\t4\tATR\t_\t_
//             4\tnomine\tnomen\tn\tn\tn|-|s|-|-|-|n|b|-\t2\tADV\t_\t_
//             5\tregnante\tregno\tt\tt\tt|-|s|p|p|a|m|b|-\t0\tADV\t_\t_
//
//         """)
//
//         sentences = parse(
//             data,
//             fields=(
//                 'id', 'form', 'lemma', 'upostag', 'xpostag', 'feats', 'head', 'deprel', 'deps', 'misc'
//             ),
//             field_parsers={
//                 "feats": lambda line, i: [feat for feat in line[i].split("|")]
//             }
//         )
//         self.assertEqual(
//             sentences[0][4],
//             Token([
//                 ('id', 5),
//                 ('form', 'regnante'),
//                 ('lemma', 'regno'),
//                 ('upostag', 't'),
//                 ('xpostag', 't'),
//                 ('feats', ['t', '-', 's', 'p', 'p', 'a', 'm', 'b', '-']),
//                 ('head', 0),
//                 ('deprel', 'ADV'),
//                 ('deps', None),
//                 ('misc', None),
//             ])
//         )
//         self.assertEqual(
//             sentences[0].metadata,
//             Token([
//                 ('id', "'1'-document_id='36:1047'-span='1'")
//             ])
//         )
