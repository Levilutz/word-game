use std::{collections::HashMap, env::args, fs};

use serde::{Deserialize, Serialize};
use word_core::{hint::WordHint, word::Word};

const WORD_SIZE: usize = 5;
const ALPHABET_SIZE: u8 = 26;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TreeNode {
    should_guess: Word<WORD_SIZE, ALPHABET_SIZE>,
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    next: HashMap<WordHint<WORD_SIZE>, TreeNode>,
}

fn load_tree(file_path: &str) -> TreeNode {
    serde_json::from_str(&fs::read_to_string(file_path).unwrap()).unwrap()
}

fn main() {
    let tree_a = load_tree(&args().nth(1).expect("must supply two file paths"));
    let tree_b = load_tree(&args().nth(2).expect("must supply two file paths"));

    assert_eq!(tree_a, tree_b);
}
