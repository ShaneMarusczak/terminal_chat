use std::io::stdin;

use crate::conversation::Response;

pub fn extract_message_text(response: &Response) -> Option<String> {
    for output in &response.output {
        if output.type_field == "message" {
            if let Some(content) = &output.content {
                if let Some(first_content) = content.first() {
                    return Some(first_content.text.clone());
                }
            }
        }
    }
    None
}

pub fn read_user_input(prompt: &str) -> String {
    println!("{}", prompt);
    let mut input = String::new();
    stdin().read_line(&mut input).expect("Failed to read line");
    input.trim().to_string()
}

pub fn confirm_action(prompt: &str) -> bool {
    let response = read_user_input(prompt);
    response.eq_ignore_ascii_case("y") || response.eq_ignore_ascii_case("yes")
}
