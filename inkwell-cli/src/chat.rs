use colored::Colorize;
use std::io::{self, BufRead, Write};
use crate::project::LocalProject;
use crate::sdd;

pub async fn run() {
    println!("{}", "Inkwell Chat — Tapez vos messages, /inkwell.* pour les commandes, 'quit' pour quitter".bold().cyan());
    println!("{}", "Contexte: #codebase, #file:path — Commandes: /inkwell.specify, /inkwell.plan, ...".dimmed());
    println!();

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        print!("{} ", "inkwell>".green().bold());
        let _ = stdout.flush();

        let mut input = String::new();
        if stdin.lock().read_line(&mut input).is_err() { break; }
        let input = input.trim().to_string();

        if input.is_empty() { continue; }
        if input == "quit" || input == "exit" { println!("Au revoir!"); break; }

        // Slash commands
        if input.starts_with("/inkwell.") {
            handle_command(&input).await;
            continue;
        }

        // Built-in commands
        match input.as_str() {
            "status" => { crate::commands::status(); continue; }
            "validate" => { sdd::validate(); continue; }
            "list" => { crate::commands::list(); continue; }
            "help" => { crate::commands::help(); continue; }
            _ => {}
        }

        // Regular chat with LLM
        chat_with_llm(&input).await;
    }
}

async fn handle_command(input: &str) {
    let parts: Vec<&str> = input.splitn(2, ' ').collect();
    let cmd = parts[0];
    let args = parts.get(1).map(|s| s.to_string());

    match cmd {
        "/inkwell.specify" => sdd::specify(args.as_deref().unwrap_or("")).await,
        "/inkwell.plan" => sdd::plan(args).await,
        "/inkwell.tasks" => sdd::tasks().await,
        "/inkwell.clarify" => sdd::clarify(args).await,
        "/inkwell.constitution" => sdd::constitution(args).await,
        "/inkwell.implement" => sdd::implement().await,
        "/inkwell.checklist" => sdd::checklist().await,
        "/inkwell.analyze" => sdd::analyze().await,
        "/inkwell.validate" => sdd::validate(),
        _ => println!("{} Commande inconnue: {}", "✗".red(), cmd),
    }
}

async fn chat_with_llm(message: &str) {
    let project = LocalProject::load_current();
    let settings = crate::project::LocalSettings::load();
    let model = if settings.selected_model.is_empty() { "gpt-4o-mini" } else { &settings.selected_model };

    // Build context
    let mut context = String::new();
    if let Some(ref p) = project {
        if !p.constitution().is_empty() { context.push_str(&format!("## Constitution du projet\n{}\n\n", p.constitution())); }
        if !p.specification().is_empty() { context.push_str(&format!("## Specification\n{}\n\n", p.specification())); }
    }

    // Detect intent
    let is_question = message.ends_with('?') || message.to_lowercase().starts_with("comment")
        || message.to_lowercase().starts_with("pourquoi") || message.to_lowercase().starts_with("how")
        || message.to_lowercase().starts_with("what");

    let system = if is_question {
        "You are a helpful assistant. Answer clearly. Use the project context provided. Write in the user's language."
    } else {
        "You are a developer. Help with the request using the project context. Write in the user's language."
    };

    let user_prompt = if context.is_empty() { message.to_string() } else { format!("{}\n\n{}", context, message) };

    println!("{}", "...".dimmed());

    let key = if model.starts_with("claude") {
        if settings.api_key_anthropic.is_empty() { std::env::var("ANTHROPIC_API_KEY").unwrap_or_default() } else { settings.api_key_anthropic.clone() }
    } else if model.starts_with("gemini") {
        if settings.api_key_google.is_empty() { std::env::var("GOOGLE_API_KEY").unwrap_or_default() } else { settings.api_key_google.clone() }
    } else {
        if settings.api_key_openai.is_empty() { std::env::var("OPENAI_API_KEY").unwrap_or_default() } else { settings.api_key_openai.clone() }
    };

    if key.is_empty() {
        println!("{} Configurez une cle API: inkwell config set openai-key sk-...", "✗".red());
        return;
    }

    let client = reqwest::Client::builder().timeout(std::time::Duration::from_secs(60)).build().unwrap_or_default();
    let is_anthropic = model.starts_with("claude");
    let (url, body) = if is_anthropic {
        ("https://api.anthropic.com/v1/messages".to_string(),
         serde_json::json!({"model": model, "max_tokens": 2048, "system": system, "messages": [{"role": "user", "content": user_prompt}]}))
    } else if model.starts_with("gemini") {
        ("https://generativelanguage.googleapis.com/v1beta/openai/chat/completions".to_string(),
         serde_json::json!({"model": model, "messages": [{"role": "system", "content": system}, {"role": "user", "content": user_prompt}], "temperature": 0.7, "max_tokens": 2048}))
    } else {
        ("https://api.openai.com/v1/chat/completions".to_string(),
         serde_json::json!({"model": model, "messages": [{"role": "system", "content": system}, {"role": "user", "content": user_prompt}], "temperature": 0.7, "max_tokens": 2048}))
    };

    let mut req = client.post(&url).json(&body);
    if is_anthropic {
        req = req.header("x-api-key", &key).header("anthropic-version", "2023-06-01");
    } else if model.starts_with("gemini") {
        req = req.header("x-goog-api-key", &key);
    } else {
        req = req.header("Authorization", format!("Bearer {}", key));
    }

    match req.send().await {
        Ok(resp) => {
            let status = resp.status();
            if !status.is_success() {
                let body_text = resp.text().await.unwrap_or_default();
                println!("{} HTTP {} — {}", "✗".red(), status, body_text);
                return;
            }
            if let Ok(data) = resp.json::<serde_json::Value>().await {
                let text = if is_anthropic {
                    data["content"][0]["text"].as_str().unwrap_or("")
                } else {
                    data["choices"][0]["message"]["content"].as_str().unwrap_or("")
                };
                println!("\n{}\n", text);
            }
        }
        Err(e) => println!("{} {}", "Erreur:".red(), e),
    }
}
