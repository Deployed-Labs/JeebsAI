
# JeebsAI

JeebsAI is a modular Rust-based AI assistant with a web UI and persistent storage.

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
3. Access the web UI at [http://localhost:8080](http://localhost:8080)

## Development

- All business logic is modularized for easy extension.
- See each module for details and add new features in their own modules/submodules.

---
This project is modularized and ready for further extension.
