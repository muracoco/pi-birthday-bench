#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendMode {
    CpuSingle,
    CpuMulti,
    CudaCompute,
    CudaSearchOnly,
    Hip,
    OpenCl,
    Vulkan,
}

impl BackendMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::CpuSingle => "cpu-single",
            Self::CpuMulti => "cpu-multi",
            Self::CudaCompute => "cuda-compute",
            Self::CudaSearchOnly => "cuda-search-only",
            Self::Hip => "hip",
            Self::OpenCl => "opencl",
            Self::Vulkan => "vulkan",
        }
    }
}

impl std::str::FromStr for BackendMode {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "cpu-single" => Ok(Self::CpuSingle),
            "cpu-multi" => Ok(Self::CpuMulti),
            "cuda-compute" => Ok(Self::CudaCompute),
            "cuda-search-only" => Ok(Self::CudaSearchOnly),
            "hip" => Ok(Self::Hip),
            "opencl" => Ok(Self::OpenCl),
            "vulkan" => Ok(Self::Vulkan),
            _ => Err(format!("unsupported backend '{value}'")),
        }
    }
}

impl std::fmt::Display for BackendMode {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunPhase {
    Idle,
    Validating,
    ComputingPi,
    Searching,
    Completed,
    Cancelled,
    Error,
}

impl RunPhase {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Validating => "validating",
            Self::ComputingPi => "computing_pi",
            Self::Searching => "searching",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
            Self::Error => "error",
        }
    }
}

#[derive(Debug, Clone)]
pub struct RunConfig {
    pub target: String,
    pub max_digits: usize,
    pub chunk: usize,
    pub backend: BackendMode,
    pub benchmark_only: bool,
    pub threads: Option<usize>,
    pub verify: bool,
}

impl RunConfig {
    pub fn validate(&self) -> anyhow::Result<()> {
        crate::date::validate_yyyymmdd(&self.target)
            .map_err(|error| anyhow::anyhow!("invalid target '{}': {error}", self.target))?;

        if self.max_digits == 0 {
            anyhow::bail!("max_digits must be greater than 0");
        }
        if self.chunk == 0 {
            anyhow::bail!("chunk must be greater than 0");
        }
        if self.threads == Some(0) {
            anyhow::bail!("threads must be greater than 0");
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerificationStatus {
    Passed,
    Failed,
    Skipped,
}

impl VerificationStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Passed => "passed",
            Self::Failed => "failed",
            Self::Skipped => "skipped",
        }
    }
}

#[derive(Debug, Clone)]
pub enum ProgressEvent {
    Started {
        config: RunConfig,
    },
    PhaseChanged {
        phase: RunPhase,
    },
    Progress {
        range_start: usize,
        range_end: usize,
        digits_computed: usize,
        elapsed_seconds: f64,
        digits_per_second: f64,
    },
    Completed(BenchmarkResult),
    Cancelled,
    Failed(String),
}

#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub target: String,
    pub found: bool,
    pub first_position: Option<usize>,
    pub backend: String,
    pub algorithm: String,
    pub digits_computed: usize,
    pub elapsed_seconds: f64,
    pub digits_per_second: f64,
    pub chunks_processed: usize,
    pub threads: Option<usize>,
    pub cpu_model: Option<String>,
    pub logical_cpu_count: Option<usize>,
    pub physical_cpu_count: Option<usize>,
    pub gpu_role: String,
    pub memory_total_mb: Option<u64>,
    pub memory_peak_mb: Option<u64>,
    pub verification_status: VerificationStatus,
}

impl BenchmarkResult {
    pub fn as_text(&self) -> String {
        format!(
            "\
target: {}
found: {}
first_position: {}
backend: {}
algorithm: {}
digits_computed: {}
elapsed_seconds: {:.6}
digits_per_second: {:.1}
chunks_processed: {}
threads: {}
cpu_model: {}
logical_cpu_count: {}
physical_cpu_count: {}
memory_total_mb: {}
memory_peak_mb: {}
verification_status: {}",
            self.target,
            self.found,
            self.first_position
                .map(|position| position.to_string())
                .unwrap_or_else(|| "null".to_owned()),
            self.backend,
            self.algorithm,
            self.digits_computed,
            self.elapsed_seconds,
            self.digits_per_second,
            self.chunks_processed,
            option_usize_text(self.threads),
            option_string_text(self.cpu_model.as_deref()),
            option_usize_text(self.logical_cpu_count),
            option_usize_text(self.physical_cpu_count),
            option_u64_text(self.memory_total_mb),
            option_u64_text(self.memory_peak_mb),
            self.verification_status.as_str()
        )
    }
}

impl BenchmarkResult {
    pub fn as_json(&self) -> String {
        serde_json::json!({
            "target": self.target,
            "found": self.found,
            "first_position": self.first_position,
            "backend": self.backend,
            "algorithm": self.algorithm,
            "digits_computed": self.digits_computed,
            "elapsed_seconds": self.elapsed_seconds,
            "digits_per_second": self.digits_per_second,
            "chunks_processed": self.chunks_processed,
            "threads": self.threads,
            "cpu_model": self.cpu_model,
            "logical_cpu_count": self.logical_cpu_count,
            "physical_cpu_count": self.physical_cpu_count,
            "gpu_name": null,
            "gpu_role": self.gpu_role.as_str(),
            "memory_total_mb": self.memory_total_mb,
            "memory_peak_mb": self.memory_peak_mb,
            "verification_status": self.verification_status.as_str(),
        })
        .to_string()
    }
}

fn option_string_text(value: Option<&str>) -> String {
    value.unwrap_or("null").to_owned()
}

fn option_usize_text(value: Option<usize>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "null".to_owned())
}

fn option_u64_text(value: Option<u64>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "null".to_owned())
}

#[cfg(test)]
mod tests {
    use super::{BackendMode, BenchmarkResult, RunConfig, VerificationStatus};

    #[test]
    fn run_config_accepts_valid_values() {
        let config = RunConfig {
            target: "20240628".to_owned(),
            max_digits: 100,
            chunk: 10,
            backend: BackendMode::CpuSingle,
            benchmark_only: false,
            threads: None,
            verify: false,
        };

        assert!(config.validate().is_ok());
    }

    #[test]
    fn run_config_rejects_invalid_values() {
        let invalid_date = RunConfig {
            target: "20240631".to_owned(),
            max_digits: 100,
            chunk: 10,
            backend: BackendMode::CpuSingle,
            benchmark_only: false,
            threads: None,
            verify: false,
        };
        assert!(invalid_date.validate().is_err());

        let zero_digits = RunConfig {
            target: "20240628".to_owned(),
            max_digits: 0,
            chunk: 10,
            backend: BackendMode::CpuSingle,
            benchmark_only: false,
            threads: None,
            verify: false,
        };
        assert!(zero_digits.validate().is_err());

        let zero_chunk = RunConfig {
            target: "20240628".to_owned(),
            max_digits: 100,
            chunk: 0,
            backend: BackendMode::CpuSingle,
            benchmark_only: false,
            threads: None,
            verify: false,
        };
        assert!(zero_chunk.validate().is_err());

        let zero_threads = RunConfig {
            target: "20240628".to_owned(),
            max_digits: 100,
            chunk: 10,
            backend: BackendMode::CpuMulti,
            benchmark_only: false,
            threads: Some(0),
            verify: false,
        };
        assert!(zero_threads.validate().is_err());
    }

    #[test]
    fn benchmark_result_json_contains_stable_fields() {
        let result = BenchmarkResult {
            target: "19930628".to_owned(),
            found: true,
            first_position: Some(12_345_678),
            backend: "cpu-single".to_owned(),
            algorithm: "chudnovsky_binary_splitting".to_owned(),
            digits_computed: 20_000_000,
            elapsed_seconds: 12.34,
            digits_per_second: 1_620_745.5,
            chunks_processed: 20,
            threads: None,
            cpu_model: Some("Test CPU".to_owned()),
            logical_cpu_count: Some(8),
            physical_cpu_count: None,
            gpu_role: "none".to_owned(),
            memory_total_mb: Some(16_384),
            memory_peak_mb: None,
            verification_status: VerificationStatus::Skipped,
        };

        let value: serde_json::Value = serde_json::from_str(&result.as_json()).expect("valid JSON");

        assert_eq!(value["target"], "19930628");
        assert_eq!(value["found"], true);
        assert_eq!(value["first_position"], 12_345_678);
        assert_eq!(value["backend"], "cpu-single");
        assert_eq!(value["algorithm"], "chudnovsky_binary_splitting");
        assert_eq!(value["digits_computed"], 20_000_000);
        assert_eq!(value["chunks_processed"], 20);
        assert!(value["threads"].is_null());
        assert_eq!(value["cpu_model"], "Test CPU");
        assert_eq!(value["logical_cpu_count"], 8);
        assert!(value["physical_cpu_count"].is_null());
        assert!(value["gpu_name"].is_null());
        assert_eq!(value["gpu_role"], "none");
        assert_eq!(value["memory_total_mb"], 16_384);
        assert!(value["memory_peak_mb"].is_null());
        assert_eq!(value["verification_status"], "skipped");
    }

    #[test]
    fn verification_status_strings_are_stable() {
        assert_eq!(VerificationStatus::Passed.as_str(), "passed");
        assert_eq!(VerificationStatus::Failed.as_str(), "failed");
        assert_eq!(VerificationStatus::Skipped.as_str(), "skipped");
    }
}
