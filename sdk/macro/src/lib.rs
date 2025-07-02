use proc_macro::TokenStream;
use quote::quote;

const WIT_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../spin-mcp.wit");

#[proc_macro_attribute]
pub fn mcp_component(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = syn::parse_macro_input!(item as syn::ItemFn);
    let func_name = &func.sig.ident;
    let preamble = preamble();

    quote!(
        #func
        mod __spin_mcp {
            mod preamble {
                #preamble
            }
            impl self::preamble::Guest for preamble::Mcp {
                fn handle_request(request: ::spin_mcp_sdk::Request) -> ::spin_mcp_sdk::Response {
                    super::#func_name(request)
                }
                
                fn initialize() -> ::std::result::Result<(), ::std::string::String> {
                    ::std::result::Result::Ok(())
                }
            }
        }
    ).into()
}

fn preamble() -> proc_macro2::TokenStream {
    let world = "spin-mcp";
    quote! {
        #![allow(missing_docs)]
        ::spin_mcp_sdk::wit_bindgen::generate!({
            world: #world,
            path: #WIT_PATH,
            runtime_path: "::spin_mcp_sdk::wit_bindgen::rt",
            exports: {
                world: Mcp
            },
            with: {
                "spin:mcp-trigger/mcp-types": ::spin_mcp_sdk::wit::spin::mcp_trigger::mcp_types,
            }
        });
        pub struct Mcp;
    }
}