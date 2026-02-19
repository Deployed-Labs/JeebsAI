# Contributing to JeebsAI

Thank you for your interest in contributing to JeebsAI! This document provides guidelines and instructions for contributing to the project.

## Table of Contents

- [Getting Started](#getting-started)
- [Development Workflow](#development-workflow)
- [Code Standards](#code-standards)
- [Testing](#testing)
- [Pull Request Process](#pull-request-process)
- [CI/CD Pipeline](#cicd-pipeline)

## Getting Started

### Prerequisites

1. **Install Rust and Cargo:**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source $HOME/.cargo/env
   ```

   > **Note:** If you are using VS Code, you can use the provided Dev Container configuration to set up your environment automatically.

2. **Install system dependencies:**
   ```bash
   # Ubuntu/Debian
   sudo apt update
   sudo apt install -y build-essential pkg-config libssl-dev sqlite3
   
   # macOS
   brew install sqlite
   ```

3. **Fork and clone the repository:**
   ```bash
   git clone https://github.com/YOUR-USERNAME/JeebsAI.git
   cd JeebsAI
   ```

4. **Set up your environment:**
   ```bash
   # Copy the example environment file
   cp .env.example .env

   # Edit .env if needed for local development
   ```

4.1 **Install Git hooks (optional — recommended)**

To enable the local git hook that automatically bumps the patch version on every commit (useful for single‑developer workflows), run:

```bash
./scripts/install-git-hooks.sh
```

The hook commits a bump (with `[skip ci]`) and will be skipped for bump commits themselves.

5. **Build and run:**
   ```bash
   cargo build
   cargo run
   ```

## Development Workflow

1. **Create a feature branch:**
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes:**
   - Follow the project structure and modularity principles
   - Add new features in their own modules/submodules
   - Write tests for new functionality

3. **Test your changes locally:**
   ```bash
   # Run all tests
   cargo test
   
   # Check formatting
   cargo fmt -- --check
   
   # Run clippy for linting
   cargo clippy -- -D warnings
   
   # Build in release mode
   cargo build --release
   ```

4. **Commit your changes:**
   ```bash
   git add .
   git commit -m "Brief description of changes"
   ```

5. **Push to your fork:**
   ```bash
   git push origin feature/your-feature-name
   ```

6. **Open a Pull Request:**
   - Go to the original repository on GitHub
   - Click "New Pull Request"
   - Select your branch
   - Fill out the PR template

## Code Standards

### Rust Code Style

- Follow the official [Rust Style Guide](https://doc.rust-lang.org/nightly/style-guide/)
- Use `cargo fmt` to format code automatically
- All code must pass `cargo clippy` without warnings

### Code Organization

- Keep modules focused and single-purpose
- Place new features in appropriate modules:
  - `src/admin/` - Administrative features
  - `src/auth/` - Authentication and authorization
  - `src/brain/` - Knowledge graph and AI logic
  - `src/cortex/` - Central processing and intent routing
  - `plugins/` - Polyglot plugins (Python, Node, etc.)
  - Create new modules for distinct feature areas

### Documentation

- Add doc comments (`///`) for public functions and modules
- Update the README if adding user-facing features
- Include examples in doc comments when helpful

### Plugin Development

JeebsAI supports a polyglot plugin model. Plugins are located in `plugins/<plugin-name>/`.

Each plugin must provide a runner executable:
- `run` (binary executable)
- `run.py` (Python)
- `run.js` or `index.js` (Node.js)

**Contract:**
The runner must read a JSON object from `stdin` containing an `input` field:
```json
{ "input": "..." }
```
And write a JSON object to `stdout` containing a `response` field:
```json
{ "response": "..." }
```

### Commit Messages

Follow the conventional commits format:

- `feat:` New features
- `fix:` Bug fixes
- `docs:` Documentation changes
- `style:` Code style changes (formatting, etc.)
- `refactor:` Code refactoring
- `test:` Adding or updating tests
- `chore:` Maintenance tasks

Example:
```
feat: add user profile endpoint

- Add GET /api/user/profile endpoint
- Include user statistics in response
- Add tests for profile retrieval
```

## Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture

# Run tests in release mode
cargo test --release
```

### Writing Tests

- Place unit tests in the same file as the code being tested
- Use the `#[cfg(test)]` module convention
- Place integration tests in `tests/` directory
- Aim for meaningful test names that describe what is being tested

Example:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_creation() {
        // Test implementation
    }
}
```

## Pull Request Process

1. **Ensure all tests pass:**
   ```bash
   cargo test
   cargo fmt -- --check
   cargo clippy -- -D warnings
   ```

2. **Update documentation:**
   - Update README.md if you changed user-facing features
   - Add/update doc comments for new code
   - Update CHANGELOG.md if applicable

3. **Create a clear PR description:**
   - Describe what changes you made
   - Explain why the changes are necessary
   - List any breaking changes
   - Reference any related issues

4. **Wait for CI checks:**
   - All CI checks must pass before merge
   - Address any issues found by automated checks

5. **Respond to review feedback:**
   - Be open to suggestions and constructive criticism
   - Make requested changes in new commits
   - Push updates to the same branch

6. **Merge:**
   - Once approved and all checks pass, a maintainer will merge your PR
   - Delete your feature branch after merge

## CI/CD Pipeline

### Continuous Integration

Every push and pull request triggers automated checks:

1. **Code Formatting** - Ensures code follows Rust formatting standards
2. **Linting** - Checks for common mistakes and style issues
3. **Build** - Verifies the project compiles
4. **Tests** - Runs all unit and integration tests
5. **Security Audit** - Checks for known vulnerabilities in dependencies
6. **Secret Scan** - Scans for accidental secret/token commits

### Deployment

Merges to the `main` branch automatically trigger deployment to production:

1. Code is transferred to the VPS
2. Release binary is built
3. Service is restarted
4. Deployment is verified

See [.github/GITHUB_ACTIONS_SETUP.md](.github/GITHUB_ACTIONS_SETUP.md) for details on the CI/CD configuration.

## Getting Help

- **Questions?** Open a discussion in GitHub Discussions
- **Bug?** Open an issue with steps to reproduce
- **Feature idea?** Open an issue to discuss before implementing

## Code of Conduct

- Be respectful and inclusive
- Focus on constructive feedback
- Help create a welcoming environment for all contributors

---

Thank you for contributing to JeebsAI! 🚀
