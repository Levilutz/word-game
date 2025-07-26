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
    fn with_prefix(&self, prefix: String) -> Self;
    fn get_prefix(&self) -> &str;
}

pub fn compute_decision_tree_aggressive(
    hints: &[Vec<u8>],
    possible_answers: HashSet<u16>,
    depth: u8,
    max_depth: u8,
    max_cost: f64,
    printer: Option<&impl DebugPrinter>,
) -> Option<(TreeNode, f64)> {
    // Set the printer to `None` if we're past the configured depth
    let printer = match printer {
        Some(printer) if printer.should_print_at_depth(depth) => Some(printer),
        _ => None,
    };

    if let Some(printer) = printer {
        println!(
            "{}must compute {} possible answers with max cost of {}",
            printer.get_prefix(),
            possible_answers.len(),
            max_cost
        );
    }

    // Don't continue if we've already hit depth limit
    if depth == max_depth {
        if let Some(printer) = printer {
            println!("{}depth limit reached", printer.get_prefix());
        }
        return None;
    }

    // Don't continue if we've already hit cost limit
    if max_cost < 1.0 {
        if let Some(printer) = printer {
            println!("{}cost limit exceeded", printer.get_prefix());
        }
        return None;
    }

    // Shortcut - if only one option left, just guess it
    if possible_answers.len() == 1 {
        let answer = possible_answers.into_iter().next().unwrap();
        if let Some(printer) = printer {
            println!(
                "{}best guess is {} with est cost of {} (certain)",
                printer.get_prefix(),
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

    // Don't continue if we aren't guaranteed to avoid depth limit
    if depth == max_depth - 1 {
        if let Some(printer) = printer {
            println!("{}depth limit cannot be avoided", printer.get_prefix());
        }
        return None;
    }

    // Don't continue if we aren't guaranteed to avoid cost limit
    if max_cost < 1.5 {
        if let Some(printer) = printer {
            println!("{}cost limit cannot be avoided", printer.get_prefix());
        }
        return None;
    }

    // Shortcut - if only two options left, just guess one of them
    if possible_answers.len() == 2 {
        let mut possible_answers_iter = possible_answers.into_iter();
        let possible_answer_a = possible_answers_iter.next().unwrap();
        let possible_answer_b = possible_answers_iter.next().unwrap();
        if let Some(printer) = printer {
            println!(
                "{}best guess is {} with est cost of {}",
                printer.get_prefix(),
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
    let mut guess_max_est_cost = max_cost;

    'guess_loop: for guess_ind in 0..hints.len() as u16 {
        let guess_hints = &hints[guess_ind as usize];

        let printer_owned = printer
            .map(|printer| printer.with_prefix(format!("{} > ", printer.fmt_guess(guess_ind))));
        let printer = printer_owned.as_ref();
        if let Some(printer) = printer {
            println!(
                "{}evaluating guess {} - {:.0}% complete",
                printer.get_prefix(),
                printer.fmt_guess(guess_ind),
                100.0 * guess_ind as f64 / hints.len() as f64
            );
        }

        // Check first if this guess is useless
        // If only 1 hint is possible for this guess, then it doesn't narrow down the
        // possible answer pool at all.
        let mut useless = true;
        let mut possible_answers_iter = possible_answers.iter();
        let some_possible_answer = *possible_answers_iter.next().unwrap() as usize;
        let some_possible_guess = guess_hints[some_possible_answer];
        for &possible_answer in possible_answers_iter {
            if guess_hints[possible_answer as usize] != some_possible_guess {
                useless = false;
                break;
            }
        }
        if useless {
            if let Some(printer) = printer {
                println!(
                    "{}guess {} is useless, skipping",
                    printer.get_prefix(),
                    printer.fmt_guess(guess_ind),
                );
            }
            continue;
        }

        // Build map from possible hint to possible answers if we were to receive that hint
        let answers_by_hint: HashMap<u8, HashSet<u16>> =
            possible_answers
                .iter()
                .fold(HashMap::new(), |mut map, &answer_ind| {
                    let answers_for_hint = map.entry(guess_hints[answer_ind as usize]).or_default();
                    answers_for_hint.insert(answer_ind as u16);
                    map
                });

        // Convert into list of tuples, ordered by number of answers descending
        let mut hints_answers: Vec<(u8, HashSet<u16>)> = answers_by_hint.into_iter().collect();
        hints_answers.sort_unstable_by(|(_, answers_a), (_, answers_b)| {
            answers_a.len().cmp(&answers_b.len())
        });

        if let Some(printer) = printer {
            println!(
                "{}considering {} possible hints",
                printer.get_prefix(),
                hints_answers.len(),
            );
        }

        // Add up estimated cost across all possibilities, weighted by likelihood
        let mut guess_est_cost = 1.0;
        let mut guess_next_nodes: HashMap<u8, TreeNode> = HashMap::new();

        for (hint, hint_possible_answers) in hints_answers.into_iter() {
            let hint_num_possible_answers = hint_possible_answers.len();
            let hint_likelihood = hint_num_possible_answers as f64 / possible_answers.len() as f64;

            let printer_owned = printer.map(|printer| {
                printer.with_prefix(format!("{} > ", printer.fmt_clue(hint, guess_ind)))
            });
            let printer = printer_owned.as_ref();

            if let Some(printer) = printer {
                println!(
                    "{}evaluating clue {} with {}/{} possible answers - {:.2}% chance",
                    printer.get_prefix(),
                    printer.fmt_clue(hint, guess_ind),
                    hint_num_possible_answers,
                    possible_answers.len(),
                    100.0 * hint_likelihood,
                )
            }

            // If we happened to guess correctly, there is no additional cost
            if hint == 0 {
                continue;
            }

            // Find the child node for this clue
            let remaining_est_cost_budget = guess_max_est_cost - guess_est_cost;
            let child_max_est_cost = remaining_est_cost_budget / hint_likelihood;
            if let Some((child_tree_node, child_est_cost)) = compute_decision_tree_aggressive(
                hints,
                hint_possible_answers,
                depth + 1,
                max_depth,
                child_max_est_cost,
                printer,
            ) {
                guess_est_cost += child_est_cost * hint_likelihood;
                guess_next_nodes.insert(hint, child_tree_node);
            } else {
                if let Some(printer) = printer {
                    println!(
                        "{}guess {} cannot guarantee an answer within depth limit",
                        printer.get_prefix(),
                        printer.fmt_guess(guess_ind),
                    );
                }
                continue 'guess_loop;
            }
            if guess_est_cost >= guess_max_est_cost {
                if let Some(printer) = printer {
                    println!(
                        "{}guess {} est cost of {:.3} already exceeds max of {:.3}",
                        printer.get_prefix(),
                        printer.fmt_guess(guess_ind),
                        guess_est_cost,
                        guess_max_est_cost,
                    );
                }
                continue 'guess_loop;
            }
        }

        // Evaluate if this guess beats the current best guess
        let this_guess_is_new_best = match best {
            Some((_, best_guess_est_cost)) if best_guess_est_cost <= guess_est_cost => false,
            _ => true,
        };
        if let Some(printer) = printer {
            println!(
                "{}guess {} has est cost {} - {}",
                printer.get_prefix(),
                printer.fmt_guess(guess_ind),
                guess_est_cost,
                if this_guess_is_new_best {
                    "new best"
                } else {
                    "rejecting"
                }
            );
        }
        if this_guess_is_new_best {
            best = Some((
                TreeNode {
                    should_guess: GuessFrom::Guess(guess_ind),
                    next: guess_next_nodes,
                },
                guess_est_cost,
            ));
            guess_max_est_cost = guess_est_cost;
        }
    }

    // Print the best guess and return
    if let Some(printer) = printer {
        match &best {
            Some((tree_node, est_cost)) => println!(
                "{}best guess is {} with est cost of {}",
                printer.get_prefix(),
                match tree_node.should_guess {
                    GuessFrom::Guess(guess_ind) => printer.fmt_guess(guess_ind),
                    GuessFrom::Answer(answer_ind) => printer.fmt_answer(answer_ind),
                },
                est_cost
            ),
            None => println!(
                "{}no guesses are guaranteed to solve within depth limit",
                printer.get_prefix(),
            ),
        }
    }
    best
}
