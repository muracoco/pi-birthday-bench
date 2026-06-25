#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendMode {
    CpuSingle,
}

impl BackendMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::CpuSingle => "cpu-single",
        }
    }
}

impl std::str::FromStr for BackendMode {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "cpu-single" => Ok(Self::CpuSingle),
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

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum ProgressEvent {
    Started { config: RunConfig },
    PhaseChanged { phase: RunPhase },
    Progress {
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
chunks_processed: {}",
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
            self.chunks_processed
        )
    }
}

#[cfg(feature = "gui")]
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
        })
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::{BackendMode, RunConfig};

    #[test]
    fn run_config_accepts_valid_values() {
        let config = RunConfig {
            target: "20240628".to_owned(),
            max_digits: 100,
            chunk: 10,
            backend: BackendMode::CpuSingle,
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
        };
        assert!(invalid_date.validate().is_err());

        let zero_digits = RunConfig {
            target: "20240628".to_owned(),
            max_digits: 0,
            chunk: 10,
            backend: BackendMode::CpuSingle,
        };
        assert!(zero_digits.validate().is_err());

        let zero_chunk = RunConfig {
            target: "20240628".to_owned(),
            max_digits: 100,
            chunk: 0,
            backend: BackendMode::CpuSingle,
        };
        assert!(zero_chunk.validate().is_err());
    }
}
