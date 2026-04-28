#[cfg(any(feature = "builder", feature = "macros"))]
pub mod proto {
    #[cfg(feature = "builder")]
    pub use tonic_prost_build::configure;

    #[cfg(feature = "macros")]
    pub use tonic::{include_file_descriptor_set, include_proto};
}
