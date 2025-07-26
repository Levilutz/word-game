use std::env::args;

use word_core::{
    decision_tree::compute_node_aggressive, load_words::load_words, word_search::SearchableWords,
};

const WORD_SIZE: usize = 3;

fn main() {
    let words = load_words(&args().nth(1).expect("Must supply word list as first arg"));
    println!("loaded {} words", words.len());

    let possible_answers: SearchableWords<WORD_SIZE, 26> = SearchableWords::build(words.clone());
    let (decision_tree, est_cost) = compute_node_aggressive(&words, possible_answers, 0, 4, false)
        .expect("failed to compute top-level result");
    println!("{}", serde_json::to_string_pretty(&decision_tree).unwrap());
    println!("est cost: {}", est_cost);
}
