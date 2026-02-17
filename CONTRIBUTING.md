# Contributing to JeebsAI

Thank you for your interest in contributing to JeebsAI! We welcome contributions from the community to help make Jeebs smarter and more robust.

## Getting Started

1.  **Fork the repository** on GitHub.
2.  **Clone your fork** locally:
    ```bash
    git clone https://github.com/YOUR_USERNAME/JeebsAI-1.git
    cd JeebsAI-1
    ```
3.  **Install Prerequisites**:
    - [Rust](https://www.rust-lang.org/tools/install) (latest stable)
    - SQLite (development headers, e.g., `libsqlite3-dev` on Ubuntu)

## Development Workflow

1.  Create a new branch for your feature or bugfix:
    ```bash
    git checkout -b feature/my-awesome-feature
    ```
2.  Make your changes.
3.  Ensure the code compiles and tests pass:
    ```bash
    cargo test
    ```
4.  Run the linter to ensure code quality:
    ```bash
    cargo clippy -- -D warnings
    ```
5.  Commit your changes with clear messages.
6.  Push to your fork and submit a Pull Request.

## Project Structure

*   `src/main.rs`: Application entry point.
*   `src/cortex.rs`: The central processing logic ("thinking").
*   `src/brain/`: Core logic for knowledge graph and memory storage.
*   `src/auth/`: User authentication and management.
*   `src/plugins/`: Modular capabilities (add new skills here).
*   `webui/`: Frontend assets.

## Testing & CI

We use GitHub Actions for Continuous Integration. All pull requests must pass the automated test suite, which includes:
*   Unit tests (`cargo test`)
*   Linting (`cargo clippy`)
*   Security audit (`cargo audit`)

## Reporting Issues

If you find a bug or have a feature request, please open an issue on the GitHub repository describing the problem or idea in detail.