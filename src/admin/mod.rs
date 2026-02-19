pub mod blacklist;
pub mod whitelist;

// Only export the specific items needed to avoid conflicts
pub use blacklist::IpRequest as BlacklistRequest;
pub use whitelist::IpRequest as WhitelistRequest;
