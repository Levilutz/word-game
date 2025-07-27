use std::collections::{HashMap, HashSet};

/// A representation of a guess coming from one of either input list
pub enum GuessFrom {
    Guess(u16),
    Answer(u16),
}

pub struct TreeNode {
    pub should_guess: GuessFrom,
    pub est_cost: f64,
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
) -> Option<TreeNode> {
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
        return Some(TreeNode {
            should_guess: GuessFrom::Answer(answer),
            est_cost: 1.0,
            next: HashMap::new(),
        });
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
        return Some(TreeNode {
            should_guess: GuessFrom::Answer(possible_answer_a),
            est_cost: 1.5,
            next: HashMap::from([(
                hints[possible_answer_a as usize][possible_answer_b as usize],
                TreeNode {
                    should_guess: GuessFrom::Answer(possible_answer_b),
                    est_cost: 1.0,
                    next: HashMap::new(),
                },
            )]),
        });
    }

    // Go through every possible guess and determine which is the best
    let mut best: Option<TreeNode> = None;
    let mut guess_max_est_cost = max_cost;

    // We can filter more aggressively if we happen to see the best possible guess sooner
    // The best possible guess _tends_ to have an "even" distribution of hints. i.e. no
    // single hint downstream of that guess gives a huge of the answers.
    // To improve how early we see the best possible guess, we can thus order guesses by
    // the frequency of their most common subsequent hint.
    // We can also take this as an opportunity to filter out "useless" guesses, as they
    // will have all answers under a single hint.
    let mut guess_order: Vec<(u16, usize)> = (0..hints.len())
        .map(|guess_ind| {
            let guess_hints = &hints[guess_ind];
            let num_answers_by_hint: HashMap<u8, usize> =
                possible_answers
                    .iter()
                    .fold(HashMap::new(), |mut map, &answer_ind| {
                        let hint = guess_hints[answer_ind as usize];
                        *map.entry(hint).or_insert(0) += 1;
                        map
                    });
            let most_answers_for_any_hint = *num_answers_by_hint.values().max().unwrap();
            (guess_ind as u16, most_answers_for_any_hint)
        })
        .filter(|(_, most_answers_for_any_hint)| {
            *most_answers_for_any_hint != possible_answers.len()
        })
        .collect();
    guess_order.sort_unstable_by(
        |(_, a_most_answers_possible), (_, b_most_answers_possible)| {
            a_most_answers_possible.cmp(b_most_answers_possible)
        },
    );
    let guess_order: Vec<u16> = guess_order
        .into_iter()
        .map(|(guess_ind, _)| guess_ind)
        .collect();

    if let Some(printer) = printer {
        println!(
            "{}first guesses will be {}",
            printer.get_prefix(),
            guess_order[..5]
                .iter()
                .map(|guess_ind| printer.fmt_guess(*guess_ind))
                .collect::<Vec<String>>()
                .join(", ")
        );
    }

    'guess_loop: for guess_ind in guess_order {
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

        if let Some(printer) = printer {
            let distribution: HashMap<usize, usize> =
                answers_by_hint
                    .iter()
                    .fold(HashMap::new(), |mut map, (_, answers)| {
                        *map.entry(answers.len()).or_insert(0) += 1;
                        map
                    });
            let mode_val = *distribution.values().max().unwrap();
            let max_key = *distribution.keys().max().unwrap();
            let distribution_flat: Vec<usize> = (0..=max_key)
                .map(|ind| distribution.get(&ind).cloned().unwrap_or(0))
                .collect();
            let heights = [" ", "⡀", "⣀", "⣄", "⣤", "⣦", "⣶", "⣷", "⣿"];
            let distribution_fmt: Vec<&str> = distribution_flat
                .into_iter()
                .map(|n_hints| heights[(8 * n_hints + mode_val - 1) / mode_val])
                .collect();
            println!(
                "{}distribution: {}<{}",
                printer.get_prefix(),
                distribution_fmt.join(""),
                max_key
            );
        }

        let correct_hint_present = answers_by_hint.contains_key(&0);

        // Convert into list of tuples, ordered by number of answers descending
        let mut hints_answers: Vec<(u8, HashSet<u16>)> = answers_by_hint.into_iter().collect();
        hints_answers.sort_unstable_by(|(_, answers_a), (_, answers_b)| {
            answers_a.len().cmp(&answers_b.len())
        });

        // Set lower bound on estimated cost given what we know so far, so we can prune earlier
        // Lower bound cost for a single hint is `(2p - 1)` / p (p is # of possible answers for that hint)
        // or 0 if the hint is all-correct.
        // This is based on the best-case scenario of guessing the correct answer next with 1/p odds, or
        // knowing exactly which of the remaining is the answer with (p-1)/p odds.
        // The lower bound for the whole set of hints then simplifies to:
        // > `2 - h / p` if correct hint not present
        // > `2 - (h + 1) / p` if correct hint present
        // h = total # of hints, p = total # of possible answers
        // h is the total number of hints and p is the total number of possible answers.
        // We then must add 1 more to accommodate the hint we just made above=
        let est_cost_lower_bound = if correct_hint_present {
            3.0 - ((hints_answers.len() as f64 + 1.0) / possible_answers.len() as f64)
        } else {
            3.0 - (hints_answers.len() as f64 / possible_answers.len() as f64)
        };

        if est_cost_lower_bound >= guess_max_est_cost {
            if let Some(printer) = printer {
                println!(
                    "{}est cost lower bound of {:.3} already exceeds max of {:.3}",
                    printer.get_prefix(),
                    est_cost_lower_bound,
                    guess_max_est_cost,
                );
            }
            continue;
        }

        if let Some(printer) = printer {
            println!(
                "{}considering {} possible hints - lower bound est_cost of {:.3}",
                printer.get_prefix(),
                hints_answers.len(),
                est_cost_lower_bound,
            );
        }

        // Initialize guess with lower bound est cost
        let mut guess = TreeNode {
            should_guess: GuessFrom::Guess(guess_ind),
            est_cost: est_cost_lower_bound,
            next: HashMap::new(),
        };

        // Reorder hints to be ascending on number of possible answers, with 1s & 2s in the back
        let first_ind_at_least_3 = hints_answers
            .iter()
            .enumerate()
            .find(|(_, (_, answers))| answers.len() >= 3)
            .map(|(ind, _)| ind);
        if let Some(split_ind) = first_ind_at_least_3 {
            hints_answers.rotate_left(split_ind);
        }

        // Add up estimated cost across all possibilities, weighted by likelihood
        for (hint, hint_possible_answers) in hints_answers.into_iter() {
            // If we happened to guess correctly, there is no additional cost
            if hint == 0 {
                continue;
            }

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

            // Reconstruct the lower bound we made earlier, for this specific hint
            let child_est_cost_lower_bound =
                (2.0 * hint_num_possible_answers as f64 - 1.0) / possible_answers.len() as f64;

            // Compute how much "budget" we have at our level for total est cost
            let remaining_est_cost_budget =
                guess_max_est_cost - guess.est_cost + child_est_cost_lower_bound;

            // Compute the child's est cost based on hint probability
            let child_max_est_cost = remaining_est_cost_budget / hint_likelihood;

            // Find the child node for this clue
            if let Some(child_tree_node) = compute_decision_tree_aggressive(
                hints,
                hint_possible_answers,
                depth + 1,
                max_depth,
                child_max_est_cost,
                printer,
            ) {
                let child_est_cost_scaled = child_tree_node.est_cost * hint_likelihood;
                if (child_est_cost_scaled - child_est_cost_lower_bound).abs() > 1e-6 {
                    guess.est_cost += child_est_cost_scaled - child_est_cost_lower_bound;
                }
                guess.next.insert(hint, child_tree_node);
            } else {
                if let Some(printer) = printer {
                    println!(
                        "{}guess {} cannot guarantee an answer within constraints",
                        printer.get_prefix(),
                        printer.fmt_guess(guess_ind),
                    );
                }
                continue 'guess_loop;
            }
            if guess.est_cost >= guess_max_est_cost {
                if let Some(printer) = printer {
                    println!(
                        "{}guess {} est cost of {:.3} already exceeds max of {:.3}",
                        printer.get_prefix(),
                        printer.fmt_guess(guess_ind),
                        guess.est_cost,
                        guess_max_est_cost,
                    );
                }
                continue 'guess_loop;
            }
        }

        // Evaluate if this guess beats the current best guess
        let this_guess_is_new_best = match &best {
            Some(best_guess) if best_guess.est_cost <= guess.est_cost => false,
            _ => true,
        };
        if let Some(printer) = printer {
            println!(
                "{}guess {} has est cost {} - {}",
                printer.get_prefix(),
                printer.fmt_guess(guess_ind),
                guess.est_cost,
                if this_guess_is_new_best {
                    "new best"
                } else {
                    "rejecting"
                }
            );
        }
        if this_guess_is_new_best {
            guess_max_est_cost = guess.est_cost;
            best = Some(guess);
        }
    }

    // Print the best guess and return
    if let Some(printer) = printer {
        match &best {
            Some(tree_node) => println!(
                "{}best guess is {} with est cost of {}",
                printer.get_prefix(),
                match tree_node.should_guess {
                    GuessFrom::Guess(guess_ind) => printer.fmt_guess(guess_ind),
                    GuessFrom::Answer(answer_ind) => printer.fmt_answer(answer_ind),
                },
                tree_node.est_cost
            ),
            None => println!(
                "{}no guesses are guaranteed to solve within depth limit",
                printer.get_prefix(),
            ),
        }
    }
    best
}
