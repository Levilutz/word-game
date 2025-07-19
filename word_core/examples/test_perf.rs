use std::{cmp::min, env::args, fs, time::Instant};

use word_core::{
    dumb_word_search::dumb_search_words, hint::WordHint, query_generation::clue_to_query,
    word::Word, word_search::SearchableWords,
};

const WORD_SIZE: usize = 5;

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

    let limit_trials: Option<usize> = args().nth(2).map(|limit| limit.parse().ok()).flatten();

    let num_trials = match limit_trials {
        Some(limit) => min(limit, words.len() * words.len()),
        None => words.len() * words.len(),
    };

    println!("loaded {} words, {} test cases", words.len(), num_trials);

    let smart_search: SearchableWords<WORD_SIZE, 26> = SearchableWords::build(words.clone());

    println!("<- testing dumb search ->");
    let start = Instant::now();
    let mut i = 0;
    for answer in &words {
        for guess in &words {
            if i >= num_trials {
                break;
            }
            if i % 10000 == 0 {
                let elapsed_s = start.elapsed().as_secs_f64();
                let total_est = (elapsed_s * num_trials as f64) / i as f64;
                println!(
                    "finished {} in {:.3}s - {:.2} iter/s - {:.3}s remaining - {:.3}s total",
                    i,
                    elapsed_s,
                    i as f64 / elapsed_s,
                    total_est - elapsed_s,
                    total_est,
                );
            }

            let word_hint = WordHint::from_guess_and_answer(guess, answer);

            // Get possible answers via dumb search
            dumb_search_words(&words, *guess, word_hint);
            i += 1;
        }
    }
    let total_elapsed = start.elapsed().as_secs_f64();
    println!(
        "finished {} in {:.3}s - {:.2} iter/s",
        num_trials,
        total_elapsed,
        num_trials as f64 / total_elapsed
    );

    println!("<- testing smart search ->");
    let start = Instant::now();
    let mut i = 0;
    for answer in &words {
        for guess in &words {
            if i >= num_trials {
                break;
            }
            if i % 10000 == 0 {
                let elapsed_s = start.elapsed().as_secs_f64();
                let total_est = (elapsed_s * num_trials as f64) / i as f64;
                println!(
                    "finished {} in {:.3}s - {:.2} iter/s - {:.3}s remaining - {:.3}s total",
                    i,
                    elapsed_s,
                    i as f64 / elapsed_s,
                    total_est - elapsed_s,
                    total_est,
                );
            }

            let word_hint = WordHint::from_guess_and_answer(guess, answer);

            // Get possible answers via smart search
            let query = clue_to_query(*guess, word_hint);
            smart_search.filter_words(&smart_search.eval_query(query.clone()));
            i += 1;
        }
    }
    let total_elapsed = start.elapsed().as_secs_f64();
    println!(
        "finished {} in {:.3}s - {:.2} iter/s",
        num_trials,
        total_elapsed,
        num_trials as f64 / total_elapsed
    );
}
