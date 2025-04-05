use crate::{tc_config::ConfigTC, utils::calculate_message_width};
use crossterm::style::{Color, Stylize};

const UPPER_LEFT: &str = "┌";
const UPPER_RIGHT: &str = "┐";
const BOTTOM_LEFT: &str = "└";
const BOTTOM_RIGHT: &str = "┘";
const HORIZONTAL_BAR: &str = "─";
const VERTICAL_BAR: &str = "│";

pub(crate) enum MessageType {
    User,
    Assistant,
    System,
}

const MAX_CHAT_WIDTH: usize = 50;
const MESSAGE_WIDTH_PERCENT: usize = 80;

pub(crate) fn print_message(message_text: &str, message_type: MessageType, config: &ConfigTC) {
    let max_width = calculate_message_width(message_text, MAX_CHAT_WIDTH, MESSAGE_WIDTH_PERCENT);

    let effective_width = max_width - 2; // For the box characters on both sides

    let prefix = match message_type {
        MessageType::User => " ".repeat(MAX_CHAT_WIDTH - max_width),
        MessageType::System => {
            let space = (MAX_CHAT_WIDTH - max_width) / 2;
            " ".repeat(space)
        }
        _ => String::new(),
    };

    let color = match message_type {
        MessageType::User => parse_color(&config.theme.user_color),
        MessageType::Assistant => parse_color(&config.theme.assistant_color),
        MessageType::System => parse_color(&config.theme.system_color),
    };

    let first_row = match message_type {
        MessageType::User => format!(
            "{}{}{}{}",
            UPPER_LEFT.with(color),
            HORIZONTAL_BAR.repeat(effective_width - 4).with(color),
            "User",
            UPPER_RIGHT.with(color)
        ),
        MessageType::Assistant => format!(
            "{}{}{}{}",
            UPPER_LEFT.with(color),
            "Assistant",
            HORIZONTAL_BAR.repeat(effective_width - 9).with(color),
            UPPER_RIGHT.with(color)
        ),
        MessageType::System => format!(
            "{}{}{}",
            UPPER_LEFT.with(color),
            HORIZONTAL_BAR.repeat(effective_width).with(color),
            UPPER_RIGHT.with(color)
        ),
    };

    let vertical_bar_styled = VERTICAL_BAR.with(color);
    let body = word_wrap(message_text, max_width, vertical_bar_styled.to_string());

    let formatted_body = match message_type {
        MessageType::User | MessageType::System => {
            let lines = body.trim_end().split('\n');
            lines
                .map(|line| format!("{}{}", prefix, line))
                .collect::<Vec<_>>()
                .join("\n")
        }
        _ => body.trim_end().to_string(),
    };

    let last_row = format!(
        "{}{}{}",
        BOTTOM_LEFT.with(color),
        HORIZONTAL_BAR.repeat(effective_width).with(color),
        BOTTOM_RIGHT.with(color)
    );

    println!(
        "\n{}{}\n{}\n{}{}",
        prefix, first_row, formatted_body, prefix, last_row
    );
}

fn word_wrap(text: &str, width: usize, wrapper: String) -> String {
    let mut result = String::new();
    let effective_width = width - 4;

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

fn parse_color(color_name: &str) -> Color {
    match color_name.to_lowercase().as_str() {
        "red" => Color::Red,
        "green" => Color::Green,
        "yellow" => Color::Yellow,
        "blue" => Color::Blue,
        "magenta" => Color::Magenta,
        "cyan" => Color::Cyan,
        "white" => Color::White,
        "black" => Color::Black,
        "dark_grey" => Color::DarkGrey,
        "light_grey" => Color::Grey, // Also known as Light Grey
        "dark_red" => Color::DarkRed,
        "dark_green" => Color::DarkGreen,
        "dark_yellow" => Color::DarkYellow,
        "dark_blue" => Color::DarkBlue,
        "dark_magenta" => Color::DarkMagenta,
        "dark_cyan" => Color::DarkCyan,
        _ => Color::Reset,
    }
}
