pub mod admin;
pub mod auth;
pub mod brain;
pub mod chat;
pub mod cli;
pub mod cortex;
pub mod evolution;
pub mod logging;
pub mod plugins;
pub mod security;
pub mod server;
pub mod state;
pub mod updater;
pub mod utils;

// Re-export AppState for convenience
pub use crate::state::AppState;
