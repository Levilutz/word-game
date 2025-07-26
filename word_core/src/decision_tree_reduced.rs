use std::collections::{HashMap, HashSet};

pub struct TreeNode {
    pub should_guess: u16,
    pub next: HashMap<u8, TreeNode>,
}

pub struct PrintConfig {
    pub fmt_guess: fn(u16) -> String,
    pub fmt_answer: fn(u16) -> String,
    pub fmt_hint: fn(u8) -> String,
    pub fmt_hint_guess: fn(u8, u16) -> String,
}

pub fn compute_decision_tree_aggressive(
    hints: Vec<Vec<u8>>,
    possible_answers: HashSet<u16>,
    depth: u8,
    max_depth: u8,
    print_config: Option<PrintConfig>,
) -> Option<(TreeNode, f64)> {
    let prefix = "\t".repeat(depth as usize * 2);

    // Don't continue if we've already hit depth limit
    if depth == max_depth {
        if let Some(_) = print_config {
            println!("{prefix}depth limit reached");
        }
        return None;
    }

    // Shortcut - if only one option left, just guess it
    if possible_answers.len() == 1 {
        let answer = possible_answers.into_iter().next().unwrap();
        if let Some(print_config) = print_config {
            println!(
                "{prefix}best guess is {} with est cost of {}",
                (print_config.fmt_answer)(answer),
                1.0
            );
        }
        return Some((
            TreeNode {
                should_guess: answer,
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
        if let Some(print_config) = print_config {
            println!(
                "{prefix}best guess is {} with est cost of {}",
                (print_config.fmt_answer)(possible_answer_a),
                1.5
            );
        }
        return Some((
            TreeNode {
                should_guess: possible_answer_a,
                next: HashMap::from([(
                    hints[possible_answer_a as usize][possible_answer_b as usize],
                    TreeNode {
                        should_guess: possible_answer_b,
                        next: HashMap::new(),
                    },
                )]),
            },
            1.5,
        ));
    }

    None
}
