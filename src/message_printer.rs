use crate::conversation::Role;
use crate::tc_config::ConfigTC;
use crossterm::style::{Color, Stylize};

pub fn print_message(text: &str, role: Role, config: &ConfigTC) {
    let color = match role {
        Role::User => parse_color(&config.theme.user_color),
        Role::Assistant => parse_color(&config.theme.assistant_color),
        Role::Developer | Role::System => parse_color(&config.theme.system_color),
    };

    let label = role.display_name();
    let styled_label = format!("[{}]", label).with(color);

    for (i, line) in text.lines().enumerate() {
        if i == 0 {
            println!("{} {}", styled_label, line);
        } else {
            let padding = " ".repeat(label.len() + 3);
            println!("{}{}", padding, line);
        }
    }
}

pub fn parse_color(color_name: &str) -> Color {
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
        "light_grey" => Color::Grey,
        "dark_red" => Color::DarkRed,
        "dark_green" => Color::DarkGreen,
        "dark_yellow" => Color::DarkYellow,
        "dark_blue" => Color::DarkBlue,
        "dark_magenta" => Color::DarkMagenta,
        "dark_cyan" => Color::DarkCyan,
        _ => Color::Reset,
    }
}
