use std::{env::args, fs, time::Instant};

use word_core::{
    hint::WordHint,
    query_generation::{clue_possible, clue_to_query},
    word::Word,
    word_search::SearchableWords,
};

const WORD_SIZE: usize = 5;
const ALPHABET_SIZE: u8 = 26;

fn load_words(file_path: String) -> Vec<Word<WORD_SIZE, ALPHABET_SIZE>> {
    let file = fs::read_to_string(file_path).unwrap();
    file.split("\n")
        .map(|row| row.trim())
        .filter(|row| row.len() > 0)
        .map(|word| Word::from_str(word))
        .collect()
}

fn main() {
    let allowed_guesses = load_words(
        args()
            .nth(1)
            .expect("Must supply allowed guesses word list file as first arg"),
    );
    println!("loaded {} allowed guesses", allowed_guesses.len());

    let possible_answers = load_words(
        args()
            .nth(2)
            .expect("Must supply possible answers word list file as second arg"),
    );
    println!("loaded {} possible answers", possible_answers.len());

    println!("<- testing simple scan ->");
    let start = Instant::now();
    let mut all_hints_simple: Vec<Vec<WordHint<WORD_SIZE>>> =
        Vec::with_capacity(allowed_guesses.len());
    for guess in &allowed_guesses {
        let mut hints_for_guess = Vec::with_capacity(possible_answers.len());
        for answer in &possible_answers {
            let hint = WordHint::from_guess_and_answer(guess, answer);
            hints_for_guess.push(hint);
        }
        all_hints_simple.push(hints_for_guess);
    }
    let total_elapsed = start.elapsed().as_secs_f64();
    println!("finished in {:.3}s", total_elapsed);

    println!("<- testing query engine scan ->");
    let start = Instant::now();
    let mut all_hints_query_engine: Vec<Vec<WordHint<WORD_SIZE>>> =
        Vec::with_capacity(allowed_guesses.len());
    let searchable_answers = SearchableWords::build(possible_answers.clone());
    for guess in &allowed_guesses {
        let mut hints_for_guess = vec![WordHint::default(); possible_answers.len()];
        for hint in WordHint::all_possible() {
            if !clue_possible(*guess, hint) {
                continue;
            }
            let answers_giving_this_hint_mask =
                searchable_answers.eval_query(clue_to_query(*guess, hint));
            for answer_ind in answers_giving_this_hint_mask.true_inds() {
                hints_for_guess[answer_ind] = hint;
            }
        }
        all_hints_query_engine.push(hints_for_guess);
    }
    let total_elapsed = start.elapsed().as_secs_f64();
    println!("finished in {:.3}s", total_elapsed);

    let mut found_diff = false;
    for guess_ind in 0..allowed_guesses.len() {
        for answer_ind in 0..possible_answers.len() {
            let hint_simple = all_hints_simple[guess_ind][answer_ind];
            let hint_query_engine = all_hints_query_engine[guess_ind][answer_ind];

            if hint_simple != hint_query_engine {
                found_diff = true;
                println!("<difference in results>");
                println!(
                    "simple:\nguess:  {}\nanswer: {}",
                    hint_simple.color_guess(&allowed_guesses[guess_ind]),
                    &possible_answers[answer_ind]
                );
                println!(
                    "query engine:\nguess:  {}\nanswer: {}",
                    hint_query_engine.color_guess(&allowed_guesses[guess_ind]),
                    &possible_answers[answer_ind]
                );
            }
        }
    }
    if !found_diff {
        println!("both scans gave equivalent results")
    }
}
