use std::process::Command;

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct GpuInfo {
    pub name: String,
    pub vram_mb: u64,
}

pub fn gpu_info() -> Option<GpuInfo> {
    let output = Command::new("nvidia-smi")
        .args([
            "--query-gpu=name,memory.total",
            "--format=csv,noheader,nounits",
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    parse_gpu_info(&String::from_utf8_lossy(&output.stdout))
}

fn parse_gpu_info(stdout: &str) -> Option<GpuInfo> {
    let line = stdout.lines().find(|line| !line.trim().is_empty())?;
    let mut parts = line.splitn(2, ',');

    let name = parts.next()?.trim().to_string();
    let vram_mb = parts.next()?.trim().parse::<u64>().ok()?;

    if name.is_empty() {
        return None;
    }

    Some(GpuInfo { name, vram_mb })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_gpu_line() {
        let info = parse_gpu_info("NVIDIA GeForce RTX 4060 Laptop GPU, 8188\n").unwrap();
        assert_eq!(info.name, "NVIDIA GeForce RTX 4060 Laptop GPU");
        assert_eq!(info.vram_mb, 8188);
    }

    #[test]
    fn takes_first_gpu() {
        let info = parse_gpu_info("GPU A, 8192\nGPU B, 4096\n").unwrap();
        assert_eq!(info.name, "GPU A");
    }

    #[test]
    fn rejects_invalid_output() {
        assert!(parse_gpu_info("").is_none());
        assert!(parse_gpu_info("NVIDIA GPU\n").is_none());
        assert!(parse_gpu_info(", 8192\n").is_none());
    }
}
