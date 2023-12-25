**IMPORTANT:** For the YouTube Video [Coding a Rust OpenAI Assistant CLI from scratch](https://youtu.be/PHbCmIckV20), be sure to refer to the [GitHub Tag E01](https://github.com/rust10x/rust-ai-buddy/tree/E01) (see note below).

# Rust AI-Buddy

The vision for Rust AI-Buddy is to create a simple on-device AI assistant that leverages AI assistant services such as OpenAI (Cloud), Gemini (Cloud), ollama (local), lamafile (local), among others. It aims to demonstrate best practices in Rust coding.

## YouTube Videos

- E01 [GitHub Tag E01](https://github.com/rust10x/rust-ai-buddy/tree/E01) - [Coding a Rust OpenAI Assistant CLI from scratch](https://youtu.be/PHbCmIckV20)

**NOTE:** The `main` branch has been updated with a `* MAJOR - Refactoring multiple crates` commit and other significant upgrades.

## Cargo Commands

```sh
# Build everything
cargo build

# Run the command line
cargo run -p ai-buddy-cli

# Install the `buddy` command line locally
cargo install --path crates/ai-buddy-cli 
```

## Context

Rust AI-Buddy is part of the Rust10x blueprint family (https://github.com/rust10x).

The concept of Rust AI-Buddy is to leverage existing AI remote and cloud services like OpenAI (Cloud), Gemini (Cloud), ollama (local), lamafile (local), and others as it makes sense, and build higher-level constructs that are useful for end users.

`ai-buddy` is a multi-crate codebase that includes the following crates:

| Crate Name             | Description                                |
|------------------------|--------------------------------------------|
| `/crates/ai-buddy`     | The core/main library used by ai-buddy-cli |
| `/crates/ai-buddy-cli` | The CLI for ai-buddy (`buddy` binary name) |
| `/crates/ai-buddy-app` | (Upcoming) Tauri App                       |

<br />

## Repository

[GitHub Repository](https://github.com/rust10x/rust-ai-buddy)
