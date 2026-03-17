use bot_api::script::BotScript;

/// The ABI contract for script plugins.
///
/// Script dynamic libraries must export these symbols:
///
/// ```c
/// // Creates a new instance of the script. The caller owns the returned pointer.
/// extern "C" fn _create_script() -> *mut dyn BotScript;
/// ```
///
/// In Rust, scripts implement this by:
/// ```rust,ignore
/// #[no_mangle]
/// pub extern "C" fn _create_script() -> *mut dyn BotScript {
///     let script = MyScript::new();
///     let boxed: Box<dyn BotScript> = Box::new(script);
///     Box::into_raw(boxed)
/// }
/// ```
///
/// SAFETY: The host and script must be compiled with the same Rust compiler version
/// and the same version of the `bot-api` crate. Trait object layout is not stable
/// across compiler versions.

/// Type signature of the plugin creation function.
#[allow(improper_ctypes_definitions)]
pub type CreateScriptFn = unsafe extern "C" fn() -> *mut dyn BotScript;

/// The symbol name that plugin libraries must export.
pub const CREATE_SCRIPT_SYMBOL: &[u8] = b"_create_script";

/// A loaded plugin holding the library handle and the script factory.
pub struct LoadedPlugin {
    /// The dynamic library handle. Must be kept alive as long as the script is in use.
    pub library: libloading::Library,
    /// The name of the library file (for logging).
    pub file_name: String,
}

impl LoadedPlugin {
    /// Create a new script instance from this plugin.
    ///
    /// # Safety
    /// The caller must ensure the plugin was compiled with a compatible compiler version
    /// and API crate version.
    pub unsafe fn create_script(&self) -> Result<Box<dyn BotScript>, PluginError> {
        let create_fn: libloading::Symbol<CreateScriptFn> = self
            .library
            .get(CREATE_SCRIPT_SYMBOL)
            .map_err(|e| PluginError::SymbolNotFound(e.to_string()))?;

        let raw = create_fn();
        if raw.is_null() {
            return Err(PluginError::NullScript);
        }

        Ok(Box::from_raw(raw))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    #[error("Failed to load library: {0}")]
    LoadFailed(String),
    #[error("Required symbol '_create_script' not found: {0}")]
    SymbolNotFound(String),
    #[error("Plugin returned null script")]
    NullScript,
}
