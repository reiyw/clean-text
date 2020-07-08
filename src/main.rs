use anyhow::Result;
use itertools::Itertools;
use onig::Regex;
use rust_embed::RustEmbed;
use std::collections::HashSet;
use std::io::{self, Read};

#[derive(RustEmbed)]
#[folder = "assets/"]
struct Asset;

fn main() -> Result<()> {
    let mut text = String::new();
    io::stdin().read_to_string(&mut text)?;

    println!("{}", clean_text(text));

    // if may_contain_sentence_fragments(text.lines()) {
    //     println!("{}", clean_text(text));
    // } else {
    //     println!("{}", text);
    // }

    Ok(())
}

fn clean_text(text: impl AsRef<str>) -> String {
    let words_bin = Asset::get("words.bin").unwrap();
    let words: HashSet<&str> = bincode::deserialize(&words_bin).unwrap();
    split_into_paragraphs(text)
        .iter()
        .map(|p| {
            let p = join_sentence_fragments(&p, &words);
            let sents = split_into_sentences(p);
            sents.join("\n")
        })
        .join("\n\n")
}

fn may_contain_sentence_fragments(lines: impl IntoIterator<Item = impl AsRef<str>>) -> bool {
    lines
        .into_iter()
        .filter_map(|line| line.as_ref().chars().last())
        .any(|c| c.is_ascii_alphanumeric() || c == '-')
}

fn split_into_paragraphs(text: impl AsRef<str>) -> Vec<String> {
    text.as_ref()
        .split("\n\n")
        .map(|p| p.trim().into())
        .collect()
}

fn join_sentence_fragments(paragraph: impl AsRef<str>, word_list: &HashSet<&str>) -> String {
    let lines: Vec<&str> = paragraph.as_ref().lines().collect();
    let mut words: Vec<String> = lines[0].split_whitespace().map(|w| w.into()).collect();
    for (&l1, &l2) in lines.iter().tuple_windows() {
        if let Some(c) = l1.chars().last() {
            if c == '-' {
                let left_word_fragment = l1.split_whitespace().last().unwrap();
                let right_word_fragment = l2.split_ascii_whitespace().next().unwrap();
                let n = left_word_fragment.chars().count();
                let byte_pos_before_hyphenation = left_word_fragment
                    .char_indices()
                    .map(|(i, _)| i)
                    .nth(n - 1)
                    .unwrap();
                let joined_word = {
                    let word = [
                        &left_word_fragment[0..byte_pos_before_hyphenation],
                        right_word_fragment,
                    ]
                    .concat();
                    if word_list.contains(&word.as_str()) {
                        word
                    } else {
                        [&left_word_fragment, right_word_fragment].concat()
                    }
                };
                let n = words.len();
                words[n - 1] = joined_word;
                words.extend(l2.split_ascii_whitespace().skip(1).map(|w| w.into()));
            } else {
                words.extend(l2.split_whitespace().map(|w| w.into()))
            }
        } else {
            unreachable!();
        }
    }
    words.join(" ")
}

fn split_into_sentences(paragraph: impl AsRef<str>) -> Vec<String> {
    // copy from https://stackoverflow.com/a/25736082/8607997
    let re = Regex::new(r"(?<!\w\.\w.)(?<![A-Z][a-z]\.)(?<=\.|\?)\s").unwrap();
    re.split(paragraph.as_ref()).map(|s| s.into()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_join_sentence_fragments() {
        let words_bin = Asset::get("words.bin").unwrap();
        let words: HashSet<&str> = bincode::deserialize(&words_bin).unwrap();
        let testcases = [
            ("practical-\nly", "practically"),
            ("practical-\nly easy", "practically easy"),
            ("practically\neasy", "practically easy"),
            ("state-\nof-the-art method", "state-of-the-art method"),
        ];
        for (input, expected) in &testcases {
            assert_eq!(join_sentence_fragments(input, &words), expected.to_string());
        }
    }
}
