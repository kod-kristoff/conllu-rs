use std::io;
use std::io::Write;

use conllu::models::Sentence;

fn main() -> eyre::Result<()> {
    let mut stdin = io::stdin().lock();
    let mut stdout = io::stdout().lock();
    // extract_text(&mut stdin, &mut stdout)
    for sentence in conllu::parse_incr(&mut stdin) {
        let sentence: Sentence = sentence?;
        writeln!(&mut stdout, "sentence: {}", sentence)?;
        for token in sentence.tokens() {
            writeln!(&mut stdout, "  token: {:?}", token)?;
        }
    }
    Ok(())
}
