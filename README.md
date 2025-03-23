# Terminal Chat (tc)

[![Rust CI](https://github.com/ShaneMarusczak/terminal_chat/actions/workflows/rust.yml/badge.svg?branch=main)](https://github.com/ShaneMarusczak/terminal_chat/actions/workflows/rust.yml)

Terminal Chat is an interactive, terminal‐based chat application built with Rust. It allows you to have real-time conversations with OpenAI's API while leveraging various commands to manage and document sessions. The project is designed to be highly efficient, minimal, and provides a responsive command-line interface with an integrated spinner and auto-completion support.

---

## Features

• Real-time streaming chat support via asynchronous HTTP requests using Reqwest and Tokio
• Command parser for extended functionality
• Integrated spinner for visual feedback during long-running API calls
• File-based context augmentation (upload file contents to chat context)
• Automated documentation and report generation in Markdown
• Modular design for easy future expansion

---

## Installation

1. **Prerequisites**

   - Rust (latest stable version; requires Rust 2024 edition)
   - Cargo package manager
   - An OpenAI API key – set your environment variable:
     `export OPENAI_API_KEY=your_api_key_here`

2. **Clone the repository**

   Run the following commands:

   $ git clone <repository_url>
   $ cd terminal-chat

3. **Build the project**

   $ cargo build --release

4. **Run the project**

   $ cargo run --release

---

## Usage Guide

- **Chatting:**
  Simply type your message at the prompt (🗣️). The application will send your input to the OpenAI API and stream the response in real-time.

- **Commands:**
  Commands are prefixed with a colon (`:`). Some common commands include:

  • `:help` – Display this help and available commands
  • `:clear` – Clears the current conversation context
  • `:cm` – Change the active Chat model
  • `:gf <file1> <file2> ...` – Add the contents of specified files to the conversation context
  • `:readme <directory> [extensions...]` – Generate a README.md document based on files in a directory
  • `:doc` – Document the current context into a Markdown report
  • `:q` or `:quit` – Quit the application

- **Tips:**

  • If a command is unrecognized, the tool will suggest a similar command based on minimum edit distance.
  • For commands that generate output files (such as readme and doc), follow the prompts to confirm the filename and save location.

---

## File and Structure Overview

- **Cargo.toml**
  Contains project metadata, version, compilation profiles (release optimizations, panic-abort, LTO), and dependency definitions.

- **src/main.rs**
  Sets up the terminal interface using the `linefeed` crate, initializes the conversation context, and manages the main input loop.

- **src/chat_client.rs**
  Implements the HTTP client that interacts with the OpenAI API.
  Provides streaming and non-streaming request methods.

- **src/conversation.rs**
  Defines data structures for messages and conversation contexts.
  Contains types for responses and helper functions to extract data.

- **src/commands_registry.rs & src/commands.rs**
  Registers available commands (like clearing the conversation, model switching, document generation, etc.) and handles command parsing and execution.

- **src/spinner.rs**
  Provides a vibrant spinner animation using `crossterm` to indicate ongoing asynchronous operations.

- **src/messages.rs**
  Defines pre-set prompt messages for different operations (developer prompts, README generation, documentation requests, etc.)

---

## Configuration Details

• The project uses a custom release profile configured in Cargo.toml:
  - `panic = "abort"`
  - `lto = true`
  - `opt-level = 'z'` (size optimizations)
  - `codegen-units = 1`
  - `strip = true`
  - `debug = false`
  - `incremental = false`

• Environment Variable:
  Set `OPENAI_API_KEY` to your valid OpenAI API key.

• API Endpoints:
  - Standard API: `https://api.openai.com/v1/responses`
  - Chat completions: `https://api.openai.com/v1/chat/completions`

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
