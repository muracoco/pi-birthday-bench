use anyhow::{bail, Result};

use crate::pi::compute_pi_fractional_digits;
use crate::result::BackendMode;

pub const SELECTABLE_BACKENDS: [BackendMode; 6] = [
    BackendMode::CpuSingle,
    BackendMode::CudaCompute,
    BackendMode::CudaSearchOnly,
    BackendMode::Hip,
    BackendMode::OpenCl,
    BackendMode::Vulkan,
];

pub trait PiBackend {
    fn name(&self) -> &'static str;
    fn gpu_role(&self) -> GpuRole;
    fn is_available(&self) -> bool;
    fn compute_digits(&self, digits: usize) -> Result<String>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuRole {
    None,
    SearchOnly,
    PiCompute,
    Unavailable,
}

impl GpuRole {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::SearchOnly => "search_only",
            Self::PiCompute => "pi_compute",
            Self::Unavailable => "unavailable",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendAvailability {
    Available,
    NotImplemented,
    FeatureEnabledNotImplemented { feature: &'static str },
    MissingFeature { feature: &'static str },
}

impl BackendAvailability {
    fn list_status(self) -> String {
        match self {
            Self::Available => "available".to_owned(),
            Self::NotImplemented => "unavailable, not implemented".to_owned(),
            Self::FeatureEnabledNotImplemented { feature } => {
                format!("unavailable, {feature} feature enabled but not implemented")
            }
            Self::MissingFeature { feature } => {
                format!("unavailable, build with --features {feature}")
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BackendInfo {
    pub mode: BackendMode,
    pub gpu_role: GpuRole,
    pub availability: BackendAvailability,
}

impl BackendInfo {
    pub fn name(self) -> &'static str {
        self.mode.as_str()
    }

    fn list_line(self) -> String {
        format!("- {}: {}", self.name(), self.availability.list_status())
    }
}

pub fn backend_info(mode: BackendMode) -> BackendInfo {
    match mode {
        BackendMode::CpuSingle => BackendInfo {
            mode,
            gpu_role: GpuRole::None,
            availability: BackendAvailability::Available,
        },
        BackendMode::CudaCompute => BackendInfo {
            mode,
            gpu_role: GpuRole::PiCompute,
            availability: gpu_availability("cuda"),
        },
        BackendMode::CudaSearchOnly => BackendInfo {
            mode,
            gpu_role: GpuRole::SearchOnly,
            availability: gpu_availability("cuda"),
        },
        BackendMode::Hip => BackendInfo {
            mode,
            gpu_role: GpuRole::Unavailable,
            availability: gpu_availability("hip"),
        },
        BackendMode::OpenCl => BackendInfo {
            mode,
            gpu_role: GpuRole::Unavailable,
            availability: gpu_availability("opencl"),
        },
        BackendMode::Vulkan => BackendInfo {
            mode,
            gpu_role: GpuRole::Unavailable,
            availability: gpu_availability("vulkan"),
        },
    }
}

pub fn backend_list_text() -> String {
    let lines = [
        "available backends:".to_owned(),
        backend_info(SELECTABLE_BACKENDS[0]).list_line(),
        "- cpu-multi: unavailable, not implemented".to_owned(),
        backend_info(SELECTABLE_BACKENDS[1]).list_line(),
        backend_info(SELECTABLE_BACKENDS[2]).list_line(),
        backend_info(SELECTABLE_BACKENDS[3]).list_line(),
        backend_info(SELECTABLE_BACKENDS[4]).list_line(),
        backend_info(SELECTABLE_BACKENDS[5]).list_line(),
    ];

    lines.join("\n")
}

pub fn unavailable_backend_error(mode: BackendMode) -> Result<()> {
    let info = backend_info(mode);
    match info.availability {
        BackendAvailability::Available => Ok(()),
        BackendAvailability::NotImplemented
        | BackendAvailability::FeatureEnabledNotImplemented { .. } => bail!(
            "backend '{}' is not implemented yet.\nhint: use --backend cpu-single",
            mode.as_str()
        ),
        BackendAvailability::MissingFeature { feature } => bail!(
            "backend '{}' is not available in this build.\nhint: rebuild with --features {} when this backend is implemented",
            mode.as_str(),
            feature
        ),
    }
}

pub struct CpuSingleBackend;

impl PiBackend for CpuSingleBackend {
    fn name(&self) -> &'static str {
        BackendMode::CpuSingle.as_str()
    }

    fn gpu_role(&self) -> GpuRole {
        GpuRole::None
    }

    fn is_available(&self) -> bool {
        true
    }

    fn compute_digits(&self, digits: usize) -> Result<String> {
        compute_pi_fractional_digits(digits)
    }
}

pub struct CpuMultiBackend;

impl PiBackend for CpuMultiBackend {
    fn name(&self) -> &'static str {
        "cpu-multi"
    }

    fn gpu_role(&self) -> GpuRole {
        GpuRole::None
    }

    fn is_available(&self) -> bool {
        false
    }

    fn compute_digits(&self, _digits: usize) -> Result<String> {
        bail!("backend 'cpu-multi' is not implemented yet.\nhint: use --backend cpu-single")
    }
}

fn gpu_availability(feature: &'static str) -> BackendAvailability {
    match feature {
        "cuda" if cfg!(feature = "cuda") => {
            BackendAvailability::FeatureEnabledNotImplemented { feature }
        }
        "hip" if cfg!(feature = "hip") => {
            BackendAvailability::FeatureEnabledNotImplemented { feature }
        }
        "opencl" if cfg!(feature = "opencl") => {
            BackendAvailability::FeatureEnabledNotImplemented { feature }
        }
        "vulkan" if cfg!(feature = "vulkan") => {
            BackendAvailability::FeatureEnabledNotImplemented { feature }
        }
        _ => BackendAvailability::MissingFeature { feature },
    }
}

#[cfg(test)]
mod tests {
    use super::backend_list_text;

    #[test]
    fn backend_list_preserves_current_default_output() {
        let text = backend_list_text();

        assert!(text.contains("available backends:"));
        assert!(text.contains("- cpu-single: available"));
        assert!(text.contains("- cpu-multi: unavailable, not implemented"));
        assert!(text.contains("- cuda-compute: unavailable, build with --features cuda"));
        assert!(text.contains("- cuda-search-only: unavailable, build with --features cuda"));
        assert!(text.contains("- hip: unavailable, build with --features hip"));
        assert!(text.contains("- opencl: unavailable, build with --features opencl"));
        assert!(text.contains("- vulkan: unavailable, build with --features vulkan"));
    }
}
