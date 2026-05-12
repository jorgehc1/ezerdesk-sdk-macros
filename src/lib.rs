extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn main(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);
    let fn_name = &input_fn.sig.ident;

    let expanded = quote! {
        #input_fn

        #[unsafe(no_mangle)]
        pub extern "C" fn alloc(size: usize) -> *mut u8 {
            ::ezerdesk_sdk::allocate(size)
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn deallocate(ptr: *mut u8, size: usize) {
            ::ezerdesk_sdk::deallocate(ptr, size);
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn on_event(ptr: *const u8, len: usize) -> i32 {
            // Protección contra desbordamientos y punteros nulos
            if ptr.is_null() || len == 0 {
                return -1;
            }

            let input = unsafe { 
                let slice = std::slice::from_raw_parts(ptr, len);
                String::from_utf8_lossy(slice).to_string()
            };

            let event_res: Result<::ezerdesk_sdk::PluginEvent, _> = serde_json::from_str(&input);
            
            match event_res {
                Ok(event) => {
                    // Ejecutar la lógica del usuario
                    #fn_name(event)
                }
                Err(e) => {
                    // Log del error en el host para diagnóstico
                    ::ezerdesk_sdk::log(&format!("❌ [SDK]: Error decodificando evento: {}", e));
                    -1
                }
            }
        }
    };

    TokenStream::from(expanded)
}
