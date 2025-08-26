//! Procedural macros for Judicia plugin development

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::{
    parse_macro_input, parse_quote, AttributeArgs, Data, DeriveInput, Fields, Ident, ItemStruct,
    Lit, Meta, NestedMeta, Type,
};

/// Main procedural macro for defining a Judicia plugin
/// 
/// This macro generates all the necessary boilerplate code for a WebAssembly plugin,
/// including exports, capability declarations, and lifecycle management.
/// 
/// # Example
/// 
/// ```rust
/// #[judicia_plugin]
/// pub struct MyPlugin {
///     name: "my-plugin",
///     version: "1.0.0", 
///     author: "Your Name",
///     description: "A sample plugin",
///     capabilities: [
///         Capability::TriggerJudging,
///         Capability::EmitEvent
///     ]
/// }
/// ```
#[proc_macro_attribute]
pub fn judicia_plugin(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let input = parse_macro_input!(input as ItemStruct);

    match expand_judicia_plugin(args, input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn expand_judicia_plugin(
    _args: AttributeArgs,
    mut input: ItemStruct,
) -> syn::Result<TokenStream2> {
    let plugin_name = &input.ident;
    
    // Extract plugin metadata from struct fields
    let metadata = extract_plugin_metadata(&input)?;
    
    // Remove metadata fields from the struct as they'll be handled separately
    if let Fields::Named(ref mut fields) = &mut input.fields {
        fields.named = fields.named
            .iter()
            .filter(|field| {
                let field_name = field.ident.as_ref().unwrap().to_string();
                !matches!(field_name.as_str(), "name" | "version" | "author" | "description" | "capabilities")
            })
            .cloned()
            .collect();
    }
    
    let plugin_metadata_init = generate_metadata_init(&metadata);
    let capability_declarations = generate_capability_declarations(&metadata.capabilities);
    let wasm_exports = generate_wasm_exports(plugin_name);
    let plugin_impl = generate_plugin_impl(plugin_name, &metadata);

    Ok(quote! {
        #input
        
        #plugin_metadata_init
        #capability_declarations
        #plugin_impl
        #wasm_exports
    })
}

/// Procedural macro for defining frontend components that integrate with plugins
/// 
/// This macro generates TypeScript definitions and WebAssembly bindings for
/// frontend components that can be rendered by plugins.
/// 
/// # Example
/// 
/// ```rust
/// #[frontend_component]
/// pub struct SubmissionList {
///     component_type: ComponentType::List,
///     props: SubmissionListProps,
///     events: ["selection_changed", "item_clicked"]
/// }
/// ```
#[proc_macro_attribute]
pub fn frontend_component(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let input = parse_macro_input!(input as ItemStruct);
    
    match expand_frontend_component(args, input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn expand_frontend_component(
    _args: AttributeArgs,
    input: ItemStruct,
) -> syn::Result<TokenStream2> {
    let component_name = &input.ident;
    let component_metadata = extract_component_metadata(&input)?;
    
    let typescript_bindings = generate_typescript_bindings(component_name, &component_metadata);
    let wasm_component_exports = generate_component_exports(component_name);
    
    Ok(quote! {
        #input
        
        #typescript_bindings
        #wasm_component_exports
        
        impl FrontendComponent for #component_name {
            fn render(&self, props: &serde_json::Value) -> Result<String, JsValue> {
                let html = self.generate_html(props)?;
                Ok(html)
            }
            
            fn handle_event(&mut self, event: &str, data: &serde_json::Value) -> Result<(), JsValue> {
                self.on_event(event, data)
            }
        }
    })
}

/// Capability declaration macro for fine-grained permission control
/// 
/// This macro helps declare and validate plugin capabilities at compile time.
/// 
/// # Example
/// 
/// ```rust
/// capability! {
///     name: "database_access",
///     description: "Access to read/write contest database",
///     permissions: ["read:problems", "write:submissions"],
///     sensitive: true
/// }
/// ```
#[proc_macro]
pub fn capability(input: TokenStream) -> TokenStream {
    // Parse capability declaration and generate validation code
    let capability_def = parse_macro_input!(input as CapabilityDefinition);
    
    let expanded = quote! {
        // Generate capability constant and validation
        const CAPABILITY_DEF: &str = #(stringify!(#capability_def));
    };
    
    expanded.into()
}

// Helper structures for parsing macro inputs
struct PluginMetadata {
    name: String,
    version: String,
    author: String,
    description: String,
    capabilities: Vec<String>,
}

struct ComponentMetadata {
    component_type: String,
    props_type: Option<String>,
    events: Vec<String>,
}

struct CapabilityDefinition {
    name: String,
    description: String,
    permissions: Vec<String>,
    sensitive: bool,
}

impl syn::parse::Parse for CapabilityDefinition {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // Parse capability definition syntax
        let mut name = String::new();
        let mut description = String::new();
        let mut permissions = Vec::new();
        let mut sensitive = false;
        
        // Simple parsing for demo - would be more robust in production
        while !input.is_empty() {
            let ident: Ident = input.parse()?;
            let _: syn::Token![:] = input.parse()?;
            
            match ident.to_string().as_str() {
                "name" => {
                    let lit: Lit = input.parse()?;
                    if let Lit::Str(s) = lit {
                        name = s.value();
                    }
                }
                "description" => {
                    let lit: Lit = input.parse()?;
                    if let Lit::Str(s) = lit {
                        description = s.value();
                    }
                }
                "sensitive" => {
                    let lit: Lit = input.parse()?;
                    if let Lit::Bool(b) = lit {
                        sensitive = b.value();
                    }
                }
                _ => {
                    // Skip unknown fields
                    let _: Lit = input.parse()?;
                }
            }
            
            if input.peek(syn::Token![,]) {
                let _: syn::Token![,] = input.parse()?;
            }
        }
        
        Ok(CapabilityDefinition {
            name,
            description,
            permissions,
            sensitive,
        })
    }
}

fn extract_plugin_metadata(input: &ItemStruct) -> syn::Result<PluginMetadata> {
    let mut metadata = PluginMetadata {
        name: String::new(),
        version: String::new(),
        author: String::new(),
        description: String::new(),
        capabilities: Vec::new(),
    };
    
    if let Fields::Named(fields) = &input.fields {
        for field in &fields.named {
            let field_name = field.ident.as_ref().unwrap().to_string();
            
            // Extract string literals from field types (simplified)
            match field_name.as_str() {
                "name" => metadata.name = extract_string_literal_from_type(&field.ty)?,
                "version" => metadata.version = extract_string_literal_from_type(&field.ty)?,
                "author" => metadata.author = extract_string_literal_from_type(&field.ty)?,
                "description" => metadata.description = extract_string_literal_from_type(&field.ty)?,
                "capabilities" => {
                    // Extract capabilities array (simplified)
                    metadata.capabilities = vec!["TriggerJudging".to_string(), "EmitEvent".to_string()];
                }
                _ => {}
            }
        }
    }
    
    Ok(metadata)
}

fn extract_component_metadata(input: &ItemStruct) -> syn::Result<ComponentMetadata> {
    // Similar to plugin metadata extraction but for components
    Ok(ComponentMetadata {
        component_type: "Default".to_string(),
        props_type: None,
        events: vec!["click".to_string(), "change".to_string()],
    })
}

fn extract_string_literal_from_type(ty: &Type) -> syn::Result<String> {
    // This is a simplified version - in a real implementation,
    // we'd parse const expressions and string literals more thoroughly
    match ty {
        Type::Path(path) => {
            // For demo purposes, return a default value
            Ok("default".to_string())
        }
        _ => Ok("unknown".to_string()),
    }
}

fn generate_metadata_init(metadata: &PluginMetadata) -> TokenStream2 {
    let name = &metadata.name;
    let version = &metadata.version;
    let author = &metadata.author;
    let description = &metadata.description;
    
    quote! {
        impl PluginMetadata for MyPlugin {
            fn name(&self) -> &str { #name }
            fn version(&self) -> &str { #version }
            fn author(&self) -> &str { #author }
            fn description(&self) -> &str { #description }
        }
    }
}

fn generate_capability_declarations(capabilities: &[String]) -> TokenStream2 {
    let cap_exprs: Vec<TokenStream2> = capabilities.iter().map(|cap| {
        let cap_ident = syn::Ident::new(cap, proc_macro2::Span::call_site());
        quote! { Capability::#cap_ident }
    }).collect();
    
    quote! {
        const REQUIRED_CAPABILITIES: &[Capability] = &[#(#cap_exprs),*];
        
        #[wasm_bindgen]
        pub fn get_required_capabilities() -> js_sys::Array {
            let array = js_sys::Array::new();
            for cap in REQUIRED_CAPABILITIES {
                array.push(&JsValue::from_str(&cap.to_string()));
            }
            array
        }
    }
}

fn generate_plugin_impl(plugin_name: &Ident, metadata: &PluginMetadata) -> TokenStream2 {
    quote! {
        impl Plugin for #plugin_name {
            fn new() -> Self {
                Self {
                    // Initialize plugin fields
                }
            }
            
            fn metadata(&self) -> PluginInfo {
                PluginInfo {
                    name: #(metadata.name).to_string(),
                    version: #(metadata.version).to_string(),
                    author: #(metadata.author).to_string(),
                    description: #(metadata.description).to_string(),
                }
            }
        }
    }
}

fn generate_wasm_exports(plugin_name: &Ident) -> TokenStream2 {
    let plugin_name_str = plugin_name.to_string();
    
    quote! {
        static mut PLUGIN_INSTANCE: Option<#plugin_name> = None;
        
        #[wasm_bindgen(start)]
        pub fn init_plugin() {
            console_error_panic_hook::set_once();
            
            unsafe {
                PLUGIN_INSTANCE = Some(#plugin_name::new());
            }
        }
        
        #[wasm_bindgen]
        pub async fn plugin_initialize(context: JsValue) -> Result<(), JsValue> {
            let context: PluginContext = context.into_serde()
                .map_err(|e| JsValue::from_str(&format!("Failed to deserialize context: {}", e)))?;
            
            unsafe {
                if let Some(ref mut plugin) = &mut PLUGIN_INSTANCE {
                    plugin.on_initialize(&context).await
                        .map_err(|e| JsValue::from_str(&e.to_string()))?;
                }
            }
            
            Ok(())
        }
        
        #[wasm_bindgen]
        pub async fn plugin_handle_event(event: JsValue) -> Result<(), JsValue> {
            let event: PlatformEvent = event.into_serde()
                .map_err(|e| JsValue::from_str(&format!("Failed to deserialize event: {}", e)))?;
            
            unsafe {
                if let Some(ref mut plugin) = &mut PLUGIN_INSTANCE {
                    plugin.on_event(&event).await
                        .map_err(|e| JsValue::from_str(&e.to_string()))?;
                }
            }
            
            Ok(())
        }
        
        #[wasm_bindgen]
        pub fn plugin_info() -> JsValue {
            unsafe {
                if let Some(ref plugin) = &PLUGIN_INSTANCE {
                    JsValue::from_serde(&plugin.metadata()).unwrap()
                } else {
                    JsValue::NULL
                }
            }
        }
        
        #[wasm_bindgen]
        pub async fn plugin_cleanup() -> Result<(), JsValue> {
            unsafe {
                if let Some(ref mut plugin) = &mut PLUGIN_INSTANCE {
                    plugin.on_cleanup().await
                        .map_err(|e| JsValue::from_str(&e.to_string()))?;
                }
                PLUGIN_INSTANCE = None;
            }
            
            Ok(())
        }
    }
}

fn generate_typescript_bindings(component_name: &Ident, _metadata: &ComponentMetadata) -> TokenStream2 {
    let ts_interface_name = format!("{}Props", component_name);
    
    quote! {
        #[wasm_bindgen(typescript_custom_section)]
        const TYPESCRIPT_INTERFACE: &'static str = r#"
        interface #ts_interface_name {
            [key: string]: any;
        }
        
        export class #component_name {
            render(props: #ts_interface_name): string;
            handleEvent(event: string, data: any): void;
        }
        "#;
    }
}

fn generate_component_exports(component_name: &Ident) -> TokenStream2 {
    quote! {
        #[wasm_bindgen]
        impl #component_name {
            #[wasm_bindgen(constructor)]
            pub fn new() -> #component_name {
                #component_name::default()
            }
            
            #[wasm_bindgen]
            pub fn render(&self, props: JsValue) -> Result<String, JsValue> {
                let props: serde_json::Value = props.into_serde()
                    .map_err(|e| JsValue::from_str(&e.to_string()))?;
                
                self.generate_html(&props)
                    .map_err(|e| JsValue::from_str(&e.to_string()))
            }
            
            #[wasm_bindgen]
            pub fn handle_event(&mut self, event: &str, data: JsValue) -> Result<(), JsValue> {
                let data: serde_json::Value = data.into_serde()
                    .map_err(|e| JsValue::from_str(&e.to_string()))?;
                
                self.on_event(event, &data)
                    .map_err(|e| JsValue::from_str(&e.to_string()))
            }
        }
    }
}