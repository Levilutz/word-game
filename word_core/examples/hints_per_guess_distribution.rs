use std::{
    collections::{HashMap, HashSet},
    time::Instant,
};

use word_core::{
    hint::WordHint,
    load_words::load_guesses_and_answers_from_args,
    query_generation::{clue_possible, clue_to_query},
    word::Word,
    word_search::SearchableWords,
};

const WORD_SIZE: usize = 5;
const ALPHABET_SIZE: u8 = 26;
const MAX_BAR_SIZE: f64 = 64.0;

fn main() {
    let (allowed_guesses, possible_answers) = load_guesses_and_answers_from_args(true);

    println!("<- running query engine scan ->");
    let start = Instant::now();
    let mut possible_hints_per_guess: HashMap<
        Word<WORD_SIZE, ALPHABET_SIZE>,
        HashSet<WordHint<WORD_SIZE>>,
    > = HashMap::new();
    let searchable_answers = SearchableWords::build(possible_answers);
    for guess in &allowed_guesses {
        let mut possible_hints: HashSet<WordHint<WORD_SIZE>> = HashSet::new();
        for hint in WordHint::all_possible() {
            if !clue_possible(*guess, hint) {
                continue;
            }
            let answers_giving_this_hint_mask =
                searchable_answers.eval_query(clue_to_query(*guess, hint));
            if answers_giving_this_hint_mask.count_true() > 0 {
                possible_hints.insert(hint);
            }
        }
        possible_hints_per_guess.insert(*guess, possible_hints);
    }
    let total_elapsed = start.elapsed().as_secs_f64();
    println!("finished in {:.3}s", total_elapsed);

    // Get the distribution of num possible hints across guesses
    let mut num_guesses_by_num_possible_hints: HashMap<usize, usize> = HashMap::new();
    for (_guess, possible_hints) in &possible_hints_per_guess {
        *num_guesses_by_num_possible_hints
            .entry(possible_hints.len())
            .or_insert(0) += 1
    }
    let max_num_guesses = *num_guesses_by_num_possible_hints.keys().max().unwrap();
    let max_possible_hints = *num_guesses_by_num_possible_hints.values().max().unwrap();
    println!("# hints\t# guesses");
    for i in 0..max_num_guesses {
        let num_guesses = num_guesses_by_num_possible_hints
            .get(&i)
            .cloned()
            .unwrap_or(0);
        let bar_size = MAX_BAR_SIZE * num_guesses as f64 / max_possible_hints as f64;
        let bar = (0..bar_size.round() as u64)
            .map(|_| "=")
            .collect::<Vec<&str>>()
            .join("");
        println!("{i}\t{num_guesses}\t| {bar}");
    }
    let possible_hints_as_list: Vec<usize> = possible_hints_per_guess
        .values()
        .map(|possible_hints| possible_hints.len())
        .collect();
    let avg_possible_hints =
        possible_hints_as_list.iter().sum::<usize>() as f64 / possible_hints_as_list.len() as f64;
    println!("\navg {:.3} possible hints per guess", avg_possible_hints);
}
