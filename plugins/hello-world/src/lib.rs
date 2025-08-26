use plugin_sdk::main;

#[main]
fn plugin_main() -> Result<(), i32> {
    // Plugin initialization - simple function that can be called from WASM
    Ok(()) // Success
}

// Simple handler function that can be called from the host
#[no_mangle]
pub extern "C" fn handle_request(args_len: i32, _args_ptr: i32) -> i32 {
    // Simple plugin logic - just return success
    args_len // Echo back the args length as a simple response
}

#[no_mangle]
pub extern "C" fn get_info() -> i32 {
    // Return plugin info
    42 // Magic number indicating this is the hello-world plugin
}
