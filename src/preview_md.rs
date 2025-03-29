use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};

fn ansi_bold_on() -> &'static str {
    "\x1b[1m"
}
fn ansi_bold_off() -> &'static str {
    "\x1b[22m"
}
fn ansi_italic_on() -> &'static str {
    "\x1b[3m"
}
fn ansi_italic_off() -> &'static str {
    "\x1b[23m"
}
fn ansi_underline_on() -> &'static str {
    "\x1b[4m"
}
fn ansi_underline_off() -> &'static str {
    "\x1b[24m"
}
fn ansi_strikethrough_on() -> &'static str {
    "\x1b[9m"
}
fn ansi_strikethrough_off() -> &'static str {
    "\x1b[29m"
}
fn ansi_reset() -> &'static str {
    "\x1b[0m"
}

pub(crate) fn preview_markdown(md_str: &str) {
    let s = markdown_to_ansi(md_str);
    println!("{s}");
}

/// Converts a Markdown string into styled ANSI text.
pub(crate) fn markdown_to_ansi(markdown: &str) -> String {
    let parser = Parser::new_ext(markdown, Options::all());
    let mut output = String::new();

    // Track heading depth, nesting levels, etc. for formatting
    let mut list_stack = vec![];
    let mut heading_level = 0;

    for event in parser {
        match event {
            // Text
            Event::Text(text) => {
                // If we're in a heading, color or style accordingly
                if (1..=6).contains(&heading_level) {
                    output.push_str(&format!("\x1b[1;3{}m{}\x1b[0m", heading_level, text));
                } else {
                    output.push_str(&text);
                }
            }
            // Code spans (inline)
            Event::Code(text) => {
                // Use a different color or style for inline code
                output.push_str("\x1b[96m");
                output.push_str(&text);
                output.push_str("\x1b[0m");
            }
            // Handles <hr> (horizontal rule)
            Event::Rule => {
                output.push_str("\n\x1b[90m----------------\x1b[0m\n");
            }
            // Soft break (wrap)
            Event::SoftBreak => {
                output.push('\n');
            }
            // Hard line break
            Event::HardBreak => {
                output.push_str("\n\n");
            }
            // Start of a tag
            Event::Start(tag) => match tag {
                Tag::Strong => {
                    output.push_str(ansi_bold_on());
                }
                Tag::Emphasis => {
                    output.push_str(ansi_italic_on());
                }
                Tag::Strikethrough => {
                    output.push_str(ansi_strikethrough_on());
                }
                Tag::Heading {
                    level,
                    id: _,
                    classes: _,
                    attrs: _,
                } => {
                    heading_level = match level {
                        HeadingLevel::H1 => 1,
                        HeadingLevel::H2 => 2,
                        HeadingLevel::H3 => 3,
                        HeadingLevel::H4 => 4,
                        HeadingLevel::H5 => 5,
                        HeadingLevel::H6 => 6,
                    };
                    output.push_str(ansi_bold_on());
                }
                Tag::BlockQuote(_) => {
                    // Handle different kinds if necessary
                    output.push_str("\n\x1b[90m>\x1b[0m ");
                }
                Tag::CodeBlock(kind) => {
                    // For code blocks, add a color
                    output.push('\n');
                    match kind {
                        CodeBlockKind::Indented => {
                            output.push_str("\x1b[94m"); // color
                        }
                        CodeBlockKind::Fenced(lang) => {
                            output.push_str("\x1b[94m"); // can check lang if desired
                            let _ = lang; // do something with the language if you want syntax highlighting
                        }
                    }
                }
                Tag::List(start) => {
                    list_stack.push(start);
                    output.push('\n');
                }
                Tag::Item => {
                    output.push_str("* ");
                }
                Tag::Link {
                    link_type: _,
                    dest_url,
                    title: _,
                    id: _,
                } => {
                    // Underline links
                    output.push_str(ansi_underline_on());
                    // Optionally display the URL after: "[text](dest)"
                    output.push_str(&format!("(Link: {}) ", dest_url));
                }
                Tag::Image {
                    link_type: _,
                    dest_url,
                    title: _,
                    id: _,
                } => {
                    // Just show the source in parentheses
                    output.push_str(&format!("(Image: {})", dest_url));
                }
                Tag::FootnoteDefinition(name) => {
                    output.push_str(&format!("[Footnote: {}]", name));
                }
                Tag::Table(_alignments) => {
                    output.push_str("\n\x1b[4m"); // Underline table text
                }
                Tag::TableHead => { /* Could style table headers differently */ }
                Tag::TableRow => {}
                Tag::TableCell => {}
                _ => {}
            },
            // End of a tag
            Event::End(tag) => match tag {
                TagEnd::Strong => {
                    output.push_str(ansi_bold_off());
                }
                TagEnd::Emphasis => {
                    output.push_str(ansi_italic_off());
                }
                TagEnd::Strikethrough => {
                    output.push_str(ansi_strikethrough_off());
                }
                TagEnd::Heading(_) => {
                    heading_level = 0;
                    output.push_str(ansi_bold_off());
                    output.push('\n');
                }
                TagEnd::BlockQuote(_) => {
                    // currently ignoring 'kind'
                    output.push_str("\n\x1b[90m>\x1b[0m ");
                }
                TagEnd::CodeBlock => {
                    output.push_str(ansi_reset());
                    output.push('\n');
                }
                TagEnd::List(_) => {
                    list_stack.pop();
                    output.push('\n');
                }
                TagEnd::Item => {
                    output.push('\n');
                }
                TagEnd::Link => {
                    // Close underline
                    output.push_str(ansi_underline_off());
                }
                TagEnd::Image => {}
                TagEnd::Table => {
                    output.push_str("\x1b[0m\n"); // reset style
                }
                TagEnd::TableHead => {}
                TagEnd::TableRow => {
                    output.push('\n');
                }
                TagEnd::TableCell => {
                    output.push('\t');
                }
                TagEnd::FootnoteDefinition => {}
                _ => {}
            },
            // HTML or footnotes – we ignore or handle them plainly
            Event::Html(html) => {
                // Just append, or handle if you want to parse further
                output.push_str(&html);
            }
            Event::FootnoteReference(name) => {
                output.push_str(&format!("[^{}]", name));
            }
            // Task list markers: could handle or ignore
            Event::TaskListMarker(is_checked) => {
                let mark = if is_checked { "[x] " } else { "[ ] " };
                output.push_str(mark);
            }
            _ => {}
        }
    }

    output
}
