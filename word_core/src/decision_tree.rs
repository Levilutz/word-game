use std::{collections::HashMap, f64::INFINITY};

use crate::{
    column::Column, hint::WordHint, query_generation::clue_to_query, word::Word,
    word_search::SearchableWords,
};

/// A node in the output decision tree
#[derive(Debug, Clone)]
pub enum TreeNode<const WORD_SIZE: usize> {
    Answer {
        answer: Word<WORD_SIZE>,
    },
    Decision {
        should_enter: Word<WORD_SIZE>,
        next: HashMap<WordHint<WORD_SIZE>, TreeNode<WORD_SIZE>>,
    },
}

pub fn compute_node_aggressive<const WORD_SIZE: usize, const ALPHABET_SIZE: u8>(
    allowed_guesses: &[Word<WORD_SIZE>],
    possible_answers: SearchableWords<WORD_SIZE, ALPHABET_SIZE>,
    depth: u64,
    do_print: bool,
) -> (TreeNode<WORD_SIZE>, f64) {
    let prefix = (0..depth).map(|_| "\t").collect::<Vec<&str>>().join("");
    if possible_answers.len() == 1 {
        return (
            TreeNode::Answer {
                answer: possible_answers.filter_words(&Column::from_true(1))[0],
            },
            1.0,
        );
    }
    let mut best: Option<(
        Word<WORD_SIZE>,
        HashMap<WordHint<WORD_SIZE>, TreeNode<WORD_SIZE>>,
        f64,
    )> = None;
    for guess in allowed_guesses {
        if !do_print && depth == 0 {
            println!("evaluating top level guess {}", guess);
        }
        let child_allowed_guesses: Vec<Word<WORD_SIZE>> = allowed_guesses
            .iter()
            .filter(|allowed_guess| *allowed_guess != guess)
            .cloned()
            .collect();
        let mut guess_decision_tree: HashMap<WordHint<WORD_SIZE>, TreeNode<WORD_SIZE>> =
            HashMap::new();
        let mut guess_avg_perf = 0.0;
        for word_hint in WordHint::all_possible() {
            let mask = possible_answers.eval_query(clue_to_query(*guess, word_hint));
            let num_answers_giving_this_hint = mask.count_true();
            if num_answers_giving_this_hint == 0 {
                continue;
            }
            if num_answers_giving_this_hint == possible_answers.len() as u64 {
                // This guess doesn't filter at all and is worthless, don't consider
                guess_avg_perf = INFINITY;
                break;
            }
            if do_print {
                println!(
                    "{}{} could come from {} answers - {}",
                    prefix,
                    word_hint.color_guess(guess),
                    num_answers_giving_this_hint,
                    possible_answers
                        .filter_words(&mask)
                        .iter()
                        .map(|word| format!("{}", word))
                        .collect::<Vec<String>>()
                        .join(", ")
                );
            }
            let (child_node, child_est_guesses) = match num_answers_giving_this_hint {
                1 => (
                    TreeNode::Answer {
                        answer: possible_answers.filter_words(&mask)[0],
                    },
                    1.0,
                ),
                _ => compute_node_aggressive(
                    &child_allowed_guesses,
                    possible_answers.filter(&mask),
                    depth + 1,
                    do_print,
                ),
            };
            guess_avg_perf += (child_est_guesses + 1.0) * num_answers_giving_this_hint as f64
                / possible_answers.len() as f64;
            guess_decision_tree.insert(word_hint, child_node);
        }
        if guess_avg_perf == INFINITY {
            continue;
        }
        if do_print {
            println!(
                "{}> guess {} has avg perf of {}",
                prefix, guess, guess_avg_perf
            );
        }
        match best {
            Some((_, _, best_guess_avg_perf)) if best_guess_avg_perf <= guess_avg_perf => {
                continue;
            }
            _ => best = Some((*guess, guess_decision_tree, guess_avg_perf)),
        }
    }
    let (best_guess, best_guess_decision_tree, best_guess_avg_perf) = best.unwrap();
    if do_print {
        println!(
            "{}>> recommending {} with avg perf of {}",
            prefix, best_guess, best_guess_avg_perf
        );
    }
    (
        TreeNode::Decision {
            should_enter: best_guess,
            next: best_guess_decision_tree,
        },
        best_guess_avg_perf,
    )
}
