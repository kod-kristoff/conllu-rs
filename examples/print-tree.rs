use std::{
    env::args,
    fs::File,
    io::{self, BufReader},
};

use conllu::{models::Sentence, parse_tree_incr};

fn main() -> eyre::Result<()> {
    let Some(case_path) = args().nth(1) else {
        todo!()
    };
    let file = File::open(case_path)?;
    let rdr = BufReader::new(file);
    let mut stdout = io::stdout().lock();
    for token_tree in parse_tree_incr::<Sentence, _>(rdr) {
        let token_tree = token_tree?;
        if let Some(sent_id) = token_tree.metadata().get("sent_id") {
            println!("{{'sent_id': '{}'}}", sent_id);
        }
        token_tree.print_tree(&mut stdout, 0)?;
        println!();
    }
    Ok(())
}
