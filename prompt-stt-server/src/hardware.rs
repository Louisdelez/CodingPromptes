use serde::{Deserialize, Serialize};
use sysinfo::System;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuInfo {
    pub name: String,
    pub cores: usize,
    pub threads: usize,
    pub frequency_mhz: u64,
    pub usage_percent: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RamInfo {
    pub total_gb: f64,
    pub available_gb: f64,
    pub used_gb: f64,
    pub usage_percent: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
    pub name: String,
    pub vram_total_mb: u64,
    pub vram_used_mb: u64,
    pub vram_free_mb: u64,
    pub vram_usage_percent: f32,
    pub driver_version: String,
    pub cuda_version: String,
    pub gpu_utilization: u32,
    pub temperature: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareInfo {
    pub cpu: CpuInfo,
    pub ram: RamInfo,
    pub gpu: Option<GpuInfo>,
    pub os: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceRecommendation {
    CpuOnly,
    GpuRecommended,
    GpuRequired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelHardwareReqs {
    pub model_name: String,
    pub params: String,
    pub cpu_ram_gb: f64,
    pub gpu_vram_gb: f64,
    pub recommendation: DeviceRecommendation,
    pub cpu_note: String,
    pub gpu_note: String,
}

impl HardwareInfo {
    pub fn detect() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        // Need a short delay then re-refresh for CPU usage to be accurate
        std::thread::sleep(std::time::Duration::from_millis(200));
        sys.refresh_cpu_all();
        sys.refresh_memory();

        let cpu = {
            let cpus = sys.cpus();
            let name = if cpus.is_empty() {
                "Unknown".into()
            } else {
                cpus[0].brand().to_string()
            };
            let frequency = if cpus.is_empty() { 0 } else { cpus[0].frequency() };
            let usage = if cpus.is_empty() { 0.0 } else {
                cpus.iter().map(|c| c.cpu_usage()).sum::<f32>() / cpus.len() as f32
            };
            CpuInfo {
                name,
                cores: System::physical_core_count().unwrap_or(0),
                threads: cpus.len(),
                frequency_mhz: frequency,
                usage_percent: usage,
            }
        };

        let total_mem = sys.total_memory() as f64 / 1_073_741_824.0;
        let used_mem = sys.used_memory() as f64 / 1_073_741_824.0;
        let ram = RamInfo {
            total_gb: total_mem,
            available_gb: sys.available_memory() as f64 / 1_073_741_824.0,
            used_gb: used_mem,
            usage_percent: if total_mem > 0.0 { (used_mem / total_mem * 100.0) as f32 } else { 0.0 },
        };

        let gpu = detect_nvidia_gpu();

        let os = format!("{} {}",
            System::name().unwrap_or_else(|| "Unknown".into()),
            System::os_version().unwrap_or_default(),
        );

        HardwareInfo { cpu, ram, gpu, os }
    }

    /// Lightweight refresh — only updates usage stats, not static info
    pub fn refresh(&mut self) {
        let mut sys = System::new();
        sys.refresh_cpu_all();
        std::thread::sleep(std::time::Duration::from_millis(100));
        sys.refresh_cpu_all();
        sys.refresh_memory();

        let cpus = sys.cpus();
        if !cpus.is_empty() {
            self.cpu.usage_percent = cpus.iter().map(|c| c.cpu_usage()).sum::<f32>() / cpus.len() as f32;
        }
        let total_mem = sys.total_memory() as f64 / 1_073_741_824.0;
        let used_mem = sys.used_memory() as f64 / 1_073_741_824.0;
        self.ram.available_gb = sys.available_memory() as f64 / 1_073_741_824.0;
        self.ram.used_gb = used_mem;
        self.ram.usage_percent = if total_mem > 0.0 { (used_mem / total_mem * 100.0) as f32 } else { 0.0 };

        // Refresh GPU stats
        self.gpu = detect_nvidia_gpu();
    }

    pub fn has_gpu(&self) -> bool {
        self.gpu.is_some()
    }

    pub fn gpu_vram_gb(&self) -> f64 {
        self.gpu.as_ref().map(|g| g.vram_total_mb as f64 / 1024.0).unwrap_or(0.0)
    }

    pub fn gpu_vram_free_gb(&self) -> f64 {
        self.gpu.as_ref().map(|g| g.vram_free_mb as f64 / 1024.0).unwrap_or(0.0)
    }
}

fn detect_nvidia_gpu() -> Option<GpuInfo> {
    let output = std::process::Command::new("nvidia-smi")
        .args([
            "--query-gpu=name,memory.total,memory.used,memory.free,driver_version,utilization.gpu,temperature.gpu",
            "--format=csv,noheader,nounits",
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let line = stdout.trim();
    if line.is_empty() {
        return None;
    }

    let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
    if parts.len() < 7 {
        return None;
    }

    let cuda_version = detect_cuda_version();

    let vram_total: u64 = parts[1].parse().unwrap_or(0);
    let vram_used: u64 = parts[2].parse().unwrap_or(0);
    let vram_free: u64 = parts[3].parse().unwrap_or(0);

    Some(GpuInfo {
        name: parts[0].to_string(),
        vram_total_mb: vram_total,
        vram_used_mb: vram_used,
        vram_free_mb: vram_free,
        vram_usage_percent: if vram_total > 0 { vram_used as f32 / vram_total as f32 * 100.0 } else { 0.0 },
        driver_version: parts[4].to_string(),
        cuda_version,
        gpu_utilization: parts[5].parse().unwrap_or(0),
        temperature: parts[6].parse().unwrap_or(0),
    })
}

fn detect_cuda_version() -> String {
    let output = std::process::Command::new("nvidia-smi")
        .output()
        .ok();

    if let Some(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Parse "CUDA Version: 12.4" from nvidia-smi header
        for line in stdout.lines() {
            if let Some(pos) = line.find("CUDA Version:") {
                let ver = line[pos + 14..].trim();
                if let Some(end) = ver.find(|c: char| !c.is_ascii_digit() && c != '.') {
                    return ver[..end].to_string();
                }
                return ver.to_string();
            }
        }
    }
    "N/A".into()
}

/// Get hardware requirements for each Whisper model
pub fn whisper_model_reqs() -> Vec<ModelHardwareReqs> {
    vec![
        ModelHardwareReqs {
            model_name: "Whisper Tiny".into(), params: "39M".into(),
            cpu_ram_gb: 0.3, gpu_vram_gb: 1.0,
            recommendation: DeviceRecommendation::CpuOnly,
            cpu_note: "Excellent — temps reel sur la plupart des CPU".into(),
            gpu_note: "Inutile — le modele est trop petit pour beneficier du GPU".into(),
        },
        ModelHardwareReqs {
            model_name: "Whisper Base".into(), params: "74M".into(),
            cpu_ram_gb: 0.4, gpu_vram_gb: 1.0,
            recommendation: DeviceRecommendation::CpuOnly,
            cpu_note: "Tres rapide sur CPU, GPU non necessaire".into(),
            gpu_note: "Leger gain, pas significatif".into(),
        },
        ModelHardwareReqs {
            model_name: "Whisper Small".into(), params: "244M".into(),
            cpu_ram_gb: 0.9, gpu_vram_gb: 2.0,
            recommendation: DeviceRecommendation::GpuRecommended,
            cpu_note: "Correct — 2-5s par audio de 10s".into(),
            gpu_note: "Recommande — 3-5x plus rapide".into(),
        },
        ModelHardwareReqs {
            model_name: "Whisper Medium".into(), params: "769M".into(),
            cpu_ram_gb: 2.1, gpu_vram_gb: 5.0,
            recommendation: DeviceRecommendation::GpuRecommended,
            cpu_note: "Lent — 10-20s par audio de 10s".into(),
            gpu_note: "Recommande — necessaire pour temps reel".into(),
        },
        ModelHardwareReqs {
            model_name: "Whisper Large v3".into(), params: "1.55B".into(),
            cpu_ram_gb: 3.9, gpu_vram_gb: 10.0,
            recommendation: DeviceRecommendation::GpuRequired,
            cpu_note: "Tres lent — 30s+ par audio de 10s".into(),
            gpu_note: "Requis — 10Go+ VRAM necessaire".into(),
        },
        ModelHardwareReqs {
            model_name: "Whisper Large v3 Turbo".into(), params: "809M".into(),
            cpu_ram_gb: 2.3, gpu_vram_gb: 6.0,
            recommendation: DeviceRecommendation::GpuRecommended,
            cpu_note: "Lent mais meilleur que Large v3".into(),
            gpu_note: "Recommande — presque aussi precis que v3, 8x plus rapide".into(),
        },
    ]
}

/// Check if a model can run on GPU given available VRAM
pub fn can_run_on_gpu(model_vram_gb: f64, hw: &HardwareInfo) -> bool {
    hw.gpu_vram_free_gb() >= model_vram_gb * 0.9 // 10% margin
}

/// Check if a model can run on CPU given available RAM
pub fn can_run_on_cpu(model_ram_gb: f64, hw: &HardwareInfo) -> bool {
    hw.ram.available_gb >= model_ram_gb * 1.2 // 20% margin
}
