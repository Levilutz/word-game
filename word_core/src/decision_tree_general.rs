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
}

pub fn compute_decision_tree_aggressive(
    hints: &[Vec<u8>],
    possible_answers: HashSet<u16>,
    depth: u8,
    max_depth: u8,
    printer: Option<impl DebugPrinter>,
) -> Option<(TreeNode, f64)> {
    let prefix = "\t".repeat(depth as usize * 2);

    // Don't continue if we've already hit depth limit
    if depth == max_depth {
        if let Some(_) = printer {
            println!("{prefix}depth limit reached");
        }
        return None;
    }

    // Shortcut - if only one option left, just guess it
    if possible_answers.len() == 1 {
        let answer = possible_answers.into_iter().next().unwrap();
        if let Some(printer) = printer {
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
        if let Some(printer) = printer {
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

    None
}
