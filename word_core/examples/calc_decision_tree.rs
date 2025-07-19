use std::{env::args, fs};

use word_core::{decision_tree::compute_node_aggressive, word::Word, word_search::SearchableWords};

const WORD_SIZE: usize = 3;

fn load_words() -> Vec<Word<WORD_SIZE>> {
    let file_path = args()
        .nth(1)
        .expect("Must supply word list file as first arg");
    let file = fs::read_to_string(file_path).unwrap();
    file.split("\n")
        .map(|row| row.trim())
        .filter(|row| row.len() > 0)
        .map(|word| Word::from_str(word))
        .collect()
}

fn main() {
    let words = load_words();
    println!("loaded {} words", words.len());

    let possible_answers: SearchableWords<WORD_SIZE, 26> = SearchableWords::build(words.clone());
    let (decision_tree, est_cost) = compute_node_aggressive(&words, possible_answers, 0, 4, false)
        .expect("failed to compute top-level result");
    println!("{:#?}", decision_tree);
    println!("est cost: {}", est_cost);
}
