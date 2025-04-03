<p align="center"><img src="./images/tc_logo-min-removebg-preview.png" width="256"/></p>

# Terminal Chat (tc)

[![Rust CI](https://github.com/ShaneMarusczak/terminal_chat/actions/workflows/rust.yml/badge.svg?branch=main)](https://github.com/ShaneMarusczak/terminal_chat/actions/workflows/rust.yml)

Terminal Chat is an interactive, terminal‐based chat application built with Rust. It allows you to have real-time conversations with OpenAI and Anthropic models while leveraging various commands to manage and document sessions. The project is designed to be highly efficient, minimal, and provides a responsive command-line interface with an integrated spinner and auto-completion support.

---

## Features

- **Interactive REPL Interface:** Start a conversation in a command–line REPL with rich history, autocompletion (via linefeed), and an animated spinner during long-running API calls.
- **API Integration:** Communicates with OpenAI’s and Anthropic’s endpoints for chat completions and image generations.
- **Command Suite:** Execute a variety of built-in commands including changing models, loading/saving conversations, generating README documentation, executing shell commands, and more.
- **Custom Configuration:** Easily configure the chat model, streaming options, and custom developer prompts via a config file.
- **Markdown Preview:** Render and preview markdown responses directly in the terminal with ANSI styling.

---

## Installation

1. **Prerequisites:**
   - Install the latest stable version of [Rust](https://www.rust-lang.org/tools/install).
   - Ensure your system provides a proper terminal that supports ANSI escape codes.
   - Obtain API keys from your desired providers:
     - Set the `OPENAI_API_KEY` environment variable for OpenAI endpoints.
     - Set the `ANTHROPIC_API_KEY` environment variable for Anthropic endpoints.

2. **Clone the Repository:**

   ```sh
   git clone https://github.com/yourusername/tc.git
   cd tc
   ```

3. **Build the Project:**

   Run in release mode for optimized performance:

   ```sh
   cargo run --release
   ```

4. **Configuration Setup:**

TC Terminal Chat uses a JSON configuration file that is stored in your system’s configuration directory (or in the current directory as a fallback). Key configuration settings include:

- **enable_streaming:** Enable/disable streaming responses (default: false).
- **model:** The default AI model (default: `gpt-4o-mini`).
- **dev_message:** A custom developer message that guides AI responses.
- **preview_md:** When enabled, non-streamed responses are displayed as rendered markdown.

You can update these settings interactively on first run or manually by editing the file (`tc_config.json`).

---

## Usage Guide

- **Chatting:**
  Simply type your message at the prompt (🗣️) and press Enter. The application will send your input to the chosen model and display the response in real-time.

- **Commands:**
  Commands are prefixed with a colon (`:`). Some common commands include:

  - `:help` – Display this help and available commands
  - `:clear` – Clears the current conversation context
  - `:cm` – Change the active Chat model
  - `:gf <file1> <file2> ...` – Add the contents of specified files to the conversation context
  - `:readme <directory> [extensions...]` – Generate a README.md document based on files in a directory
  - `:doc` – Document the current context into a Markdown report
  - `:q` or `:quit` – Quit the application

- **Tips:**

  - If a command is unrecognized, the tool will suggest a similar command based on minimum edit distance.
  - For commands that generate output files (such as readme and doc), follow the prompts to confirm the filename and save location.

---

## Configuration Details

TC Terminal Chat uses a JSON configuration file that is stored in your system’s configuration directory (or in the current directory as a fallback). Key configuration settings include:

- **enable_streaming:** Enable/disable streaming responses (default: false).
- **model:** The default AI model (default: `gpt-4o-mini`).
- **dev_message:** A custom developer message that guides AI responses.
- **preview_md:** When enabled, non-streamed responses are displayed as rendered markdown.

You can update these settings interactively on first run or manually by editing the file (`tc_config.json`).

---

## Contribution Guidelines

Contributions are welcome! Please follow these steps:

1. Fork the repository and clone your fork.
2. Create a new branch for your feature or bugfix.
3. Ensure your code adheres to Rust coding best practices.
4. Submit a pull request with detailed information about your changes.

---

## License

This project is licensed under the terms specified in the [LICENSE](LICENSE) file.

---

Happy chatting!
