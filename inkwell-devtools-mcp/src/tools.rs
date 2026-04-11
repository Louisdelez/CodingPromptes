use serde_json::json;

pub fn list_tools() -> Vec<serde_json::Value> {
    vec![
        // Read tools
        tool("devtools_health_check", "Check if the Inkwell GPUI app is running and responsive", json!({"type":"object","properties":{}})),
        tool("devtools_app_state", "Get complete snapshot of the running app state (project, blocks, UI, metrics)", json!({"type":"object","properties":{}})),
        tool("devtools_get_project", "Get current project with all blocks", json!({"type":"object","properties":{}})),
        tool("devtools_get_block", "Get a specific block by index", json!({
            "type":"object",
            "properties":{"index":{"type":"integer","description":"Block index (0-based)"}},
            "required":["index"]
        })),
        tool("devtools_get_metrics", "Get prompt metrics (tokens, chars, words, lines, blocks)", json!({"type":"object","properties":{}})),
        tool("devtools_list_tabs", "List active tabs and panel visibility", json!({"type":"object","properties":{}})),
        tool("devtools_get_logs", "Get recent app logs", json!({
            "type":"object",
            "properties":{"lines":{"type":"integer","description":"Number of log lines (default 50)"}},
        })),
        tool("devtools_validate_state", "Check app state for inconsistencies", json!({"type":"object","properties":{}})),

        // Screenshot
        tool("devtools_screenshot", "Capture a screenshot of the Inkwell window", json!({"type":"object","properties":{}})),

        // Write tools
        tool("devtools_set_block", "Set the content of a block", json!({
            "type":"object",
            "properties":{
                "index":{"type":"integer","description":"Block index"},
                "content":{"type":"string","description":"New content"}
            },
            "required":["index","content"]
        })),
        tool("devtools_add_block", "Add a new block to the project", json!({
            "type":"object",
            "properties":{
                "block_type":{"type":"string","description":"Block type: role, context, task, examples, constraints, format"},
                "content":{"type":"string","description":"Initial content"}
            },
        })),
        tool("devtools_delete_block", "Delete a block by index", json!({
            "type":"object",
            "properties":{"index":{"type":"integer","description":"Block index"}},
            "required":["index"]
        })),
        tool("devtools_toggle_block", "Enable or disable a block", json!({
            "type":"object",
            "properties":{"index":{"type":"integer","description":"Block index"}},
            "required":["index"]
        })),
        tool("devtools_reorder_blocks", "Move a block from one position to another", json!({
            "type":"object",
            "properties":{
                "from":{"type":"integer","description":"Source index"},
                "to":{"type":"integer","description":"Target index"}
            },
            "required":["from","to"]
        })),
        tool("devtools_select_tab", "Switch the active right panel tab", json!({
            "type":"object",
            "properties":{"tab":{"type":"string","description":"Tab name: Preview, Playground, Chat, Sdd, History, Export, Terminal, Optimize, Lint, Analytics, Chain, Stt"}},
            "required":["tab"]
        })),
        tool("devtools_toggle_panel", "Show or hide a panel", json!({
            "type":"object",
            "properties":{"panel":{"type":"string","description":"Panel: left or right"}},
            "required":["panel"]
        })),
        tool("devtools_set_model", "Change the selected LLM model", json!({
            "type":"object",
            "properties":{"model":{"type":"string","description":"Model name (e.g. gpt-4o, claude-sonnet-4.6)"}},
            "required":["model"]
        })),
        tool("devtools_open_project", "Switch to a different project by ID", json!({
            "type":"object",
            "properties":{"project_id":{"type":"string","description":"Project UUID"}},
            "required":["project_id"]
        })),

        // Action tools
        tool("devtools_run_prompt", "Execute the current prompt against the selected LLM model", json!({"type":"object","properties":{}})),
        tool("devtools_run_sdd", "Start the SDD pipeline (constitution → specification → plan → tasks)", json!({"type":"object","properties":{}})),
        tool("devtools_send_chat", "Send a chat message in the app's chat panel", json!({
            "type":"object",
            "properties":{"message":{"type":"string","description":"Chat message to send"}},
            "required":["message"]
        })),
        tool("devtools_save_project", "Force save the current project to disk", json!({"type":"object","properties":{}})),

        // New project/variable/tab tools
        tool("devtools_new_project", "Create a brand new project with default blocks (Role/Context/Task)", json!({
            "type":"object",
            "properties":{"name":{"type":"string","description":"Project name (default: 'Nouveau prompt')"}},
        })),
        tool("devtools_rename_project", "Rename the currently open project", json!({
            "type":"object",
            "properties":{"name":{"type":"string","description":"New project name"}},
            "required":["name"]
        })),
        tool("devtools_set_variable", "Set a template variable value (substitutes {{key}} in the prompt)", json!({
            "type":"object",
            "properties":{
                "key":{"type":"string","description":"Variable name (without braces)"},
                "value":{"type":"string","description":"Value to substitute"}
            },
            "required":["key","value"]
        })),
        tool("devtools_delete_variable", "Remove a template variable", json!({
            "type":"object",
            "properties":{"key":{"type":"string","description":"Variable name"}},
            "required":["key"]
        })),
        tool("devtools_select_left_tab", "Switch the active left panel tab", json!({
            "type":"object",
            "properties":{"tab":{"type":"string","description":"Tab name: Library, Frameworks, Versions"}},
            "required":["tab"]
        })),

        // New read tools
        tool("devtools_get_variables", "Get all template variables defined on the current project", json!({"type":"object","properties":{}})),
        tool("devtools_get_chat_messages", "Get the chat message history (role, content)", json!({
            "type":"object",
            "properties":{"limit":{"type":"integer","description":"Max messages (default: all)"}},
        })),
        tool("devtools_get_executions", "Get recent prompt executions with metrics and previews", json!({
            "type":"object",
            "properties":{"limit":{"type":"integer","description":"Max executions to return (default 20)"}},
        })),
        tool("devtools_get_playground_response", "Get the last run_prompt response and loading state", json!({"type":"object","properties":{}})),
    ]
}

fn tool(name: &str, description: &str, input_schema: serde_json::Value) -> serde_json::Value {
    json!({
        "name": name,
        "description": description,
        "inputSchema": input_schema,
    })
}

pub async fn call_tool(name: &str, args: &serde_json::Value) -> String {
    // Map MCP tool names to socket methods
    let method = match name {
        "devtools_health_check" => "devtools/health_check",
        "devtools_app_state" => "devtools/app_state",
        "devtools_get_project" => "devtools/get_project",
        "devtools_get_block" => "devtools/get_block",
        "devtools_get_metrics" => "devtools/get_metrics",
        "devtools_list_tabs" => "devtools/list_tabs",
        "devtools_get_logs" => "devtools/get_logs",
        "devtools_validate_state" => "devtools/validate_state",
        "devtools_screenshot" => "devtools/screenshot",
        "devtools_set_block" => "devtools/set_block",
        "devtools_add_block" => "devtools/add_block",
        "devtools_delete_block" => "devtools/delete_block",
        "devtools_toggle_block" => "devtools/toggle_block",
        "devtools_reorder_blocks" => "devtools/reorder_blocks",
        "devtools_select_tab" => "devtools/select_tab",
        "devtools_toggle_panel" => "devtools/toggle_panel",
        "devtools_set_model" => "devtools/set_model",
        "devtools_open_project" => "devtools/open_project",
        "devtools_run_prompt" => "devtools/run_prompt",
        "devtools_run_sdd" => "devtools/run_sdd",
        "devtools_send_chat" => "devtools/send_chat",
        "devtools_save_project" => "devtools/save_project",
        "devtools_new_project" => "devtools/new_project",
        "devtools_rename_project" => "devtools/rename_project",
        "devtools_set_variable" => "devtools/set_variable",
        "devtools_delete_variable" => "devtools/delete_variable",
        "devtools_select_left_tab" => "devtools/select_left_tab",
        "devtools_get_variables" => "devtools/get_variables",
        "devtools_get_chat_messages" => "devtools/get_chat_messages",
        "devtools_get_executions" => "devtools/get_executions",
        "devtools_get_playground_response" => "devtools/get_playground_response",
        _ => return format!("Unknown tool: {}", name),
    };

    let params = if args.is_null() || args.is_object() && args.as_object().map_or(true, |m| m.is_empty()) {
        json!({})
    } else {
        args.clone()
    };

    match crate::socket::call(method, params).await {
        Ok(result) => serde_json::to_string_pretty(&result).unwrap_or_else(|_| result.to_string()),
        Err(e) => format!("Error: {}", e),
    }
}
