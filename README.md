# Terminal Chat

A Rust-based console application for chatting with OpenAI’s language models right in your terminal.

## Overview

This application remembers conversation context and offers specialized commands:
- “:doc” automatically creates a Markdown report of the conversation.
- “:gf” (get file) uploads file content into the conversation context.

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

## Commands

Available commands:

- :clear – Clears the conversation context.
- :debug – Prints current conversation model and messages for debugging.
- :doc – Generates a Markdown report of the conversation and offers to save it.
- :cm – Changes the active chat model.
- :help – Displays this help information.
- :gf <path> – Appends the content of the specified file to the conversation.
- :rmr – Launches rmr if it’s installed in your system’s PATH.[rmr](https://github.com/ShaneMarusczak/rm-repl)

## Contributing

Open an issue or submit a pull request to share ideas or improvements.
