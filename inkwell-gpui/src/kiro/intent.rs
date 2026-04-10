//! Intent detection — determines if a user message is a question (info) or action (code change).
//! Inspired by Kiro's intent recognition system.

#[derive(Clone, Debug, PartialEq)]
pub enum Intent {
    Question,   // "How does this work?" → explain without code changes
    Action,     // "Create a component" → generate/modify code
    Command,    // "/speckit.specify ..." → explicit command
    Clarify,    // "What if...?" → needs clarification
}

/// Detect intent from a user message
pub fn detect(message: &str) -> Intent {
    let msg = message.trim().to_lowercase();

    // Explicit commands
    if msg.starts_with('/') || msg.starts_with('#') {
        return Intent::Command;
    }

    // Question indicators
    let question_starters = [
        "comment", "pourquoi", "quand", "ou ", "qui ", "quel", "quelle",
        "est-ce que", "est ce que", "c'est quoi", "cest quoi",
        "how", "why", "when", "where", "who", "what", "which",
        "can you explain", "peux-tu expliquer", "explique",
    ];
    let question_endings = ["?"];

    for starter in &question_starters {
        if msg.starts_with(starter) {
            return Intent::Question;
        }
    }
    for ending in &question_endings {
        if msg.ends_with(ending) {
            return Intent::Question;
        }
    }

    // Clarification indicators
    let clarify_words = ["et si", "what if", "suppose", "imaginons", "hypothese"];
    for word in &clarify_words {
        if msg.contains(word) {
            return Intent::Clarify;
        }
    }

    // Action indicators (imperative verbs)
    let action_starters = [
        "cree", "creer", "ajoute", "ajouter", "supprime", "supprimer",
        "modifie", "modifier", "genere", "generer", "implemente", "implementer",
        "corrige", "corriger", "refactore", "refactorer", "optimise", "optimiser",
        "ecris", "ecrire", "fais", "faire", "met", "mettre",
        "create", "add", "remove", "delete", "modify", "generate", "implement",
        "fix", "refactor", "optimize", "write", "make", "build", "update",
    ];
    for starter in &action_starters {
        if msg.starts_with(starter) || msg.contains(&format!(" {}", starter)) {
            return Intent::Action;
        }
    }

    // Default: treat as action (like Kiro's default behavior)
    Intent::Action
}

/// Get system prompt based on detected intent
pub fn system_prompt_for_intent(intent: &Intent) -> &'static str {
    match intent {
        Intent::Question => "You are a helpful assistant. Answer the user's question clearly and concisely. Do NOT modify any code or files — only explain.",
        Intent::Action => "You are a skilled developer. Implement what the user asks. Write clean, well-structured code. Explain your changes briefly.",
        Intent::Command => "You are executing a spec-driven development command. Follow the SpecKit workflow precisely.",
        Intent::Clarify => "The user is exploring possibilities. Ask clarifying questions to understand their intent better before taking action. List 3-5 options or considerations.",
    }
}
