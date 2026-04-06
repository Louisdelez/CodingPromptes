use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub size_mb: u64,
    pub url: String,
    pub filename: String,
    pub description: String,
}

pub fn available_models() -> Vec<ModelInfo> {
    vec![
        ModelInfo {
            id: "tiny".into(),
            name: "Whisper Tiny".into(),
            size_mb: 75,
            url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.bin".into(),
            filename: "ggml-tiny.bin".into(),
            description: "39M params — Ultra rapide, qualite basique, ideal CPU faible".into(),
        },
        ModelInfo {
            id: "base".into(),
            name: "Whisper Base".into(),
            size_mb: 142,
            url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin".into(),
            filename: "ggml-base.bin".into(),
            description: "74M params — Rapide, bonne qualite pour CPU".into(),
        },
        ModelInfo {
            id: "small".into(),
            name: "Whisper Small".into(),
            size_mb: 466,
            url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin".into(),
            filename: "ggml-small.bin".into(),
            description: "244M params — Bon compromis vitesse/qualite".into(),
        },
        ModelInfo {
            id: "medium".into(),
            name: "Whisper Medium".into(),
            size_mb: 1500,
            url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium.bin".into(),
            filename: "ggml-medium.bin".into(),
            description: "769M params — Haute qualite, necessite bon CPU ou GPU".into(),
        },
        ModelInfo {
            id: "large-v3".into(),
            name: "Whisper Large v3".into(),
            size_mb: 2900,
            url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3.bin".into(),
            filename: "ggml-large-v3.bin".into(),
            description: "1.5B params — Meilleure qualite, GPU 10Go+ recommande".into(),
        },
        ModelInfo {
            id: "large-v3-turbo".into(),
            name: "Whisper Large v3 Turbo".into(),
            size_mb: 1500,
            url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-turbo.bin".into(),
            filename: "ggml-large-v3-turbo.bin".into(),
            description: "809M params — Quasi aussi precis que v3, 8x plus rapide".into(),
        },
    ]
}

pub fn models_dir() -> PathBuf {
    let base = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("inkwell-server").join("models")
}

pub fn model_path(model: &ModelInfo) -> PathBuf {
    models_dir().join(&model.filename)
}

pub fn is_model_installed(model: &ModelInfo) -> bool {
    model_path(model).exists()
}

pub fn installed_models() -> Vec<ModelInfo> {
    available_models()
        .into_iter()
        .filter(|m| is_model_installed(m))
        .collect()
}
