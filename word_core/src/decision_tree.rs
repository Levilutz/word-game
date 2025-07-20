use std::{
    collections::{HashMap, HashSet},
    f64::INFINITY,
};

use serde::{Deserialize, Serialize};

use crate::{
    column::Column, hint::WordHint, query_generation::clue_to_query, word::Word,
    word_search::SearchableWords,
};

/// Must use const alphabet size to satisfy serde traits constrained to 26
const ALPHABET_SIZE: u8 = 26;

/// A node in the output decision tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeNode<const WORD_SIZE: usize> {
    should_enter: Word<WORD_SIZE, ALPHABET_SIZE>,
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    next: HashMap<WordHint<WORD_SIZE>, TreeNode<WORD_SIZE>>,
}

pub fn compute_node_aggressive<const WORD_SIZE: usize>(
    allowed_guesses: &[Word<WORD_SIZE, ALPHABET_SIZE>],
    possible_answers: SearchableWords<WORD_SIZE, ALPHABET_SIZE>,
    depth: u64,
    max_depth: u64,
    do_print: bool,
) -> Option<(TreeNode<WORD_SIZE>, f64)> {
    let prefix = (0..depth * 2).map(|_| "\t").collect::<Vec<&str>>().join("");
    if depth == max_depth {
        if do_print {
            println!("{}depth limit reached", prefix);
        }
        return None;
    }
    // Shortcut - if only one option left, just guess it
    if possible_answers.len() == 1 {
        let answer = possible_answers.filter_words(&Column::from_true(1))[0];
        if do_print {
            println!(
                "{}best guess is \x1b[1m{}\x1b[0m with est cost of {}",
                prefix, answer, 1.0
            );
        }
        return Some((
            TreeNode {
                should_enter: answer,
                next: HashMap::new(),
            },
            1.0,
        ));
    }
    // Shortcut - if only two options left, just guess one of them
    if possible_answers.len() == 2 {
        let possible_answer_words = possible_answers.filter_words(&Column::from_true(2));
        let possible_answer_a = possible_answer_words[0];
        let possible_answer_b = possible_answer_words[1];
        if do_print {
            println!(
                "{}best guess is \x1b[1m{}\x1b[0m with est cost of {}",
                prefix, possible_answer_a, 1.5
            );
        }
        return Some((
            TreeNode {
                should_enter: possible_answer_a,
                next: HashMap::from([(
                    WordHint::from_guess_and_answer(&possible_answer_a, &possible_answer_b),
                    TreeNode {
                        should_enter: possible_answer_b,
                        next: HashMap::new(),
                    },
                )]),
            },
            1.5,
        ));
    }
    let mut best: Option<(
        Word<WORD_SIZE, ALPHABET_SIZE>,
        HashMap<WordHint<WORD_SIZE>, TreeNode<WORD_SIZE>>,
        f64,
    )> = None;
    for (guess_ind, guess) in allowed_guesses.iter().enumerate() {
        if !do_print && depth <= 0 {
            println!(
                "evaluating level {} guess \x1b[1m{}\x1b[0m - {:.0}%",
                depth,
                guess,
                100.0 * guess_ind as f64 / allowed_guesses.len() as f64
            );
        }
        if do_print {
            println!("{}evaluating guess \x1b[1m{}\x1b[0m", prefix, guess)
        }

        // Evaluate if this guess is useless before scanning all possible hints
        // Pull a random possible answer, generate a random possible hint, and see if
        // that hint covers every answer.
        let mask = possible_answers.eval_query(clue_to_query(
            *guess,
            WordHint::from_guess_and_answer(guess, &possible_answers.words()[0]),
        ));
        if mask.count_true() == possible_answers.len() as u64 {
            if do_print {
                println!(
                    "{}guess \x1b[1m{}\x1b[0m is useless, skipping",
                    prefix, guess
                );
            }
            continue;
        }

        let child_allowed_guesses: Vec<Word<WORD_SIZE, ALPHABET_SIZE>> = allowed_guesses
            .iter()
            .filter(|allowed_guess| *allowed_guess != guess)
            .cloned()
            .collect();
        let mut guess_decision_tree: HashMap<WordHint<WORD_SIZE>, TreeNode<WORD_SIZE>> =
            HashMap::new();
        let mut guess_est_cost = 1.0;
        let possible_hints: Vec<WordHint<WORD_SIZE>> = possible_answers
            .words()
            .iter()
            .map(|answer| WordHint::from_guess_and_answer(guess, answer))
            .collect::<HashSet<WordHint<WORD_SIZE>>>()
            .into_iter()
            .collect();
        let num_possible_hints = possible_hints.len();
        for (word_hint_ind, word_hint) in possible_hints.into_iter().enumerate() {
            if !do_print && depth < 0 {
                println!(
                    "evaluating level {} clue {}\x1b[0m - {:.0}%",
                    depth,
                    word_hint.color_guess(guess),
                    100.0 * word_hint_ind as f64 / num_possible_hints as f64
                );
            }
            let mask = possible_answers.eval_query(clue_to_query(*guess, word_hint));
            let num_answers_giving_this_hint = mask.count_true();
            if num_answers_giving_this_hint == 0 {
                continue;
            }
            if do_print {
                println!(
                    "{}\tclue {} would indicate {} possible answer{} - {}",
                    prefix,
                    word_hint.color_guess(guess),
                    num_answers_giving_this_hint,
                    if num_answers_giving_this_hint > 1 {
                        "s"
                    } else {
                        ""
                    },
                    possible_answers
                        .filter_words(&mask)
                        .iter()
                        .map(|word| format!("{}", word))
                        .collect::<Vec<String>>()
                        .join(", ")
                );
            }
            if word_hint.all_correct() {
                // We happened to guess correctly, there is no additional cost
                continue;
            }
            if depth == max_depth - 1 {
                // We've used all our allowed guesses, don't consider this path
                if do_print {
                    println!("{}guess \x1b[1m{}\x1b[0m is too expensive", prefix, guess);
                }
                guess_est_cost = INFINITY;
                break;
            }
            if let Some((child_node, child_est_addl_cost)) = compute_node_aggressive(
                &child_allowed_guesses,
                possible_answers.filter(&mask),
                depth + 1,
                max_depth,
                do_print,
            ) {
                guess_est_cost += child_est_addl_cost * num_answers_giving_this_hint as f64
                    / possible_answers.len() as f64;
                guess_decision_tree.insert(word_hint, child_node);
            } else {
                if do_print {
                    println!(
                        "{}guess \x1b[1m{}\x1b[0m cannot guarantee an answer within depth limit",
                        prefix, guess
                    );
                }
                guess_est_cost = INFINITY;
                break;
            }
        }
        if guess_est_cost == INFINITY {
            continue;
        }
        let this_guess_is_new_best = match best {
            Some((_, _, best_guess_est_cost)) if best_guess_est_cost <= guess_est_cost => false,
            _ => true,
        };
        if do_print {
            println!(
                "{}guess \x1b[1m{}\x1b[0m has est cost {} - {}",
                prefix,
                guess,
                guess_est_cost,
                if this_guess_is_new_best {
                    "\x1b[1mnew best\x1b[0m"
                } else {
                    "rejecting"
                }
            );
        }
        if this_guess_is_new_best {
            best = Some((*guess, guess_decision_tree, guess_est_cost))
        }
    }
    let (best_guess, best_guess_decision_tree, best_guess_est_cost) = best?;
    if do_print {
        println!(
            "{}best guess is \x1b[1m{}\x1b[0m with est cost of {}",
            prefix, best_guess, best_guess_est_cost
        );
    }
    Some((
        TreeNode {
            should_enter: best_guess,
            next: best_guess_decision_tree,
        },
        best_guess_est_cost,
    ))
}
