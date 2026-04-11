use colored::Colorize;
use crate::project::{LocalProject, LocalSettings};

pub fn init(name: Option<String>, here: bool) {
    let project_name = name.unwrap_or_else(|| {
        if here { std::env::current_dir().unwrap_or_else(|e| { eprintln!("{} Impossible de lire le repertoire courant: {}", "✗".red(), e); std::path::PathBuf::from(".") }).file_name().unwrap_or_else(|| { eprintln!("{} Impossible de lire le nom du repertoire", "✗".red()); std::ffi::OsStr::new("projet") }).to_string_lossy().to_string() }
        else { "nouveau-projet".to_string() }
    });
    if project_name.contains('/') || project_name.contains('\\') || project_name.contains("..") {
        println!("{} Nom de projet invalide (caracteres interdits: / \\ ..)", "✗".red());
        return;
    }
    let project = LocalProject::new(&project_name);
    if !here {
        if let Err(e) = std::fs::create_dir_all(&project_name) {
            println!("{} Erreur creation repertoire: {}", "✗".red(), e);
        }
        if let Err(e) = project.save_to_inkwell(std::path::Path::new(&project_name)) {
            println!("{} Erreur sauvegarde: {}", "✗".red(), e);
        }
        println!("{} Projet {} cree", "✓".green(), project_name.cyan());
        println!("  cd {}", project_name);
    } else {
        if let Err(e) = project.save_to_inkwell(std::path::Path::new(".")) {
            println!("{} Erreur sauvegarde: {}", "✗".red(), e);
        }
        println!("{} Projet initialise dans le repertoire courant", "✓".green());
    }
    if let Err(e) = project.save() {
        println!("{} Erreur sauvegarde: {}", "✗".red(), e);
    }
    println!("\n{}", "Commandes:".bold());
    println!("  inkwell constitution  — Principes du projet");
    println!("  inkwell specify       — Specification");
    println!("  inkwell plan          — Plan technique");
    println!("  inkwell tasks         — Taches");
    println!("  inkwell implement     — Autopilot");
}

pub fn list() {
    let projects = LocalProject::load_all();
    println!("{}", "Projets Inkwell:".bold());
    if projects.is_empty() {
        println!("  {}", "Aucun projet. 'inkwell init' pour commencer.".dimmed());
        return;
    }
    let current_id = std::fs::read_to_string(LocalProject::data_dir().join("current-project-id.txt")).unwrap_or_default();
    for p in &projects {
        let phases: Vec<(&str, bool)> = vec![
            ("C", !p.constitution().is_empty()), ("S", !p.specification().is_empty()),
            ("P", !p.plan().is_empty()), ("T", !p.tasks().is_empty()), ("I", !p.implementation().is_empty()),
        ];
        let active = if current_id.trim() == p.id { "→ ".cyan().to_string() } else { "  ".into() };
        let progress: String = phases.iter().map(|(l, done)| if *done { l.green().to_string() } else { l.dimmed().to_string() }).collect::<Vec<_>>().join("");
        println!("{}{} [{}]", active, p.name.bold(), progress);
    }
}

pub fn status() {
    if let Some(p) = LocalProject::load_current() {
        println!("{} {}", "Projet:".bold(), p.name.cyan());
        let phases = [("Constitution", p.constitution()), ("Specification", p.specification()), ("Plan", p.plan()), ("Tasks", p.tasks()), ("Implementation", p.implementation())];
        let done = phases.iter().filter(|(_, c)| !c.is_empty()).count();
        println!("{} {}/5", "Progression:".bold(), done);
        for (name, content) in &phases {
            let s = if content.is_empty() { "vide".dimmed().to_string() } else { format!("{} car.", content.len()).green().to_string() };
            println!("  {} — {}", name, s);
        }
    } else { println!("{}", "Aucun projet actif. 'inkwell init'.".yellow()); }
}

pub fn config(action: Option<String>, key: Option<String>, value: Option<String>) {
    let mut settings = LocalSettings::load();
    match action.as_deref() {
        Some("set") => {
            let k = key.unwrap_or_default();
            let v = value.unwrap_or_default();
            match k.as_str() {
                "model" => { settings.selected_model = v.clone(); println!("{} Modele: {}", "✓".green(), v.cyan()); }
                "openai-key" => { settings.api_key_openai = v; println!("{} Cle OpenAI configuree", "✓".green()); }
                "anthropic-key" => { settings.api_key_anthropic = v; println!("{} Cle Anthropic configuree", "✓".green()); }
                "google-key" => { settings.api_key_google = v; println!("{} Cle Google configuree", "✓".green()); }
                _ => { println!("{} Cle inconnue: {}. Options: model, openai-key, anthropic-key, google-key", "✗".red(), k); return; }
            }
            if let Err(e) = settings.save() {
                println!("{} Erreur sauvegarde: {}", "✗".red(), e);
            }
        }
        _ => {
            println!("{}", "Configuration Inkwell:".bold());
            println!("  {} {}", "Modele:".dimmed(), if settings.selected_model.is_empty() { "gpt-4o-mini (defaut)".dimmed().to_string() } else { settings.selected_model.cyan().to_string() });
            println!("  {} {}", "OpenAI:".dimmed(), if settings.api_key_openai.is_empty() { "non configure".red().to_string() } else { "configure".green().to_string() });
            println!("  {} {}", "Anthropic:".dimmed(), if settings.api_key_anthropic.is_empty() { "non configure".red().to_string() } else { "configure".green().to_string() });
            println!("  {} {}", "Google:".dimmed(), if settings.api_key_google.is_empty() { "non configure".red().to_string() } else { "configure".green().to_string() });
            println!("\n  {} inkwell config set model claude-sonnet-4.6", "Exemple:".dimmed());
        }
    }
}

pub fn mcp_install() {
    let mcp_binary = std::env::current_exe().ok()
        .and_then(|p| p.parent().map(|d| d.join("inkwell-mcp").to_string_lossy().to_string()))
        .unwrap_or_else(|| "inkwell-mcp".into());

    // Write to ~/.claude.json
    let claude_config = dirs::home_dir().unwrap_or_default().join(".claude.json");
    let mut config: serde_json::Value = std::fs::read_to_string(&claude_config).ok()
        .and_then(|j| serde_json::from_str(&j).ok())
        .unwrap_or(serde_json::json!({}));

    config["mcpServers"]["inkwell"] = serde_json::json!({
        "command": mcp_binary,
        "args": []
    });

    if let Ok(json) = serde_json::to_string_pretty(&config) {
        let _ = std::fs::write(&claude_config, json);
        println!("{} MCP server Inkwell configure dans {}", "✓".green(), claude_config.display());
        println!("  Redemarrez Claude Code pour activer.");
    } else {
        println!("{} Erreur lors de la configuration", "✗".red());
    }
}

pub fn help() {
    println!("{}", "Inkwell CLI — Spec-Driven Development".bold().cyan());
    println!();
    println!("{}", "SDD:".bold());
    println!("  {} {}", "constitution".cyan(), " Principes du projet");
    println!("  {} {}", "specify".cyan(), "      Specification");
    println!("  {} {}", "plan".cyan(), "         Plan technique");
    println!("  {} {}", "tasks".cyan(), "        Taches");
    println!("  {} {}", "implement".cyan(), "    Autopilot");
    println!("  {} {}", "checklist".cyan(), "    Checklist qualite");
    println!("  {} {}", "analyze".cyan(), "      Auditer le plan");
    println!("  {} {}", "clarify".cyan(), "      Clarifier");
    println!("  {} {}", "validate".cyan(), "     Valider (offline)");
    println!();
    println!("{}", "GESTION:".bold());
    println!("  {} {}", "init".cyan(), "         Initialiser un projet");
    println!("  {} {}", "list".cyan(), "         Lister les projets");
    println!("  {} {}", "status".cyan(), "       Statut du projet");
    println!("  {} {}", "config".cyan(), "       Configuration (cles API, modele)");
    println!("  {} {}", "mcp-install".cyan(), "  Configurer le MCP pour Claude Code");
    println!("  {} {}", "chat".cyan(), "         Chat interactif");
    println!("  {} {}", "completions".cyan(), "  Completions shell (bash, zsh, fish)");
}
