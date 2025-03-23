# Terminal Chat (tc)

[![Rust CI](https://github.com/ShaneMarusczak/terminal_chat/actions/workflows/rust.yml/badge.svg?branch=main)](https://github.com/ShaneMarusczak/terminal_chat/actions/workflows/rust.yml)

Terminal Chat is a command line chat interface built in Rust. It allows interactive, conversational sessions with OpenAI’s language models. Users can chat in real time with streaming responses or perform various commands (e.g., clearing conversations, debugging, document and README generation, and more). The project emphasizes concise, accurate communication and provides a friendly environment for both casual and development-related conversations.

---

## Features

• Interactive CLI with real-time, streaming responses  
• Multiple command support
• Asynchronous networking with [reqwest](https://docs.rs/reqwest/) and [tokio](https://docs.rs/tokio/)  
• File processing for documentation: the tool can scan directories and aggregate project files to generate comprehensive README documentation  
• Built-in history and tab completion with [linefeed](https://docs.rs/linefeed/)

---

## Installation

1. **Prerequisites**  
   - Install [Rust](https://www.rust-lang.org/tools/install) (includes Cargo)  
   - Set your OpenAI API key as an environment variable:  
     - Linux/Mac: `export OPENAI_API_KEY=your_api_key_here`  
     - Windows (PowerShell): `$env:OPENAI_API_KEY="your_api_key_here"`

2. **Clone the Repository**  
   Run:  
   `git clone https://github.com/ShaneMaruszcak/terminal-chat.git`  
   `cd terminal-chat`

3. **Build the Project**  
   Use Cargo to build in debug or release mode:  
   - Debug: `cargo build`  
   - Release: `cargo build --release`

4. **Run the Application**  
   Execute with Cargo:  
   `cargo run --release`

---

## Usage Guide

• When the project is running, you will see a prompt:  
  `🗣️ `  
  - Simply type your message and press Enter to send.

• **Chat Interaction**  
   - Regular messages are sent directly to the chat service.  
   - The assistant’s responses are shown in a streaming manner with a spinner.

• **Commands**  
   - Prefix any command with a colon (`:`).
   - Run `:help` to see current commands.

• **Exiting**  
   - Type `:q` or `:quit` to exit the application.

---

## File and Structure Overview

• **src/main.rs**  
   - Contains the main entry point and command loop.  
   - Sets up the interactive CLI using the [linefeed](https://docs.rs/linefeed/) crate.

• **src/chat_client.rs**  
   - Responsible for making HTTP requests to OpenAI API endpoints.  
   - Implements both streaming and traditional request/response chat functions.

• **src/conversation.rs**  
   - Defines conversation context, message structure, and response data types.  
   - Manages message history and conversation state.

• **src/commands.rs**  
   - Implements support for chat and development commands (clear, debug, doc, change model, readme generation, etc.).  
   - Handles file operations and document/report generation.

• **src/spinner.rs**  
   - Provides a spinner animation while asynchronous operations (e.g., network requests) are in progress.

• **src/messages.rs**  
   - Contains default messages and prompts that guide conversation and report generation.

---

## Configuration Details

• **Environment Variable**  
   - `OPENAI_API_KEY`: Must be set for the chat client to authenticate with the OpenAI API.

• **Endpoints**  
   - API requests are made to:  
     `https://api.openai.com/v1/responses`  
     `https://api.openai.com/v1/chat/completions`

• **Async Runtime**  
   - The project utilizes Tokio’s asynchronous runtime, as specified by the `[tokio::main]` attribute in `main.rs`.

---

## Contribution Guidelines

Contributions are welcome! If you have ideas, feature enhancements, or bug fixes, please follow these steps:

1. Fork the repository.
2. Create a new branch for your feature or fix.
3. Commit your changes with clear, concise messages.
4. Open a pull request describing your changes.

---

## License

This project is licensed under the terms specified in the [LICENSE](LICENSE) file.

---

Happy chatting! Enjoy your interactive conversation with Terminal Chat.
