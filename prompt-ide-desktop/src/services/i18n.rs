use std::collections::HashMap;

pub struct I18n {
    lang: String,
    translations: HashMap<&'static str, (&'static str, &'static str)>, // key -> (fr, en)
}

impl I18n {
    pub fn new(lang: &str) -> Self {
        let mut t = HashMap::new();
        // App
        t.insert("app.title", ("Prompt IDE", "Prompt IDE"));
        t.insert("app.saving", ("Sauvegarde...", "Saving..."));
        // Tabs
        t.insert("tab.library", ("Bibliotheque", "Library"));
        t.insert("tab.frameworks", ("Frameworks", "Frameworks"));
        t.insert("tab.versions", ("Versions", "Versions"));
        t.insert("tab.preview", ("Preview", "Preview"));
        t.insert("tab.playground", ("Playground", "Playground"));
        t.insert("tab.settings", ("Parametres", "Settings"));
        // Blocks
        t.insert("block.add", ("Ajouter un bloc", "Add a block"));
        t.insert("block.delete", ("Supprimer", "Delete"));
        t.insert("block.enable", ("Activer", "Enable"));
        t.insert("block.disable", ("Desactiver", "Disable"));
        // Variables
        t.insert("variables.title", ("Variables", "Variables"));
        t.insert("variables.value_for", ("Valeur pour", "Value for"));
        // Counter
        t.insert("counter.tokens", ("tokens", "tokens"));
        t.insert("counter.chars", ("car.", "chars"));
        t.insert("counter.words", ("mots", "words"));
        t.insert("counter.lines", ("lignes", "lines"));
        // Preview
        t.insert("preview.title", ("Prompt compile", "Compiled prompt"));
        t.insert("preview.copy", ("Copier", "Copy"));
        t.insert("preview.copied", ("Copie !", "Copied!"));
        t.insert("preview.empty", ("Ecrivez dans les blocs pour voir le prompt...", "Write in the blocks to see the prompt..."));
        // Playground
        t.insert("playground.execute", ("Executer", "Execute"));
        t.insert("playground.executing", ("Execution...", "Executing..."));
        t.insert("playground.select_model", ("Selectionnez un modele", "Select a model"));
        t.insert("playground.temperature", ("Temperature", "Temperature"));
        t.insert("playground.max_tokens", ("Max tokens", "Max tokens"));
        t.insert("playground.free", ("gratuit", "free"));
        // Library
        t.insert("library.search", ("Rechercher...", "Search..."));
        t.insert("library.new_workspace", ("Nouveau projet", "New project"));
        t.insert("library.new_prompt", ("Nouveau prompt", "New prompt"));
        t.insert("library.free_prompts", ("Prompts libres", "Free prompts"));
        t.insert("library.empty", ("Rien ici encore", "Nothing here yet"));
        t.insert("library.delete", ("Supprimer", "Delete"));
        t.insert("library.duplicate", ("Dupliquer", "Duplicate"));
        // Versions
        t.insert("versions.title", ("Versions", "Versions"));
        t.insert("versions.save", ("Sauver", "Save"));
        t.insert("versions.label", ("Label de version...", "Version label..."));
        t.insert("versions.empty", ("Aucune version", "No versions"));
        t.insert("versions.restore", ("Restaurer", "Restore"));
        // Frameworks
        t.insert("frameworks.title", ("Frameworks", "Frameworks"));
        t.insert("frameworks.builtin", ("Integres", "Built-in"));
        // Settings
        t.insert("settings.title", ("Parametres", "Settings"));
        t.insert("settings.api_keys", ("Cles API", "API Keys"));
        t.insert("settings.theme", ("Theme", "Theme"));
        t.insert("settings.language", ("Langue", "Language"));
        t.insert("settings.local_server", ("Serveur local", "Local server"));
        // Auth
        t.insert("auth.login", ("Connexion", "Sign in"));
        t.insert("auth.register", ("Inscription", "Sign up"));
        t.insert("auth.email", ("Email", "Email"));
        t.insert("auth.password", ("Mot de passe", "Password"));
        t.insert("auth.confirm_password", ("Confirmer", "Confirm"));
        t.insert("auth.display_name", ("Nom", "Name"));
        t.insert("auth.login_button", ("Se connecter", "Sign in"));
        t.insert("auth.register_button", ("Creer un compte", "Create account"));
        t.insert("auth.logout", ("Deconnexion", "Sign out"));
        t.insert("auth.welcome", ("Bienvenue sur Prompt IDE", "Welcome to Prompt IDE"));
        t.insert("auth.subtitle", ("Votre atelier de prompts IA", "Your AI prompt workshop"));
        t.insert("auth.email_exists", ("Cet email est deja utilise.", "This email is already in use."));
        t.insert("auth.invalid_credentials", ("Email ou mot de passe incorrect.", "Invalid email or password."));
        t.insert("auth.password_short", ("Mot de passe trop court (6 min).", "Password too short (6 min)."));
        t.insert("auth.password_mismatch", ("Mots de passe differents.", "Passwords don't match."));
        // Export
        t.insert("export.title", ("Exporter", "Export"));
        t.insert("export.txt", ("Texte brut (.txt)", "Plain text (.txt)"));
        t.insert("export.json", ("JSON complet", "Full JSON"));
        t.insert("export.md", ("Markdown (.md)", "Markdown (.md)"));

        Self {
            lang: lang.to_string(),
            translations: t,
        }
    }

    pub fn t(&self, key: &str) -> &'static str {
        self.translations.get(key).map(|(fr, en)| {
            if self.lang == "fr" { *fr } else { *en }
        }).unwrap_or("???")
    }

    pub fn set_lang(&mut self, lang: &str) {
        self.lang = lang.to_string();
    }

    pub fn lang(&self) -> &str {
        &self.lang
    }
}
