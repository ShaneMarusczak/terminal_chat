use std::error::Error;

#[derive(Debug, PartialEq, Eq)]
pub enum Mode {
    Chat,
    Doc,
    Readme,
}

pub struct CliArgs {
    pub mode: Mode,
    // shared
    pub model_override: Option<String>,
    pub use_local: bool,
    pub max_tokens: Option<usize>,
    pub system_override: Option<String>,
    pub help: bool,
    pub gf_files: Vec<String>,
    pub output: Option<String>,
    // chat / doc
    pub prompt: Option<String>,
    // readme
    pub directory: Option<String>,
    pub extensions: Vec<String>,
}

impl CliArgs {
    fn new() -> Self {
        Self {
            mode: Mode::Chat,
            model_override: None,
            use_local: false,
            max_tokens: None,
            system_override: None,
            help: false,
            gf_files: Vec::new(),
            output: None,
            prompt: None,
            directory: None,
            extensions: Vec::new(),
        }
    }
}

/// Pull the next argument or fail with a flag-specific message.
fn take_value(args: &[String], i: usize, err: &'static str) -> Result<String, &'static str> {
    args.get(i).ok_or(err).cloned()
}

pub fn parse_cli_args(args: &[String]) -> Result<CliArgs, Box<dyn Error>> {
    let mut cli = CliArgs::new();
    let mut prompt_parts: Vec<String> = Vec::new();
    let mut positional_index: usize = 0;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" | "help" => cli.help = true,
            "-m" | "--model" => {
                i += 1;
                cli.model_override = Some(take_value(args, i, "--model requires a model name")?);
            }
            "--local" => cli.use_local = true,
            "--max-tokens" => {
                i += 1;
                let raw = take_value(args, i, "--max-tokens requires a number")?;
                cli.max_tokens = Some(
                    raw.parse::<usize>()
                        .map_err(|_| "--max-tokens must be a positive integer")?,
                );
            }
            "-s" | "--system" => {
                i += 1;
                cli.system_override =
                    Some(take_value(args, i, "--system requires a text argument")?);
            }
            "-o" | "--output" => {
                i += 1;
                cli.output = Some(take_value(args, i, "--output requires a path")?);
            }
            "--gf" => {
                i += 1;
                // Slurp until: end of args, next flag, or first non-file arg
                // (lets a trailing positional prompt work naturally).
                while let Some(arg) = args.get(i)
                    && !arg.starts_with('-')
                    && std::path::Path::new(arg).is_file()
                {
                    cli.gf_files.push(arg.clone());
                    i += 1;
                }
                continue;
            }
            other => {
                consume_positional(&mut cli, &mut prompt_parts, positional_index, other);
                positional_index += 1;
            }
        }
        i += 1;
    }

    if !prompt_parts.is_empty() {
        cli.prompt = Some(prompt_parts.join(" "));
    }
    Ok(cli)
}

fn consume_positional(
    cli: &mut CliArgs,
    prompt_parts: &mut Vec<String>,
    positional_index: usize,
    arg: &str,
) {
    // Subcommand detection only on the very first positional argument.
    if positional_index == 0 {
        match arg {
            "doc" => {
                cli.mode = Mode::Doc;
                return;
            }
            "readme" => {
                cli.mode = Mode::Readme;
                return;
            }
            _ => {}
        }
    }

    match cli.mode {
        Mode::Chat | Mode::Doc => prompt_parts.push(arg.to_string()),
        Mode::Readme => {
            if cli.directory.is_none() {
                cli.directory = Some(arg.to_string());
            } else {
                cli.extensions.push(arg.to_string());
            }
        }
    }
}
