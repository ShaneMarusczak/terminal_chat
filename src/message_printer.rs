use crate::{tc_config::ConfigTC, utils::calculate_message_width};
use crossterm::style::{Color, Stylize};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

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

const MAX_CHAT_WIDTH: usize = 70;
const MESSAGE_WIDTH_PERCENT: usize = 80;

pub(crate) fn print_message(message_text: &str, message_type: MessageType, config: &ConfigTC) {
    let lines: Vec<&str> = message_text.lines().collect();

    let (calculated_width, terminal_width) =
        calculate_message_width(message_text, MAX_CHAT_WIDTH, MESSAGE_WIDTH_PERCENT);

    let min_width = match message_type {
        MessageType::User => 6,
        MessageType::Assistant => 11,
        MessageType::System => 0,
    };

    let effective_max_width = if lines.len() > 1 && matches!(message_type, MessageType::System) {
        let max_line_length = lines.iter().map(|line| line.width()).max().unwrap_or(0);
        max_line_length.min(calculated_width) + 4
    } else {
        calculated_width
    };
    let effective_width = effective_max_width.max(min_width) - 2;

    let prefix = match message_type {
        MessageType::User => " ".repeat((terminal_width.min(MAX_CHAT_WIDTH) - 2) - effective_width),
        MessageType::System => {
            let space = (terminal_width.min(MAX_CHAT_WIDTH) - effective_width) / 2;
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
    let body = word_wrap(
        message_text,
        effective_max_width.max(min_width),
        vertical_bar_styled.to_string(),
    );

    let formatted_body = {
        let lines = body.trim_end().split('\n');
        lines
            .map(|line| format!("{}{}", prefix, line))
            .collect::<Vec<_>>()
            .join("\n")
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
    let effective_width = width - 4;
    let mut result = String::new();

    for line in text.lines() {
        let graphemes: Vec<&str> = line.graphemes(true).collect();
        let mut current_pos = 0;

        while current_pos < graphemes.len() {
            let mut current_width = 0;
            let mut end_pos = current_pos;

            // Accumulate graphemes' width until we reach the effective width
            while end_pos < graphemes.len() {
                let g_width = graphemes[end_pos].width();
                if current_width + g_width > effective_width {
                    break;
                }
                current_width += g_width;
                end_pos += 1;
            }

            // Try to backtrack to the last space if we're mid-word
            if end_pos < graphemes.len() && !graphemes[end_pos].trim().is_empty() {
                if let Some(last_space) = graphemes[current_pos..end_pos]
                    .iter()
                    .rposition(|g| g.trim().is_empty())
                {
                    end_pos = current_pos + last_space + 1;
                }
            }

            let line_content = graphemes[current_pos..end_pos].join("");
            let final_width: usize = line_content.graphemes(true).map(|g| g.width()).sum();
            let final_dif = (effective_width - final_width).max(0);

            result.push_str(&format!(
                "{} {}{} {}\n",
                wrapper,
                line_content,
                " ".repeat(final_dif),
                wrapper
            ));

            // Move past any trailing spaces
            current_pos = end_pos;
            while current_pos < graphemes.len() && graphemes[current_pos].trim().is_empty() {
                current_pos += 1;
            }
        }
    }

    // If the entire text was empty, add one empty line
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
