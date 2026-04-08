use tokio::sync::Mutex;

use crate::{
    commands::{command_context::CommandContext, commands_registry::TC_COMMANDS},
    conversation::{ConversationContext, Message},
};
use std::{error::Error, sync::Arc};

pub async fn handle_command(
    cmd: &str,
    context: Arc<Mutex<ConversationContext>>,
    dev_message: Arc<Message>,
) -> Result<(), Box<dyn Error>> {
    let trimmed = cmd.trim();
    let (main_cmd, rest) = trimmed
        .split_once(char::is_whitespace)
        .map(|(c, r)| (c, r.trim_start()))
        .unwrap_or((trimmed, ""));

    if main_cmd.is_empty() {
        return Err("No command provided".into());
    }

    let cc = CommandContext::new(
        Arc::clone(&context),
        Arc::clone(&dev_message),
        main_cmd.to_owned(),
        split_args(rest),
    );

    if let Some(tc) = TC_COMMANDS.get(main_cmd) {
        (tc.run)(Some(cc)).await?;
    } else {
        eprintln!("\nUnknown command: {}", main_cmd);
        let words: Vec<String> = TC_COMMANDS.keys().map(|s| s.to_string()).collect();
        let maybe = find_matching_word(main_cmd, &words)?;
        eprintln!("Did you mean {maybe}?\n");
    }

    Ok(())
}

/// Splits a string into args, respecting single and double quotes.
pub fn split_args(input: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut quote: Option<char> = None;

    for ch in input.chars() {
        match (quote, ch) {
            (Some(q), c) if c == q => quote = None,
            (Some(_), c) => current.push(c),
            (None, '"') | (None, '\'') => quote = Some(ch),
            (None, c) if c.is_whitespace() => {
                if !current.is_empty() {
                    args.push(std::mem::take(&mut current));
                }
            }
            (None, c) => current.push(c),
        }
    }
    if !current.is_empty() {
        args.push(current);
    }
    args
}

/// Returns the entry from `words` with the smallest Levenshtein distance
/// to `word`. Errors only if the word list is empty.
pub fn find_matching_word(word: &str, words: &[String]) -> Result<String, String> {
    words
        .iter()
        .min_by_key(|w| min_distance(w, word))
        .cloned()
        .ok_or_else(|| "No suggestions available".to_string())
}

/// Levenshtein distance using a single rolling row of DP state.
pub fn min_distance(word1: &str, word2: &str) -> usize {
    let (a, b) = (word1.as_bytes(), word2.as_bytes());
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut curr = vec![0usize; b.len() + 1];

    for i in 1..=a.len() {
        curr[0] = i;
        for j in 1..=b.len() {
            curr[j] = if a[i - 1] == b[j - 1] {
                prev[j - 1]
            } else {
                1 + curr[j - 1].min(prev[j]).min(prev[j - 1])
            };
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[b.len()]
}
