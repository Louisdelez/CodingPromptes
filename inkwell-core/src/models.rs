/// Canonical list of LLM models supported across Inkwell (GPUI, CLI, MCP).
/// Single source of truth for model validation.

pub const SUPPORTED_MODELS: &[&str] = &[
    // OpenAI
    "gpt-4o",
    "gpt-4o-mini",
    "gpt-4.1",
    "gpt-4.1-mini",
    "gpt-4.1-nano",
    "o3-mini",
    // Anthropic
    "claude-opus-4-6",
    "claude-sonnet-4-6",
    "claude-haiku-4-5",
    // Google
    "gemini-2.5-pro",
    "gemini-2.5-flash",
    // Ollama (local) — common names, extend as needed
    "llama3.2",
    "llama3.1",
    "qwen2.5",
    "mistral",
];

pub fn is_supported(model: &str) -> bool {
    let m = model.to_lowercase();
    SUPPORTED_MODELS.iter().any(|s| *s == m)
        // Accept any Ollama-style model name as a fallback (contains : or starts with "ollama/")
        || m.starts_with("ollama/")
        || m.contains(':')
}
