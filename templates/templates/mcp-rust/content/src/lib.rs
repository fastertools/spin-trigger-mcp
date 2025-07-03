use spin_mcp_sdk::{mcp_component, Request, Response, Error};

mod tools;
mod resources;
mod prompts;

#[cfg(target_family = "wasm")]
#[global_allocator]
static ALLOC: talc::Talck<talc::locking::AssumeUnlockable, talc::ClaimOnOom> = {
    use talc::*;
    // Choose an arena size based on your tool's needs.
    // - 64 * 1024 (64KB): For minimal tools (e.g., simple text processing).
    // - 1 * 1024 * 1024 (1MB): A good default for most tools.
    // - 4 * 1024 * 1024 (4MB): For data-intensive tools (e.g., image processing).
    const ARENA_SIZE: usize = 1 * 1024 * 1024; // Default: 1MB
    static mut ARENA: [u8; ARENA_SIZE] = [0; ARENA_SIZE];
    Talc::new(unsafe {
        ClaimOnOom::new(Span::from_base_size(
            std::ptr::addr_of_mut!(ARENA).cast(),
            ARENA_SIZE,
        ))
    })
    .lock()
};


#[mcp_component]
fn handle_request(request: Request) -> Response {
    match request {
        Request::ToolsList => {
            Response::ToolsList(tools::get_tools_list())
        }
        
        Request::ToolsCall(params) => {
            Response::ToolsCall(tools::handle_tool_call(&params.name, &params.arguments))
        }
        
        Request::ResourcesList => {
            Response::ResourcesList(resources::get_resources_list())
        }
        
        Request::PromptsList => {
            Response::PromptsList(prompts::get_prompts_list())
        }
        
        Request::Ping => Response::Pong,
        
        _ => Response::Error(Error {
            code: -32601,
            message: "Method not found".to_string(),
            data: None,
        })
    }
}