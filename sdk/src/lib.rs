pub use spin_mcp_macro::mcp_component;

#[doc(hidden)]
pub mod wit {
    #![allow(missing_docs)]

    wit_bindgen::generate!({
        world: "spin-mcp-sdk",
        path: "..",
    });
}

#[doc(hidden)]
pub use wit_bindgen;

#[doc(inline)]
pub use wit::spin::mcp_trigger::mcp_types::{
    Error, Request, Response, Tool, ToolResult, Prompt, PromptArgument, 
    PromptMessage, ResourceInfo, ResourceContents,
};