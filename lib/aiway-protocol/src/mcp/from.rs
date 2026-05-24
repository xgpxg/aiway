use crate::mcp::mcp::McpTool;
use rmcp::model::{Tool, object};
use serde_json::{Value, json};
use std::sync::{Arc, LazyLock};

static EMPTY_INPUT_SCHEMA: LazyLock<Value> = LazyLock::new(|| {
    json!({
      "type": "object",
      "properties": {}
    })
});
impl From<McpTool> for Tool {
    fn from(mcp_tool: McpTool) -> Self {
        Tool::new(
            mcp_tool.name,
            mcp_tool.description,
            Arc::new(object(
                mcp_tool.input_schema.unwrap_or(EMPTY_INPUT_SCHEMA.clone()),
            )),
        )
    }
}
