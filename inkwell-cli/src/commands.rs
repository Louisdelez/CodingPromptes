use colored::Colorize;
use crate::project::InkwellProject;

pub fn init(name: Option<String>, here: bool) {
    let project_name = name.unwrap_or_else(|| {
        if here { std::env::current_dir().unwrap().file_name().unwrap().to_string_lossy().to_string() }
        else { "nouveau-projet".to_string() }
    });

    let project = InkwellProject::new(&project_name);

    if !here {
        let _ = std::fs::create_dir_all(&project_name);
        let _ = project.save_to_inkwell(std::path::Path::new(&project_name));
        println!("{} Projet {} cree", "✓".green(), project_name.cyan());
        println!("  cd {}", project_name);
    } else {
        let _ = project.save_to_inkwell(std::path::Path::new("."));
        println!("{} Projet initialise dans le repertoire courant", "✓".green());
    }

    let _ = project.save();
    println!("\n{}", "Commandes disponibles:".bold());
    println!("  inkwell constitution  — Definir les principes du projet");
    println!("  inkwell specify       — Creer une specification");
    println!("  inkwell plan          — Creer un plan technique");
    println!("  inkwell tasks         — Generer les taches");
    println!("  inkwell implement     — Executer les taches");
}

pub fn list() {
    let dir = InkwellProject::project_dir();
    println!("{}", "Projets Inkwell:".bold());
    if let Ok(project) = std::fs::read_to_string(dir.join("current-project.json")) {
        if let Ok(p) = serde_json::from_str::<InkwellProject>(&project) {
            let phases = [
                ("Constitution", !p.constitution.is_empty()),
                ("Specification", !p.specification.is_empty()),
                ("Plan", !p.plan.is_empty()),
                ("Tasks", !p.tasks.is_empty()),
                ("Implementation", !p.implementation.is_empty()),
            ];
            println!("  {} {}", "→".cyan(), p.name.bold());
            for (name, done) in &phases {
                let icon = if *done { "✓".green() } else { "○".dimmed() };
                println!("    {} {}", icon, name);
            }
        }
    } else {
        println!("  {}", "Aucun projet. Utilisez 'inkwell init' pour commencer.".dimmed());
    }
}

pub fn status() {
    if let Some(p) = InkwellProject::load() {
        println!("{} {}", "Projet:".bold(), p.name.cyan());
        println!("{} {}", "Cree le:".dimmed(), p.created_at);
        let phases = [
            ("Constitution", &p.constitution),
            ("Specification", &p.specification),
            ("Plan", &p.plan),
            ("Tasks", &p.tasks),
            ("Implementation", &p.implementation),
        ];
        let done = phases.iter().filter(|(_, c)| !c.is_empty()).count();
        println!("{} {}/5 phases completees", "Progression:".bold(), done);
        for (name, content) in &phases {
            let status = if content.is_empty() { "vide".dimmed().to_string() } else { format!("{} car.", content.len()).green().to_string() };
            println!("  {} — {}", name, status);
        }
        if !p.steering.is_empty() {
            println!("\n{}", "Steering rules:".bold());
            for (name, content) in &p.steering {
                println!("  {} — {} car.", name.cyan(), content.len());
            }
        }
    } else {
        println!("{}", "Aucun projet actif. Utilisez 'inkwell init'.".yellow());
    }
}

pub fn help() {
    println!("{}", "Inkwell CLI — Spec-Driven Development".bold().cyan());
    println!();
    println!("{}", "COMMANDES SDD:".bold());
    println!("  {} {}", "inkwell constitution".cyan(), "Definir les principes du projet");
    println!("  {} {}", "inkwell specify".cyan(), "    Creer une specification");
    println!("  {} {}", "inkwell plan".cyan(), "       Creer un plan technique");
    println!("  {} {}", "inkwell tasks".cyan(), "      Generer les taches");
    println!("  {} {}", "inkwell implement".cyan(), "  Executer les taches");
    println!("  {} {}", "inkwell checklist".cyan(), "  Generer un checklist qualite");
    println!("  {} {}", "inkwell analyze".cyan(), "    Auditer le plan");
    println!("  {} {}", "inkwell validate".cyan(), "   Valider toutes les phases");
    println!();
    println!("{}", "GESTION:".bold());
    println!("  {} {}", "inkwell init".cyan(), "       Initialiser un projet");
    println!("  {} {}", "inkwell list".cyan(), "       Lister les projets");
    println!("  {} {}", "inkwell status".cyan(), "     Statut du projet");
}
