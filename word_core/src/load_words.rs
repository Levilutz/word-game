use std::{env::args, fs};

use crate::word::Word;

pub fn load_words<const WORD_SIZE: usize, const ALPHABET_SIZE: u8>(
    file_path: &str,
) -> Vec<Word<WORD_SIZE, ALPHABET_SIZE>> {
    let file = fs::read_to_string(file_path).unwrap();
    file.split("\n")
        .map(|row| row.trim())
        .filter(|row| row.len() > 0)
        .map(|word| Word::from_str(word))
        .collect()
}

pub fn load_guesses_and_answers<const WORD_SIZE: usize, const ALPHABET_SIZE: u8>(
    allowed_guesses_file_path: &str,
    possible_answers_file_path: &str,
    do_print: bool,
) -> (
    Vec<Word<WORD_SIZE, ALPHABET_SIZE>>,
    Vec<Word<WORD_SIZE, ALPHABET_SIZE>>,
) {
    let mut allowed_guesses = load_words(allowed_guesses_file_path);
    if do_print {
        println!("loaded {} allowed guesses", allowed_guesses.len());
    }
    let possible_answers = load_words(possible_answers_file_path);
    if do_print {
        println!("loaded {} possible answers", possible_answers.len());
    }
    let mut additional_guesses_added = 0;
    for possible_answer in &possible_answers {
        if !allowed_guesses.contains(possible_answer) {
            additional_guesses_added += 1;
            allowed_guesses.push(*possible_answer);
        }
    }
    if do_print && additional_guesses_added != 0 {
        println!(
            "loaded {} additional allowed guesses from answer list",
            additional_guesses_added
        );
        println!("now {} allowed guesses", allowed_guesses.len());
    }
    (allowed_guesses, possible_answers)
}

pub fn load_guesses_and_answers_from_args<const WORD_SIZE: usize, const ALPHABET_SIZE: u8>(
    do_print: bool,
) -> (
    Vec<Word<WORD_SIZE, ALPHABET_SIZE>>,
    Vec<Word<WORD_SIZE, ALPHABET_SIZE>>,
) {
    load_guesses_and_answers(
        &args()
            .nth(1)
            .expect("Must supply allowed guesses word list file as first arg"),
        &args()
            .nth(2)
            .expect("Must supply possible answers word list file as second arg"),
        do_print,
    )
}
