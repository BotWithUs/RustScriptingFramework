use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

/// Attribute macro that generates the FFI export functions required for
/// a script plugin. Apply to a struct that implements `BotScript`.
///
/// # Usage
/// ```rust,ignore
/// #[export_script]
/// pub struct MyScript { ... }
/// ```
///
/// This generates:
/// ```rust,ignore
/// #[no_mangle]
/// pub extern "C" fn _create_script() -> *mut dyn bot_api::script::BotScript {
///     let script = MyScript::default();
///     let boxed: Box<dyn bot_api::script::BotScript> = Box::new(script);
///     Box::into_raw(boxed)
/// }
/// ```
#[proc_macro_attribute]
pub fn export_script(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let name = &input.ident;

    let expanded = quote! {
        #input

        #[no_mangle]
        pub extern "C" fn _create_script() -> *mut dyn bot_api::script::BotScript {
            let script = #name::default();
            let boxed: Box<dyn bot_api::script::BotScript> = Box::new(script);
            Box::into_raw(boxed)
        }
    };

    TokenStream::from(expanded)
}

/// Helper macro to create a `ScriptManifest` inline.
///
/// # Usage
/// ```rust,ignore
/// fn manifest(&self) -> ScriptManifest {
///     script_manifest!(
///         name: "My Script",
///         version: "1.0",
///         author: "Dev",
///         description: "Does things"
///     )
/// }
/// ```
#[proc_macro]
pub fn script_manifest(input: TokenStream) -> TokenStream {
    let input_str = input.to_string();

    // Parse key: "value" pairs
    let mut name = String::new();
    let mut version = String::new();
    let mut author = String::new();
    let mut description = String::new();

    for part in input_str.split(',') {
        let part = part.trim();
        if let Some((key, val)) = part.split_once(':') {
            let key = key.trim();
            let val = val.trim().trim_matches('"');
            match key {
                "name" => name = val.to_string(),
                "version" => version = val.to_string(),
                "author" => author = val.to_string(),
                "description" => description = val.to_string(),
                _ => {}
            }
        }
    }

    let expanded = quote! {
        bot_api::script::ScriptManifest {
            name: #name.to_string(),
            version: #version.to_string(),
            author: #author.to_string(),
            description: #description.to_string(),
        }
    };

    TokenStream::from(expanded)
}
