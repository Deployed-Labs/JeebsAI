
# JeebsAI

JeebsAI is a modular Rust-based AI assistant with a web UI and persistent storage.
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Project Structure

- `src/`
	- `main.rs` — Application entry point, web server, and CLI.
	- `admin/` — Admin features (user management, now in `admin/user/`).
	- `brain/` — Knowledge graph and training logic.
	- `auth/` — Authentication, registration, and password reset.
- `webui/` — Web user interface (HTML, JS, CSS).

## Modularity

All major features are separated into modules and submodules for maintainability:

- `admin::user` — Admin user management endpoints and types
- `brain` — Knowledge graph, training, and storage
- `auth` — Registration, login, password reset

## Running

1. Install Rust and Cargo.
2. Build and run:
	 ```sh
	 cargo run
	 ```
3. Access the web UI at [http://jeebs.club](http://jeebs.club)

### Running with Docker

1. Build the image:
   ```sh
   docker build -t jeebs .
   ```
2. Run the container (persisting data to a local `data` folder):
   ```sh
   docker run -d -p 8080:8080 -v $(pwd)/data:/data --name jeebs jeebs
   ```

## Development

- All business logic is modularized for easy extension.
- See each module for details and add new features in their own modules/submodules.

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for details.

### Issue Templates
When opening an issue, please use the provided templates:
- **Bug Report**: For reporting errors or unexpected behavior.
- **Feature Request**: For suggesting new ideas or improvements.

## Roadmap

- [ ] **v0.2.0**: Enhanced Plugin System with hot-reloading.
- [ ] **v0.3.0**: Distributed Brain (P2P knowledge sharing).
- [ ] **v1.0.0**: Full Self-Evolution capabilities enabled.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---
This project is modularized and ready for further extension.
