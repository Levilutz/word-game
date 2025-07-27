use std::{collections::HashMap, time::Instant};

use serde::{Deserialize, Serialize};
use word_core::{
    decision_tree_general::{DebugPrinter, GuessFrom, TreeNode, compute_decision_tree_aggressive},
    hint::WordHint,
    load_words::load_guesses_and_answers_from_args,
    query_generation::{clue_possible, clue_to_query},
    word::Word,
    word_search::SearchableWords,
};

const WORD_SIZE: usize = 5;
const ALPHABET_SIZE: u8 = 26;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadableTreeNode {
    should_guess: Word<WORD_SIZE, ALPHABET_SIZE>,
    est_cost: f64,
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    next: HashMap<WordHint<WORD_SIZE>, ReadableTreeNode>,
}

impl ReadableTreeNode {
    fn from_generalized_tree_node(
        tree_node: &TreeNode,
        allowed_guesses: &[Word<WORD_SIZE, ALPHABET_SIZE>],
        possible_answers: &[Word<WORD_SIZE, ALPHABET_SIZE>],
    ) -> Self {
        Self {
            should_guess: match tree_node.should_guess {
                GuessFrom::Guess(guess_ind) => allowed_guesses[guess_ind as usize],
                GuessFrom::Answer(answer_ind) => possible_answers[answer_ind as usize],
            },
            est_cost: tree_node.est_cost,
            next: tree_node
                .next
                .iter()
                .map(|(hint_id, next_node)| {
                    (
                        WordHint::from_id(*hint_id),
                        ReadableTreeNode::from_generalized_tree_node(
                            &next_node,
                            allowed_guesses,
                            possible_answers,
                        ),
                    )
                })
                .collect(),
        }
    }
}

struct MyDebugPrinter<'a> {
    allowed_guesses: &'a [Word<WORD_SIZE, ALPHABET_SIZE>],
    possible_answers: &'a [Word<WORD_SIZE, ALPHABET_SIZE>],
    max_print_depth: Option<u8>,
    prefix: String,
}

impl<'a> DebugPrinter for MyDebugPrinter<'a> {
    fn fmt_guess(&self, guess_ind: u16) -> String {
        format!("{}", self.allowed_guesses[guess_ind as usize])
    }

    fn fmt_answer(&self, answer_ind: u16) -> String {
        format!("{}", self.possible_answers[answer_ind as usize])
    }

    fn fmt_hint(&self, hint_id: u8) -> String {
        format!("{}", WordHint::<WORD_SIZE>::from_id(hint_id))
    }

    fn fmt_clue(&self, hint_id: u8, guess_ind: u16) -> String {
        WordHint::<WORD_SIZE>::from_id(hint_id)
            .color_guess(&self.allowed_guesses[guess_ind as usize])
    }

    fn should_print_at_depth(&self, depth: u8) -> bool {
        match self.max_print_depth {
            Some(max_print_depth) => depth <= max_print_depth,
            None => true,
        }
    }

    fn with_prefix(&self, prefix: String) -> Self {
        Self {
            allowed_guesses: self.allowed_guesses,
            possible_answers: self.possible_answers,
            max_print_depth: self.max_print_depth,
            prefix: format!("{}{}", self.prefix, prefix),
        }
    }

    fn get_prefix(&self) -> &str {
        &self.prefix
    }
}

fn main() {
    let (allowed_guesses, possible_answers) = load_guesses_and_answers_from_args(true);

    println!("precomputing all hints...");
    let start = Instant::now();
    let mut all_hints: Vec<Vec<u8>> = Vec::with_capacity(allowed_guesses.len());
    let searchable_answers = SearchableWords::build(possible_answers.clone());
    for guess in &allowed_guesses {
        let mut hints_for_guess = vec![0; possible_answers.len()];
        for hint in WordHint::all_possible() {
            if !clue_possible(*guess, hint) {
                continue;
            }
            let answers_giving_this_hint_mask =
                searchable_answers.eval_query(clue_to_query(*guess, hint));
            let hint_id = hint.hint_id();
            for answer_ind in answers_giving_this_hint_mask.true_inds() {
                hints_for_guess[answer_ind] = hint_id;
            }
        }
        all_hints.push(hints_for_guess);
    }
    let total_elapsed = start.elapsed().as_secs_f64();
    println!("done in {:.3}s", total_elapsed);

    println!("generating decision tree...");
    let start = Instant::now();
    let decision_tree = compute_decision_tree_aggressive(
        &all_hints,
        (0..possible_answers.len() as u16).into_iter().collect(),
        0,
        6,
        3.0402,
        // None::<&MyDebugPrinter>,
        Some(&MyDebugPrinter {
            allowed_guesses: &allowed_guesses,
            possible_answers: &possible_answers,
            max_print_depth: Some(0),
            prefix: "".to_string(),
        }),
    )
    .expect("failed to compute top-level result");
    let readable_decision_tree = ReadableTreeNode::from_generalized_tree_node(
        &decision_tree,
        &allowed_guesses,
        &possible_answers,
    );
    let total_elapsed = start.elapsed().as_secs_f64();
    println!(
        "{}",
        serde_json::to_string_pretty(&readable_decision_tree).unwrap()
    );
    println!("est cost: {}", decision_tree.est_cost);
    println!("done in {:.3}s", total_elapsed);
}
