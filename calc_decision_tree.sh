#!/usr/bin/env bash
set -ex
cargo build --example calc_decision_tree_general --release
time ./target/release/examples/calc_decision_tree_general word_lists/"$1" word_lists/"$1"
