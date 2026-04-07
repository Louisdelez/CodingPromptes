use std::path::Path;
use std::sync::{Arc, Mutex};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComputeDevice {
    Cpu,
    Gpu,
}

pub struct WhisperEngine {
    ctx: Arc<Mutex<Option<WhisperContext>>>,
    current_model: Arc<Mutex<Option<String>>>,
    device: Arc<Mutex<ComputeDevice>>,
}

impl WhisperEngine {
    pub fn new() -> Self {
        Self {
            ctx: Arc::new(Mutex::new(None)),
            current_model: Arc::new(Mutex::new(None)),
            device: Arc::new(Mutex::new(ComputeDevice::Cpu)),
        }
    }

    pub fn load_model(&self, model_path: &Path, use_gpu: bool) -> Result<(), String> {
        let path_str = model_path
            .to_str()
            .ok_or("Invalid model path")?
            .to_string();

        let mut params = WhisperContextParameters::default();
        params.use_gpu(use_gpu);

        let ctx = WhisperContext::new_with_params(&path_str, params)
            .map_err(|e| format!("Failed to load model: {e}"))?;

        *self.ctx.lock().unwrap() = Some(ctx);
        *self.current_model.lock().unwrap() = Some(path_str);
        *self.device.lock().unwrap() = if use_gpu { ComputeDevice::Gpu } else { ComputeDevice::Cpu };
        Ok(())
    }

    pub fn is_loaded(&self) -> bool {
        self.ctx.lock().unwrap().is_some()
    }

    pub fn current_model_path(&self) -> Option<String> {
        self.current_model.lock().unwrap().clone()
    }

    pub fn current_device(&self) -> ComputeDevice {
        *self.device.lock().unwrap()
    }

    pub fn transcribe(&self, audio_data: &[f32], language: Option<&str>) -> Result<String, String> {
        let ctx_guard = self.ctx.lock().unwrap();
        let ctx = ctx_guard.as_ref().ok_or("No model loaded")?;

        let mut state = ctx.create_state().map_err(|e| format!("State error: {e}"))?;

        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });

        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);
        params.set_translate(false);
        params.set_no_context(true);
        params.set_single_segment(false);
        params.set_n_threads(num_cpus().min(8) as i32);

        if let Some(lang) = language {
            params.set_language(Some(lang));
        } else {
            params.set_language(Some("auto"));
        }

        state
            .full(params, audio_data)
            .map_err(|e| format!("Transcription error: {e}"))?;

        let num_segments = state.full_n_segments().map_err(|e| format!("Segment error: {e}"))?;
        let mut result = String::new();

        for i in 0..num_segments {
            if let Ok(text) = state.full_get_segment_text(i) {
                result.push_str(&text);
            }
        }

        Ok(result.trim().to_string())
    }
}

fn num_cpus() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
}

impl Clone for WhisperEngine {
    fn clone(&self) -> Self {
        Self {
            ctx: Arc::clone(&self.ctx),
            current_model: Arc::clone(&self.current_model),
            device: Arc::clone(&self.device),
        }
    }
}
