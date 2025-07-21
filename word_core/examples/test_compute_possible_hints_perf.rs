use std::{
    collections::{HashMap, HashSet},
    env::args,
    fs,
    time::Instant,
};

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
    let mut possible_hints_per_guess_simple: HashMap<
        Word<WORD_SIZE, ALPHABET_SIZE>,
        HashSet<WordHint<WORD_SIZE>>,
    > = HashMap::new();
    for guess in &allowed_guesses {
        let mut possible_hints: HashSet<WordHint<WORD_SIZE>> = HashSet::new();
        for answer in &possible_answers {
            let hint = WordHint::from_guess_and_answer(&guess, answer);
            possible_hints.insert(hint);
        }
        possible_hints_per_guess_simple.insert(*guess, possible_hints);
    }
    let total_elapsed = start.elapsed().as_secs_f64();
    println!("finished in {:.3}s", total_elapsed);

    println!("<- testing query engine scan ->");
    let start = Instant::now();
    let mut possible_hints_per_guess_query_engine: HashMap<
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
        possible_hints_per_guess_query_engine.insert(*guess, possible_hints);
    }
    let total_elapsed = start.elapsed().as_secs_f64();
    println!("finished in {:.3}s", total_elapsed);

    assert_eq!(
        possible_hints_per_guess_simple
            .keys()
            .into_iter()
            .cloned()
            .collect::<HashSet<Word<WORD_SIZE, ALPHABET_SIZE>>>(),
        allowed_guesses
            .iter()
            .cloned()
            .collect::<HashSet<Word<WORD_SIZE, ALPHABET_SIZE>>>()
    );
    assert_eq!(
        possible_hints_per_guess_query_engine
            .keys()
            .into_iter()
            .cloned()
            .collect::<HashSet<Word<WORD_SIZE, ALPHABET_SIZE>>>(),
        allowed_guesses
            .iter()
            .cloned()
            .collect::<HashSet<Word<WORD_SIZE, ALPHABET_SIZE>>>()
    );
    for guess in &allowed_guesses {
        let possible_hints_simple = possible_hints_per_guess_simple.get(guess).unwrap();
        let possible_hints_query_engine = possible_hints_per_guess_query_engine.get(guess).unwrap();

        let a_not_b: HashSet<&WordHint<WORD_SIZE>> = possible_hints_simple
            .difference(possible_hints_query_engine)
            .collect();
        let b_not_a: HashSet<&WordHint<WORD_SIZE>> = possible_hints_query_engine
            .difference(possible_hints_simple)
            .collect();

        if a_not_b.len() > 0 || b_not_a.len() > 0 {
            println!("Two scans got different results for {}", guess);
            println!("Hints discovered by simple scan but not query engine:");
            for hint in a_not_b {
                println!("{}", hint);
            }
            println!("Hints discovered by query engine but not simple scan:");
            for hint in b_not_a {
                println!("{}", hint);
            }
            println!("");
        }
    }
    println!("both scans gave equivalent results")
}
