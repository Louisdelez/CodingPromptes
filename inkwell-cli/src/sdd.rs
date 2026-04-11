use colored::Colorize;
use crate::project::{LocalProject, LocalSettings};
use inkwell_core::types::BlockType;

async fn llm_call(system: &str, user: &str) -> String {
    let settings = LocalSettings::load();
    let model = if settings.selected_model.is_empty() { "gpt-4o-mini".to_string() } else { settings.selected_model.clone() };

    // Determine provider and API key
    let (url, key, is_anthropic): (String, String, bool) = if model.starts_with("claude") {
        let key = if settings.api_key_anthropic.is_empty() { std::env::var("ANTHROPIC_API_KEY").unwrap_or_default() } else { settings.api_key_anthropic.clone() };
        ("https://api.anthropic.com/v1/messages".into(), key, true)
    } else if model.starts_with("gemini") {
        let key = if settings.api_key_google.is_empty() { std::env::var("GOOGLE_API_KEY").unwrap_or_default() } else { settings.api_key_google.clone() };
        ("https://generativelanguage.googleapis.com/v1beta/openai/chat/completions".into(), key, false)
    } else {
        let key = if settings.api_key_openai.is_empty() { std::env::var("OPENAI_API_KEY").unwrap_or_default() } else { settings.api_key_openai.clone() };
        ("https://api.openai.com/v1/chat/completions".into(), key, false)
    };

    if key.is_empty() && !model.contains("ollama") {
        eprintln!("{} Cle API non configuree pour le modele '{}'.", "✗".red(), model);
        eprintln!("  Configurez avec: {} ou {}", "inkwell config set openai-key sk-...".cyan(), "export OPENAI_API_KEY=sk-...".cyan());
        // Try Ollama local fallback
        eprintln!("{} Tentative Ollama local...", "⏳".yellow());
        return llm_call_local(system, user, &model).await;
    }

    let client = reqwest::Client::builder().timeout(std::time::Duration::from_secs(60)).build().unwrap_or_default();
    let body = if is_anthropic {
        serde_json::json!({"model": model, "max_tokens": 4096, "system": system, "messages": [{"role": "user", "content": user}]})
    } else {
        serde_json::json!({"model": model, "messages": [{"role": "system", "content": system}, {"role": "user", "content": user}], "temperature": 0.3, "max_tokens": 4096})
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
                eprintln!("{} HTTP {} — {}", "✗".red(), status, body_text);
                return String::new();
            }
            if let Ok(data) = resp.json::<serde_json::Value>().await {
                if is_anthropic {
                    data["content"][0]["text"].as_str().unwrap_or("").to_string()
                } else {
                    data["choices"][0]["message"]["content"].as_str().unwrap_or("").to_string()
                }
            } else { "Erreur: reponse invalide".into() }
        }
        Err(e) => format!("Erreur: {}", e),
    }
}

async fn llm_call_local(system: &str, user: &str, _model: &str) -> String {
    let client = reqwest::Client::builder().timeout(std::time::Duration::from_secs(60)).build().unwrap_or_default();
    let body = serde_json::json!({"model": "llama3", "messages": [{"role": "system", "content": system}, {"role": "user", "content": user}], "temperature": 0.3, "max_tokens": 4096, "stream": false});
    match client.post("http://localhost:11434/v1/chat/completions").json(&body).send().await {
        Ok(resp) => {
            if let Ok(data) = resp.json::<serde_json::Value>().await {
                data["choices"][0]["message"]["content"].as_str().unwrap_or("Ollama non disponible").to_string()
            } else { "Erreur Ollama".into() }
        }
        Err(_) => "Ollama non disponible. Installez-le ou configurez une cle API.".into(),
    }
}

pub async fn constitution(description: Option<String>) {
    let mut project = LocalProject::load_current().unwrap_or_else(|| LocalProject::new("projet"));
    println!("{} Generation de la constitution...", "⏳".yellow());
    let desc = description.unwrap_or_else(|| project.name.clone());
    let result = llm_call(
        "You are a software architect. Create a project constitution with core principles, constraints, and governance. Use numbered principles (I, II, III). Write in the user's language.",
        &format!("Create a constitution for project: {}", desc),
    ).await;
    if result.is_empty() {
        println!("{} Erreur: contenu vide, sauvegarde ignoree.", "✗".red());
        return;
    }
    project.set_phase(BlockType::SddConstitution, result.clone());
    if let Err(e) = project.save() {
        println!("{} Erreur sauvegarde: {}", "✗".red(), e);
    }
    println!("{} Constitution generee ({} car.)", "✓".green(), result.len());
    println!("\n{}", result.dimmed());
}

pub async fn specify(description: &str) {
    let mut project = LocalProject::load_current().unwrap_or_else(|| LocalProject::new("projet"));
    println!("{} Generation de la specification...", "⏳".yellow());
    let mut ctx = String::new();
    if !project.constitution().is_empty() { ctx.push_str(&format!("## Constitution\n{}\n\n", project.constitution())); }
    let result = llm_call(
        "You are a product manager. Create a feature specification with prioritized user stories (P1/P2/P3), acceptance scenarios (Given/When/Then), functional requirements (FR-001...), and success criteria. No technical details. Write in the user's language.",
        &format!("{}Feature: {}", ctx, description),
    ).await;
    if result.is_empty() {
        println!("{} Erreur: contenu vide, sauvegarde ignoree.", "✗".red());
        return;
    }
    project.set_phase(BlockType::SddSpecification, result.clone());
    if let Err(e) = project.save() {
        println!("{} Erreur sauvegarde: {}", "✗".red(), e);
    }
    println!("{} Specification generee ({} car.)", "✓".green(), result.len());
}

pub async fn plan(tech: Option<String>) {
    let mut project = LocalProject::load_current().unwrap_or_else(|| LocalProject::new("projet"));
    println!("{} Generation du plan...", "⏳".yellow());
    let mut ctx = String::new();
    if !project.constitution().is_empty() { ctx.push_str(&format!("## Constitution\n{}\n\n", project.constitution())); }
    if !project.specification().is_empty() { ctx.push_str(&format!("## Specification\n{}\n\n", project.specification())); }
    if let Some(ref t) = tech { ctx.push_str(&format!("## Tech Stack\n{}\n\n", t)); }
    let result = llm_call(
        "You are a technical architect. Create an implementation plan with: Technical Context, Constitution Check, Project Structure, Design Decisions. Write in the user's language.",
        &format!("{}Create implementation plan.", ctx),
    ).await;
    if result.is_empty() {
        println!("{} Erreur: contenu vide, sauvegarde ignoree.", "✗".red());
        return;
    }
    project.set_phase(BlockType::SddPlan, result.clone());
    if let Err(e) = project.save() {
        println!("{} Erreur sauvegarde: {}", "✗".red(), e);
    }
    println!("{} Plan genere ({} car.)", "✓".green(), result.len());
}

pub async fn tasks() {
    let mut project = LocalProject::load_current().unwrap_or_else(|| LocalProject::new("projet"));
    println!("{} Generation des taches...", "⏳".yellow());
    let mut ctx = String::new();
    if !project.specification().is_empty() { ctx.push_str(&format!("## Specification\n{}\n\n", project.specification())); }
    if !project.plan().is_empty() { ctx.push_str(&format!("## Plan\n{}\n\n", project.plan())); }
    let result = llm_call(
        "You are a project manager. Break down into tasks. Format: - [ ] T001 [P?] [US#] Description. Group by phases. Write in the user's language.",
        &format!("{}Generate task list.", ctx),
    ).await;
    if result.is_empty() {
        println!("{} Erreur: contenu vide, sauvegarde ignoree.", "✗".red());
        return;
    }
    project.set_phase(BlockType::SddTasks, result.clone());
    if let Err(e) = project.save() {
        println!("{} Erreur sauvegarde: {}", "✗".red(), e);
    }
    println!("{} Taches generees ({} car.)", "✓".green(), result.len());
}

pub async fn clarify(details: Option<String>) {
    let project = LocalProject::load_current().unwrap_or_else(|| LocalProject::new("projet"));
    println!("{} Generation des questions...", "⏳".yellow());
    let mut ctx = String::new();
    if !project.specification().is_empty() { ctx.push_str(&format!("## Specification\n{}\n\n", project.specification())); }
    if let Some(d) = details { ctx.push_str(&format!("Additional: {}\n\n", d)); }
    let result = llm_call("You are a business analyst. Generate 3-5 clarifying questions. Write in the user's language.", &format!("{}Generate clarifying questions.", ctx)).await;
    println!("{} Questions:", "✓".green());
    println!("\n{}", result);
}

pub async fn implement() {
    let mut project = LocalProject::load_current().unwrap_or_else(|| LocalProject::new("projet"));
    if project.tasks().is_empty() { println!("{} Aucune tache. Lancez 'inkwell tasks' d'abord.", "✗".red()); return; }
    let lines: Vec<&str> = project.tasks().lines().filter(|l| l.trim().starts_with("- [ ]")).collect();
    if lines.is_empty() { println!("{} Toutes les taches completees!", "✓".green()); return; }
    let task_line = lines[0].trim().to_string();
    let next = task_line.strip_prefix("- [ ] ").unwrap_or(&task_line);
    println!("{} Execution: {}", "⏳".yellow(), next.cyan());
    let result = llm_call("You are a developer. Implement this task. Write clean code.",
        &format!("Constitution:\n{}\n\nPlan:\n{}\n\nTask: {}", project.constitution(), project.plan(), next)).await;
    if !result.is_empty() {
        // Mark the task as done
        let updated_tasks = project.tasks().replacen(&task_line, &task_line.replacen("- [ ]", "- [x]", 1), 1);
        project.set_phase(BlockType::SddTasks, updated_tasks);
        if let Err(e) = project.save() {
            println!("{} Erreur sauvegarde: {}", "✗".red(), e);
        }
    }
    println!("{} Resultat:", "✓".green());
    println!("\n{}", result);
}

pub async fn checklist() {
    let project = LocalProject::load_current().unwrap_or_else(|| LocalProject::new("projet"));
    println!("{} Generation du checklist...", "⏳".yellow());
    let mut ctx = String::new();
    if !project.specification().is_empty() { ctx.push_str(&format!("## Specification\n{}\n\n", project.specification())); }
    if !project.plan().is_empty() { ctx.push_str(&format!("## Plan\n{}\n\n", project.plan())); }
    let result = llm_call("You are a quality auditor. Generate a quality checklist. Format: - [ ] CHK001 Description. Write in the user's language.", &format!("{}Generate checklist.", ctx)).await;
    println!("{} Checklist:", "✓".green());
    println!("\n{}", result);
}

pub async fn analyze() {
    let project = LocalProject::load_current().unwrap_or_else(|| LocalProject::new("projet"));
    println!("{} Analyse du plan...", "⏳".yellow());
    let mut ctx = String::new();
    if !project.constitution().is_empty() { ctx.push_str(&format!("## Constitution\n{}\n\n", project.constitution())); }
    if !project.specification().is_empty() { ctx.push_str(&format!("## Specification\n{}\n\n", project.specification())); }
    if !project.plan().is_empty() { ctx.push_str(&format!("## Plan\n{}\n\n", project.plan())); }
    let result = llm_call("You are an architecture reviewer. Analyze for completeness, consistency, constitutional compliance. List issues with severity.", &format!("{}Analyze.", ctx)).await;
    println!("{} Analyse:", "✓".green());
    println!("\n{}", result);
}

pub fn validate() {
    let project = LocalProject::load_current().unwrap_or_else(|| LocalProject::new("projet"));
    println!("{}", "Validation SDD:".bold());
    let checks: Vec<(&str, &str, Vec<&str>)> = vec![
        ("Constitution", project.constitution(), vec!["###", "Governance"]),
        ("Specification", project.specification(), vec!["User Story", "Given", "FR-"]),
        ("Plan", project.plan(), vec!["Technical Context", "Project Structure"]),
        ("Tasks", project.tasks(), vec!["- [ ] T", "Phase"]),
    ];
    let mut total = 0;
    for (name, content, markers) in &checks {
        if content.is_empty() { println!("  {} {} — {}", "✗".red(), name, "vide".red()); total += 1; }
        else {
            let missing: Vec<&&str> = markers.iter().filter(|m| !content.contains(**m)).collect();
            if missing.is_empty() { println!("  {} {} — {} ({} car.)", "✓".green(), name, "OK".green(), content.len()); }
            else { println!("  {} {} — manque: {}", "⚠".yellow(), name, missing.iter().map(|m| m.to_string()).collect::<Vec<_>>().join(", ").yellow()); total += missing.len(); }
        }
    }
    if total == 0 { println!("\n{}", "Valide!".green().bold()); }
    else { println!("\n{} probleme(s)", total.to_string().yellow()); }
}
