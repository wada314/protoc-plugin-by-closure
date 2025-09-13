//! Proxy crate for managing protoc-plugin-bin dependency
//!
//! This crate provides a clean interface to access the protoc-plugin-bin binary
//! while allowing conditional compilation based on feature flags.

/// Get the plugin binary path
/// 
/// Returns the path to the protoc-plugin-bin binary when the nightly feature is enabled,
/// or an error when the feature is disabled.
pub fn get_plugin_path() -> Result<&'static str, &'static str> {
    #[cfg(feature = "nightly")]
    {
        Ok(env!("CARGO_BIN_FILE_PROTOC_PLUGIN_BIN"))
    }
    #[cfg(not(feature = "nightly"))]
    {
        Err("protoc-plugin-bin binary is not available. Please enable the 'nightly' feature.")
    }
}

/// Check if the binary dependency is available
pub fn has_binary() -> bool {
    get_plugin_path().is_ok()
}
