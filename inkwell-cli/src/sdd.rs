use colored::Colorize;
use crate::project::InkwellProject;

async fn llm_call(system: &str, user: &str) -> String {
    let client = reqwest::Client::new();
    let body = serde_json::json!({
        "model": "gpt-4o-mini",
        "messages": [
            {"role": "system", "content": system},
            {"role": "user", "content": user}
        ],
        "temperature": 0.3,
        "max_tokens": 4096
    });
    match client.post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", std::env::var("OPENAI_API_KEY").unwrap_or_default()))
        .json(&body).send().await
    {
        Ok(resp) => {
            if let Ok(data) = resp.json::<serde_json::Value>().await {
                data["choices"][0]["message"]["content"].as_str().unwrap_or("").to_string()
            } else { "Erreur: reponse invalide".into() }
        }
        Err(e) => format!("Erreur: {}", e),
    }
}

pub async fn constitution(description: Option<String>) {
    let mut project = InkwellProject::load().unwrap_or_else(|| InkwellProject::new("projet"));
    println!("{} Generation de la constitution...", "⏳".yellow());
    let desc = description.unwrap_or_else(|| project.name.clone());
    let result = llm_call(
        "You are a software architect. Create a project constitution with core principles, constraints, and governance. Use numbered principles (I, II, III). Write in the user's language.",
        &format!("Create a constitution for project: {}", desc),
    ).await;
    project.constitution = result.clone();
    let _ = project.save();
    println!("{} Constitution generee ({} car.)", "✓".green(), result.len());
    println!("\n{}", result.dimmed());
}

pub async fn specify(description: &str) {
    let mut project = InkwellProject::load().unwrap_or_else(|| InkwellProject::new("projet"));
    println!("{} Generation de la specification...", "⏳".yellow());
    let mut ctx = String::new();
    if !project.constitution.is_empty() { ctx.push_str(&format!("## Constitution\n{}\n\n", project.constitution)); }
    let result = llm_call(
        "You are a product manager. Create a feature specification with prioritized user stories (P1/P2/P3), acceptance scenarios (Given/When/Then), functional requirements (FR-001...), and success criteria. No technical details. Write in the user's language.",
        &format!("{}Feature description: {}", ctx, description),
    ).await;
    project.specification = result.clone();
    let _ = project.save();
    println!("{} Specification generee ({} car.)", "✓".green(), result.len());
    println!("\n{}", result.dimmed());
}

pub async fn plan(tech: Option<String>) {
    let mut project = InkwellProject::load().unwrap_or_else(|| InkwellProject::new("projet"));
    println!("{} Generation du plan...", "⏳".yellow());
    let mut ctx = String::new();
    if !project.constitution.is_empty() { ctx.push_str(&format!("## Constitution\n{}\n\n", project.constitution)); }
    if !project.specification.is_empty() { ctx.push_str(&format!("## Specification\n{}\n\n", project.specification)); }
    if let Some(ref t) = tech { ctx.push_str(&format!("## Tech Stack\n{}\n\n", t)); }
    let result = llm_call(
        "You are a technical architect. Create an implementation plan with: Technical Context, Constitution Check, Project Structure, Design Decisions. Write in the user's language.",
        &format!("{}Create implementation plan.", ctx),
    ).await;
    project.plan = result.clone();
    let _ = project.save();
    println!("{} Plan genere ({} car.)", "✓".green(), result.len());
}

pub async fn tasks() {
    let mut project = InkwellProject::load().unwrap_or_else(|| InkwellProject::new("projet"));
    println!("{} Generation des taches...", "⏳".yellow());
    let mut ctx = String::new();
    if !project.specification.is_empty() { ctx.push_str(&format!("## Specification\n{}\n\n", project.specification)); }
    if !project.plan.is_empty() { ctx.push_str(&format!("## Plan\n{}\n\n", project.plan)); }
    let result = llm_call(
        "You are a project manager. Break down into tasks. Format: - [ ] T001 [P?] [US#] Description. Group by phases. Write in the user's language.",
        &format!("{}Generate task list.", ctx),
    ).await;
    project.tasks = result.clone();
    let _ = project.save();
    println!("{} Taches generees ({} car.)", "✓".green(), result.len());
}

pub async fn clarify(details: Option<String>) {
    let project = InkwellProject::load().unwrap_or_else(|| InkwellProject::new("projet"));
    println!("{} Generation des questions de clarification...", "⏳".yellow());
    let mut ctx = String::new();
    if !project.specification.is_empty() { ctx.push_str(&format!("## Specification\n{}\n\n", project.specification)); }
    if let Some(d) = details { ctx.push_str(&format!("Additional: {}\n\n", d)); }
    let result = llm_call(
        "You are a business analyst. Generate 3-5 clarifying questions. Write in the user's language.",
        &format!("{}Generate clarifying questions.", ctx),
    ).await;
    println!("{} Questions generees:", "✓".green());
    println!("\n{}", result);
}

pub async fn implement() {
    let project = InkwellProject::load().unwrap_or_else(|| InkwellProject::new("projet"));
    if project.tasks.is_empty() {
        println!("{} Aucune tache. Lancez 'inkwell tasks' d'abord.", "✗".red());
        return;
    }
    // Find next uncompleted task
    let lines: Vec<&str> = project.tasks.lines().filter(|l| l.trim().starts_with("- [ ]")).collect();
    if lines.is_empty() {
        println!("{} Toutes les taches sont completees!", "✓".green());
        return;
    }
    let next = lines[0].trim().strip_prefix("- [ ] ").unwrap_or(lines[0]);
    println!("{} Execution de la tache: {}", "⏳".yellow(), next.cyan());
    let result = llm_call(
        "You are a developer. Implement this task. Write clean code.",
        &format!("Constitution:\n{}\n\nPlan:\n{}\n\nTask: {}", project.constitution, project.plan, next),
    ).await;
    println!("{} Tache executee:", "✓".green());
    println!("\n{}", result);
}

pub async fn checklist() {
    let project = InkwellProject::load().unwrap_or_else(|| InkwellProject::new("projet"));
    println!("{} Generation du checklist...", "⏳".yellow());
    let mut ctx = String::new();
    if !project.specification.is_empty() { ctx.push_str(&format!("## Specification\n{}\n\n", project.specification)); }
    if !project.plan.is_empty() { ctx.push_str(&format!("## Plan\n{}\n\n", project.plan)); }
    let result = llm_call(
        "You are a quality auditor. Generate a quality checklist. Format: - [ ] CHK001 Description. Write in the user's language.",
        &format!("{}Generate checklist.", ctx),
    ).await;
    println!("{} Checklist genere:", "✓".green());
    println!("\n{}", result);
}

pub async fn analyze() {
    let project = InkwellProject::load().unwrap_or_else(|| InkwellProject::new("projet"));
    println!("{} Analyse du plan...", "⏳".yellow());
    let mut ctx = String::new();
    if !project.constitution.is_empty() { ctx.push_str(&format!("## Constitution\n{}\n\n", project.constitution)); }
    if !project.specification.is_empty() { ctx.push_str(&format!("## Specification\n{}\n\n", project.specification)); }
    if !project.plan.is_empty() { ctx.push_str(&format!("## Plan\n{}\n\n", project.plan)); }
    let result = llm_call(
        "You are an architecture reviewer. Analyze for completeness, consistency, constitutional compliance. List issues with severity.",
        &format!("{}Analyze this plan.", ctx),
    ).await;
    println!("{} Analyse:", "✓".green());
    println!("\n{}", result);
}

pub fn validate() {
    let project = InkwellProject::load().unwrap_or_else(|| InkwellProject::new("projet"));
    println!("{}", "Validation SDD:".bold());
    let checks = [
        ("Constitution", &project.constitution, vec!["principes", "Governance"]),
        ("Specification", &project.specification, vec!["User Story", "Given", "FR-"]),
        ("Plan", &project.plan, vec!["Technical Context", "Project Structure"]),
        ("Tasks", &project.tasks, vec!["- [ ] T", "Phase"]),
    ];
    let mut total_issues = 0;
    for (name, content, markers) in &checks {
        if content.is_empty() {
            println!("  {} {} — {}", "✗".red(), name, "vide".red());
            total_issues += 1;
        } else {
            let missing: Vec<&&str> = markers.iter().filter(|m| !content.contains(**m)).collect();
            if missing.is_empty() {
                println!("  {} {} — {} ({} car.)", "✓".green(), name, "OK".green(), content.len());
            } else {
                println!("  {} {} — manque: {}", "⚠".yellow(), name, missing.iter().map(|m| m.to_string()).collect::<Vec<_>>().join(", ").yellow());
                total_issues += missing.len();
            }
        }
    }
    if total_issues == 0 { println!("\n{}", "Toutes les phases sont valides!".green().bold()); }
    else { println!("\n{} probleme(s) detecte(s)", total_issues.to_string().yellow()); }
}
