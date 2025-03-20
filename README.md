# Terminal Chat (tc)

# rm-repl [![Rust](https://github.com/ShaneMarusczak/terminal-chat/actions/workflows/rust.yml/badge.svg?branch=main&event=push)](https://github.com/ShaneMarusczak/terminal-chat/actions/workflows/rust.yml)

Terminal Chat is a lightweight, interactive chat interface built in Rust. It allows users to converse with an AI powered by the OpenAI API from the terminal. Key features include real‐time chat interactions, context tracking, file-based conversation augmentation, document generation with markdown output, and support for various panel commands—all wrapped in a user-friendly terminal UI.

## Features

- Real-time chat with GPT-style models.
- Command support (e.g., “:quit” to exit).
- Conversation history across your session.
- Minimal configuration (just one environment variable).
- Built-in commands for a smoother user experience.

## Prerequisites

- [Rust](https://www.rust-lang.org/) (stable).
- The "OPENAI_API_KEY" environment variable with your OpenAI API key.

## Installation

1. Clone or download this repository.
2. Navigate to the project directory.
3. Build the project:
   cargo build --release

## Running the App

1. Export your API key:
   export OPENAI_API_KEY="your_openai_api_key"
2. Run it:
   cargo run
3. Enter chat messages or commands as needed (e.g., “:quit”).

## Usage

1. **Available Commands:**
   - **:clear** – Clear the current conversation context.
   - **:debug** – Display debugging information (current model and conversation messages).
   - **:doc** – Generate a document report based on the conversation.
   - **:cm** – Change the chat model (select from available options).
   - **:gf \<files\>** – Add content from one or more files to the conversation.
   - **:rmr** – Launch the external `rmr` tool if installed.
   - **:readme \<directory\> [extensions...]** – Process files from a directory into a formatted README in markdown.
   - **:help** – Show command usage help.
   - **:q / :quit** – Exit the application.

2. **Chat Flow:**
   - Enter plain text to chat with the assistant.
   - Use the colon-prefixed commands to perform administrative tasks or generate documentation.
   
## Contribution Guidelines

Contributions are welcome! If you wish to contribute:
- Fork the repository and create a feature/bug fix branch.
- Ensure your code adheres to existing conventions and is well-documented.
- Feel free to open issues or pull requests.
- For detailed guidelines, please refer to the [CONTRIBUTING.md](CONTRIBUTING.md) file if available.

---

## License

This project is licensed under the terms specified in the [LICENSE](LICENSE) file. Please review this file for additional details.
