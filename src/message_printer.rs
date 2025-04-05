use colored::{ColoredString, Colorize};

const UPPER_LEFT: &str = "┌";
const UPPER_RIGHT: &str = "┐";
const BOTTOM_LEFT: &str = "└";
const BOTTOM_RIGHT: &str = "┘";
const HORIZONTAL_BAR: &str = "─";
const VERTICAL_BAR: &str = "│";

pub(crate) enum MessageType {
    User,
    Assistant,
}

const MAX_CHAT_WIDTH: usize = 45;
const MESSAGE_WIDTH_PERCENT: usize = 80;

pub(crate) fn print_message(message_text: &str, message_type: MessageType) {
    let terminal_width = termsize::get().map(|size| size.cols as usize).unwrap_or(80);
    let max_chat_width = terminal_width.min(MAX_CHAT_WIDTH);

    let max_width = max_chat_width * MESSAGE_WIDTH_PERCENT / 100;

    let lines: Vec<&str> = message_text.lines().collect();
    let is_single_line = lines.len() == 1;

    let color = match message_type {
        MessageType::User => "green",
        MessageType::Assistant => "blue",
    };

    let width = if is_single_line {
        let content_width = lines[0].len() + 4; // Add 4 for padding (2 on each side)
        let label_width = match message_type {
            MessageType::User => 8,       // "User" (4) + padding (4)
            MessageType::Assistant => 13, // "Assistant" (9) + padding (4)
        };
        (content_width.max(label_width) + 2).min(max_width) // +2 for wrapper chars
    } else {
        max_width
    };

    let effective_width = width - 2; // For the box characters on both sides

    // Right align user messages within the max_chat_width
    let prefix = if matches!(message_type, MessageType::User) {
        " ".repeat(max_chat_width - width)
    } else {
        String::new()
    };

    let first_row = match message_type {
        MessageType::User => {
            let left_padding = effective_width - 4;
            format!(
                "{}{}{}{}",
                UPPER_LEFT.color(color),
                HORIZONTAL_BAR.repeat(left_padding).color(color),
                "User",
                UPPER_RIGHT.color(color)
            )
        }
        MessageType::Assistant => {
            format!(
                "{}{}{}{}",
                UPPER_LEFT.color(color),
                "Assistant",
                HORIZONTAL_BAR.repeat(effective_width - 9).color(color),
                UPPER_RIGHT.color(color)
            )
        }
    };

    let body = word_wrap(message_text, width, VERTICAL_BAR.color(color));

    let formatted_body = if matches!(message_type, MessageType::User) {
        let lines = body.trim_end().split('\n');
        lines
            .map(|line| format!("{}{}", prefix, line))
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        body.trim_end().to_string()
    };

    let last_row = format!(
        "{}{}{}",
        BOTTOM_LEFT.color(color),
        HORIZONTAL_BAR.repeat(effective_width).color(color),
        BOTTOM_RIGHT.color(color)
    );

    println!(
        "\n{}{}\n{}\n{}{}",
        prefix, first_row, formatted_body, prefix, last_row
    );
}

fn word_wrap(text: &str, width: usize, wrapper: ColoredString) -> String {
    let mut result = String::new();
    let effective_width = width - 4; // Account for wrapper chars and padding (1 on each side)

    for line in text.lines() {
        let chars: Vec<char> = line.chars().collect();
        let mut current_pos = 0;

        while current_pos < chars.len() {
            let mut end_pos = (current_pos + effective_width).min(chars.len());

            // If we're not at the end and the next character isn't whitespace,
            // try to move the end_pos back to the last whitespace.
            if end_pos < chars.len() && !chars[end_pos].is_whitespace() {
                if let Some(last_space) = chars[current_pos..end_pos]
                    .iter()
                    .rposition(|c| c.is_whitespace())
                {
                    end_pos = current_pos + last_space + 1;
                }
            }

            let line_content: String = chars[current_pos..end_pos].iter().collect();
            let line_content = line_content.trim_end();

            result.push_str(&format!(
                "{} {:<width$} {}\n",
                wrapper,
                line_content,
                wrapper,
                width = effective_width
            ));

            current_pos = end_pos;
            // Skip any additional whitespace
            while current_pos < chars.len() && chars[current_pos].is_whitespace() {
                current_pos += 1;
            }
        }
    }

    if result.is_empty() {
        result.push_str(&format!(
            "{} {} {}\n",
            wrapper,
            " ".repeat(effective_width),
            wrapper
        ));
    }

    result
}
