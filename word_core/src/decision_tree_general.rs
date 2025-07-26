use std::collections::{HashMap, HashSet};

/// A representation of a guess coming from one of either input list
pub enum GuessFrom {
    Guess(u16),
    Answer(u16),
}

pub struct TreeNode {
    pub should_guess: GuessFrom,
    pub next: HashMap<u8, TreeNode>,
}

pub trait DebugPrinter {
    fn fmt_guess(&self, guess_ind: u16) -> String;
    fn fmt_answer(&self, answer_ind: u16) -> String;
    fn fmt_hint(&self, hint_id: u8) -> String;
    fn fmt_clue(&self, hint_id: u8, guess_ind: u16) -> String;
    fn should_print_at_depth(&self, depth: u8) -> bool;
}

pub fn compute_decision_tree_aggressive(
    hints: &[Vec<u8>],
    possible_answers: HashSet<u16>,
    depth: u8,
    max_depth: u8,
    printer: Option<impl DebugPrinter>,
) -> Option<(TreeNode, f64)> {
    // Set the printer to `None` if we're past the configured depth
    let printer = match printer {
        Some(printer) if printer.should_print_at_depth(depth) => Some(printer),
        _ => None,
    };
    let prefix = "\t".repeat(depth as usize * 2);

    // Don't continue if we've already hit depth limit
    if depth == max_depth {
        if let Some(_) = &printer {
            println!("{prefix}depth limit reached");
        }
        return None;
    }

    // Shortcut - if only one option left, just guess it
    if possible_answers.len() == 1 {
        let answer = possible_answers.into_iter().next().unwrap();
        if let Some(printer) = &printer {
            println!(
                "{prefix}best guess is {} with est cost of {}",
                printer.fmt_answer(answer),
                1.0
            );
        }
        return Some((
            TreeNode {
                should_guess: GuessFrom::Answer(answer),
                next: HashMap::new(),
            },
            1.0,
        ));
    }

    // Shortcut - if only two options left, just guess one of them
    if possible_answers.len() == 2 {
        let mut possible_answers_iter = possible_answers.into_iter();
        let possible_answer_a = possible_answers_iter.next().unwrap();
        let possible_answer_b = possible_answers_iter.next().unwrap();
        if let Some(printer) = &printer {
            println!(
                "{prefix}best guess is {} with est cost of {}",
                printer.fmt_answer(possible_answer_a),
                1.5
            );
        }
        return Some((
            TreeNode {
                should_guess: GuessFrom::Answer(possible_answer_a),
                next: HashMap::from([(
                    hints[possible_answer_a as usize][possible_answer_b as usize],
                    TreeNode {
                        should_guess: GuessFrom::Answer(possible_answer_b),
                        next: HashMap::new(),
                    },
                )]),
            },
            1.5,
        ));
    }

    // Go through every possible guess and determine which is the best
    let mut best: Option<(TreeNode, f64)> = None;

    for guess_ind in 0..hints.len() as u16 {
        if let Some(printer) = &printer {
            println!(
                "{prefix}evaluating guess {} ({:.0}%",
                printer.fmt_guess(guess_ind),
                100.0 * guess_ind as f64 / hints.len() as f64
            );
        }

        // Check first if this guess is useless
        // If only 1 hint is possible for this guess, then it doesn't narrow down the
        // possible answer pool at all.
        let mut useless = true;
        for &hint in &hints[guess_ind as usize][1..] {
            if hint != hints[guess_ind as usize][0] {
                useless = false;
                break;
            }
        }
        if useless {
            if let Some(printer) = &printer {
                println!(
                    "{prefix}guess {} is useless, skipping",
                    printer.fmt_guess(guess_ind),
                );
            }
            continue;
        }
    }

    // Print the best guess and return
    if let Some(printer) = &printer {
        match &best {
            Some((tree_node, est_cost)) => println!(
                "{prefix}best guess is {} with est cost of {}",
                match tree_node.should_guess {
                    GuessFrom::Guess(guess_ind) => printer.fmt_guess(guess_ind),
                    GuessFrom::Answer(answer_ind) => printer.fmt_answer(answer_ind),
                },
                est_cost
            ),
            None => println!("{prefix}no guesses are guaranteed to solve within max_depth"),
        }
    }
    best
}
