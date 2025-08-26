use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, ItemStruct};

/// Main entry point macro for plugin development
#[proc_macro_attribute]
pub fn main(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);
    let fn_name = &input_fn.sig.ident;
    
    let expanded = quote! {
        #input_fn
        
        // Export the main function for WASM
        #[no_mangle]
        pub extern "C" fn _plugin_main() -> i32 {
            match #fn_name() {
                Ok(_) => 0,
                Err(_) => 1,
            }
        }
    };
    
    TokenStream::from(expanded)
}

/// HTTP handler macro for plugin endpoints
#[proc_macro_attribute]
pub fn handler(attr: TokenStream, item: TokenStream) -> TokenStream {
    let _path = parse_macro_input!(attr as syn::LitStr);
    let input_fn = parse_macro_input!(item as ItemFn);
    
    let expanded = quote! {
        #input_fn
        
        // TODO: Register handler with plugin runtime
    };
    
    TokenStream::from(expanded)
}

/// Event handler macro for subscribing to events
#[proc_macro_attribute]
pub fn event_handler(attr: TokenStream, item: TokenStream) -> TokenStream {
    let _event_pattern = parse_macro_input!(attr as syn::LitStr);
    let input_fn = parse_macro_input!(item as ItemFn);
    
    let expanded = quote! {
        #input_fn
        
        // TODO: Register event handler
    };
    
    TokenStream::from(expanded)
}

/// Plugin metadata macro
#[proc_macro_attribute]
pub fn plugin_metadata(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_struct = parse_macro_input!(item as ItemStruct);
    let struct_name = &input_struct.ident;
    
    let expanded = quote! {
        #input_struct
        
        // Export plugin metadata
        #[no_mangle]
        pub extern "C" fn _plugin_metadata() -> *const u8 {
            let metadata = #struct_name::default();
            // TODO: Serialize metadata
            std::ptr::null()
        }
        
        #[no_mangle]
        pub extern "C" fn _plugin_metadata_len() -> usize {
            0
        }
    };
    
    TokenStream::from(expanded)
}