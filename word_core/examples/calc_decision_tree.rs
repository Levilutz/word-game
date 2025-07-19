use std::{env::args, fs};

use word_core::{decision_tree::compute_node_aggressive, word::Word, word_search::SearchableWords};

fn load_words() -> Vec<Word<5>> {
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

    let possible_answers: SearchableWords<5, 26> = SearchableWords::build(words.clone());
    let (decision_tree, avg_perf) = compute_node_aggressive(&words, possible_answers, 0, false);
    println!("{:#?}", decision_tree);
    println!("avg perf: {}", avg_perf);
}
