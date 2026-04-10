//! Credit/cost tracking per prompt.
//! Tracks tokens used and estimated costs.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PromptCost {
    pub model: String,
    pub tokens_in: u64,
    pub tokens_out: u64,
    pub cost: f64,
    pub timestamp: i64,
}

#[derive(Clone, Debug, Default)]
pub struct CreditTracker {
    pub total_tokens_in: u64,
    pub total_tokens_out: u64,
    pub total_cost: f64,
    pub prompts_count: u32,
    pub history: Vec<PromptCost>,
}

impl CreditTracker {
    pub fn new() -> Self { Self::default() }

    /// Record a prompt's cost
    pub fn record(&mut self, model: &str, tokens_in: u64, tokens_out: u64) {
        let cost = estimate_cost(model, tokens_in, tokens_out);
        self.total_tokens_in += tokens_in;
        self.total_tokens_out += tokens_out;
        self.total_cost += cost;
        self.prompts_count += 1;
        self.history.push(PromptCost {
            model: model.to_string(), tokens_in, tokens_out, cost,
            timestamp: chrono::Utc::now().timestamp_millis(),
        });
        // Keep last 100 entries
        if self.history.len() > 100 { self.history.remove(0); }
    }

    /// Estimate cost from a response text (rough approximation)
    pub fn record_from_text(&mut self, model: &str, prompt_len: usize, response_len: usize) {
        let tokens_in = (prompt_len as f64 / 4.0).ceil() as u64;
        let tokens_out = (response_len as f64 / 4.0).ceil() as u64;
        self.record(model, tokens_in, tokens_out);
    }
}

/// Estimate cost per token based on model
fn estimate_cost(model: &str, tokens_in: u64, tokens_out: u64) -> f64 {
    let (price_in, price_out) = match model {
        m if m.contains("gpt-4o-mini") => (0.00000015, 0.0000006),
        m if m.contains("gpt-4o") => (0.0000025, 0.00001),
        m if m.contains("gpt-4.1") => (0.000002, 0.000008),
        m if m.contains("claude-sonnet") => (0.000003, 0.000015),
        m if m.contains("claude-opus") => (0.000015, 0.000075),
        m if m.contains("claude-haiku") => (0.00000025, 0.00000125),
        m if m.contains("gemini") => (0.0000001, 0.0000004),
        _ => (0.000001, 0.000004), // Default estimate
    };
    tokens_in as f64 * price_in + tokens_out as f64 * price_out
}
