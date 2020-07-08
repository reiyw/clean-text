use anyhow::Result;
use std::collections::HashSet;
use std::env;
use std::fs;
use std::fs::File;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let data = fs::read_to_string(&args[1])?;
    let words: HashSet<&str> = data
        .lines()
        .map(|line| line.rsplitn(2, '/').last().unwrap())
        .collect();

    let out_file = File::create("assets/words.bin")?;
    bincode::serialize_into(out_file, &words)?;

    Ok(())
}
