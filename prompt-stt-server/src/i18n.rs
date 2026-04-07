#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    Fr,
    En,
}

impl std::fmt::Display for Lang {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Lang::Fr => write!(f, "Francais"),
            Lang::En => write!(f, "English"),
        }
    }
}

impl Lang {
    pub const ALL: [Lang; 2] = [Lang::Fr, Lang::En];
}

pub struct T;

impl T {
    // Header
    pub fn gpu_server(lang: Lang) -> &'static str {
        match lang { Lang::Fr => "Serveur GPU", Lang::En => "GPU Server" }
    }
    pub fn online(lang: Lang) -> &'static str {
        match lang { Lang::Fr => "En ligne", Lang::En => "Online" }
    }
    pub fn offline(lang: Lang) -> &'static str {
        match lang { Lang::Fr => "Hors ligne", Lang::En => "Offline" }
    }

    // Server card
    pub fn server(lang: Lang) -> &'static str {
        match lang { Lang::Fr => "Serveur", Lang::En => "Server" }
    }
    pub fn proxy_llm(lang: Lang) -> &'static str {
        match lang { Lang::Fr => "Proxy LLM", Lang::En => "LLM Proxy" }
    }

    // Ollama card
    pub fn disabled(lang: Lang) -> &'static str {
        match lang { Lang::Fr => "Desactive", Lang::En => "Disabled" }
    }
    pub fn connected_models(lang: Lang, count: usize) -> String {
        match lang {
            Lang::Fr => format!("Connecte — {count} modele(s)"),
            Lang::En => format!("Connected — {count} model(s)"),
        }
    }
    pub fn disconnected(lang: Lang) -> &'static str {
        match lang { Lang::Fr => "Deconnecte", Lang::En => "Disconnected" }
    }
    pub fn test(lang: Lang) -> &'static str {
        match lang { Lang::Fr => "Tester", Lang::En => "Test" }
    }

    // Whisper card
    pub fn ready(lang: Lang) -> &'static str {
        match lang { Lang::Fr => "Pret", Lang::En => "Ready" }
    }
    pub fn loading(lang: Lang) -> &'static str {
        match lang { Lang::Fr => "Chargement...", Lang::En => "Loading..." }
    }
    pub fn no_model_loaded(lang: Lang) -> &'static str {
        match lang { Lang::Fr => "Aucun modele charge", Lang::En => "No model loaded" }
    }
    pub fn select_model(lang: Lang) -> &'static str {
        match lang { Lang::Fr => "Choisir un modele...", Lang::En => "Select model..." }
    }
    pub fn load(lang: Lang) -> &'static str {
        match lang { Lang::Fr => "Charger", Lang::En => "Load" }
    }

    // Downloads card
    pub fn whisper_models(lang: Lang) -> &'static str {
        match lang { Lang::Fr => "Modeles Whisper", Lang::En => "Whisper Models" }
    }
    pub fn download(lang: Lang) -> &'static str {
        match lang { Lang::Fr => "Telecharger", Lang::En => "Download" }
    }
    pub fn installed(lang: Lang) -> &'static str {
        match lang { Lang::Fr => "Installe", Lang::En => "Installed" }
    }

    // Log card
    pub fn activity(lang: Lang) -> &'static str {
        match lang { Lang::Fr => "Activite", Lang::En => "Activity" }
    }

    // Log messages
    pub fn server_started(lang: Lang) -> &'static str {
        match lang { Lang::Fr => "Inkwell Serveur GPU demarre", Lang::En => "Inkwell GPU Server started" }
    }
    pub fn listening_on(lang: Lang, port: u16) -> String {
        match lang {
            Lang::Fr => format!("Ecoute sur le port {port}"),
            Lang::En => format!("Listening on port {port}"),
        }
    }
    pub fn server_started_on(lang: Lang, port: u16) -> String {
        match lang {
            Lang::Fr => format!("Serveur demarre sur le port {port}"),
            Lang::En => format!("Server started on port {port}"),
        }
    }
    pub fn loading_model(lang: Lang, name: &str) -> String {
        match lang {
            Lang::Fr => format!("Chargement de {name}..."),
            Lang::En => format!("Loading {name}..."),
        }
    }
    pub fn model_loaded(lang: Lang) -> &'static str {
        match lang { Lang::Fr => "Modele Whisper charge", Lang::En => "Whisper model loaded" }
    }
    pub fn downloading_model(lang: Lang, name: &str) -> String {
        match lang {
            Lang::Fr => format!("Telechargement de {name}..."),
            Lang::En => format!("Downloading {name}..."),
        }
    }
    pub fn download_complete(lang: Lang) -> &'static str {
        match lang { Lang::Fr => "Telechargement termine", Lang::En => "Download complete" }
    }
    pub fn ollama_connected(lang: Lang, count: usize) -> String {
        match lang {
            Lang::Fr => format!("Ollama connecte ({count} modeles)"),
            Lang::En => format!("Ollama connected ({count} models)"),
        }
    }

    // Language selector
    pub fn language(lang: Lang) -> &'static str {
        match lang { Lang::Fr => "Langue", Lang::En => "Language" }
    }
}

/// Detect system language, fallback to English
pub fn detect_system_lang() -> Lang {
    std::env::var("LANG")
        .or_else(|_| std::env::var("LC_ALL"))
        .or_else(|_| std::env::var("LANGUAGE"))
        .map(|v| if v.starts_with("fr") { Lang::Fr } else { Lang::En })
        .unwrap_or(Lang::En)
}
