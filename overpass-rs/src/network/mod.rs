// src/network/mod.rs
pub mod discovery;
pub mod messages;
pub mod network_interface;
pub mod peer;
pub mod protocol;
pub mod sync_manager;
pub mod transport;
// re-exporting the modules for convenience
pub use discovery::*;
pub use messages::*;
pub use network_interface::*;
pub use peer::*;
pub use sync_manager::*;
pub use transport::*;
