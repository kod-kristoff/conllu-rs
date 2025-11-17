use std::io;
use std::io::Write;

use conllu::parse_incr;

fn main() -> io::Result<()> {
    let mut stdin = io::stdin().lock();
    let mut stdout = io::stdout().lock();
    // extract_text(&mut stdin, &mut stdout)
    for sentence in parse_incr(&mut stdin) {
        let sentence = sentence?;
        writeln!(&mut stdout, "sentence: {}", sentence)?;
        for token in sentence.tokens() {
            writeln!(&mut stdout, "  token: {}", token.repr())?;
        }
    }
    Ok(())
}
