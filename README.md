# basic-agent

A minimal AI coding agent built in Rust that uses Claude (via OpenRouter) to execute tasks by reading files, writing files, and running shell commands.

## How It Works

The agent runs in a loop:

1. Sends your prompt to Claude Haiku 4.5 via the OpenRouter API
2. The model can call three tools:
   - **Bash** — execute a shell command
   - **Read** — read a file's contents
   - **Write** — write content to a file
3. Tool results are fed back to the model, and the loop continues until the model responds with a final text answer

## Prerequisites

- Rust 1.93+
- An [OpenRouter](https://openrouter.ai/) API key

## Setup

```sh
export OPENROUTER_API_KEY="your-api-key"
```

Optionally override the base URL:

```sh
export OPENROUTER_BASE_URL="https://openrouter.ai/api/v1"
```

## Build & Run

```sh
cargo build
cargo run -- -p "your prompt here"
```

## Example

```sh
cargo run -- -p "List all Rust files in the current directory and summarize what they do"
```
