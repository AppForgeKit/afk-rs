#[cfg(feature = "default")]
pub mod log;

#[cfg(feature = "types")]
pub use tracing::{debug, error, info, trace, warn};

#[cfg(feature = "types")]
pub mod types {
    pub type Level = tracing::Level;
}
