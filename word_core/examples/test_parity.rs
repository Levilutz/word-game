use std::{collections::HashSet, env::args, time::Instant};

use word_core::{
    dumb_word_search::dumb_search_words, hint::WordHint, load_words::load_words,
    query_generation::clue_to_query, word::Word, word_search::SearchableWords,
};

const WORD_SIZE: usize = 5;
const ALPHABET_SIZE: u8 = 26;

fn main() {
    let words = load_words(&args().nth(1).expect("Must supply word list as first arg"));
    let num_trials = words.len() * words.len();

    println!("loaded {} words -> {} test cases", words.len(), num_trials);

    let smart_search: SearchableWords<WORD_SIZE, 26> = SearchableWords::build(words.clone());

    let start = Instant::now();
    let mut i = 0;
    for answer in &words {
        for guess in &words {
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
            let possible_answers_dumb = dumb_search_words(&words, *guess, word_hint);

            // Get possible answers via smart search
            let query = clue_to_query(*guess, word_hint);
            let possible_answers_smart =
                smart_search.filter_words(&smart_search.eval_query(query.clone()));

            // Compare results
            let dumb_set: HashSet<Word<WORD_SIZE, ALPHABET_SIZE>> =
                possible_answers_dumb.into_iter().collect();
            let smart_set: HashSet<Word<WORD_SIZE, ALPHABET_SIZE>> =
                possible_answers_smart.into_iter().collect();

            assert_eq!(
                dumb_set,
                smart_set,
                "mismatch!
guess:  {}
hints:  {}
answer: {}
<- dumb output ->
{}
<- smart output ->
{}
<- smart query ->
{:#?}
<- end ->",
                word_hint.color_guess(guess),
                word_hint,
                *answer,
                dumb_set
                    .iter()
                    .map(|word| format!("{}", word))
                    .collect::<Vec<String>>()
                    .join("\n"),
                smart_set
                    .iter()
                    .map(|word| format!("{}", word))
                    .collect::<Vec<String>>()
                    .join("\n"),
                query,
            );
            i += 1;
        }
    }
}
