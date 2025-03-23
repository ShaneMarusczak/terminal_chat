use tokio::sync::Mutex;

use crate::{
    chat_client::ChatClient,
    commands_registry::{CommandContext, TC_COMMANDS},
    conversation::{ConversationContext, Message},
};
use std::{error::Error, sync::Arc};

pub async fn handle_command(
    cmd: &str,
    context: Arc<Mutex<ConversationContext>>,
    dev_message: Arc<Message>,
    chat_client: Arc<ChatClient>,
) -> Result<(), Box<dyn Error>> {
    let cmd_string = cmd.trim().to_owned();
    let main_cmd = cmd_string.split_whitespace().next().unwrap().to_owned();
    let args: Vec<String> = cmd_string
        .split_whitespace()
        .skip(1)
        .map(|arg| arg.to_owned())
        .collect();

    let cc = CommandContext::new(
        Arc::clone(&context),
        Arc::clone(&dev_message),
        Arc::clone(&chat_client),
        main_cmd.clone(),
        args,
    );

    if let Some(tc) = TC_COMMANDS.get(main_cmd.as_str()) {
        //This line was fun to write
        (tc.run)(cc).await?;
    } else {
        eprintln!("\nUnknown command: {}", main_cmd);
        let words: Vec<String> = TC_COMMANDS.keys().map(|key| key.to_string()).collect();
        let maybe = find_matching_word(&main_cmd, words)?;
        eprintln!("Did you mean {maybe}?\n");
    }

    Ok(())
}

fn find_matching_word(word: &str, words: Vec<String>) -> Result<String, String> {
    let mut min_dist = 9999;
    let mut final_string = String::new();
    for w in words {
        let distance = min_distance(&w, word);
        if distance < min_dist {
            min_dist = distance;
            final_string = w;
        }
    }
    Ok(final_string)
}

fn min_distance(word1: &str, word2: &str) -> i32 {
    let (word1, word2) = (word1.as_bytes(), word2.as_bytes());
    let mut dist = Vec::with_capacity(word2.len() + 1);
    for j in 0..=word2.len() {
        dist.push(j)
    }
    let mut prev_dist = dist.clone();
    for i in 1..=word1.len() {
        for j in 0..=word2.len() {
            if j == 0 {
                dist[j] += 1;
            } else if word1[i - 1] == word2[j - 1] {
                dist[j] = prev_dist[j - 1];
            } else {
                dist[j] = dist[j].min(dist[j - 1]).min(prev_dist[j - 1]) + 1;
            }
        }
        prev_dist.copy_from_slice(&dist);
    }
    dist[word2.len()] as i32
}
