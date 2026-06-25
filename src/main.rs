use std::process::ExitCode;
use std::sync::atomic::AtomicBool;

use anyhow::{bail, Result};
use clap::{Parser, ValueEnum};

use pi_birthday_bench::job::run_job;
use pi_birthday_bench::result::{BackendMode as CoreBackendMode, ProgressEvent, RunConfig};

#[derive(Debug, Clone, Copy, ValueEnum)]
enum BackendMode {
    CpuSingle,
    #[value(name = "cuda-compute")]
    CudaCompute,
    #[value(name = "cuda-search-only")]
    CudaSearchOnly,
    Hip,
    #[value(name = "opencl")]
    OpenCl,
    Vulkan,
}

impl From<BackendMode> for CoreBackendMode {
    fn from(value: BackendMode) -> Self {
        match value {
            BackendMode::CpuSingle => Self::CpuSingle,
            BackendMode::CudaCompute => Self::CudaCompute,
            BackendMode::CudaSearchOnly => Self::CudaSearchOnly,
            BackendMode::Hip => Self::Hip,
            BackendMode::OpenCl => Self::OpenCl,
            BackendMode::Vulkan => Self::Vulkan,
        }
    }
}

#[derive(Debug, Parser)]
#[command(name = "pi-birthday-bench")]
#[command(about = "Find a YYYYMMDD pattern in the fractional digits of pi")]
struct Cli {
    #[arg(long, required_unless_present = "list_backends")]
    target: Option<String>,

    #[arg(long, required_unless_present = "list_backends")]
    max_digits: Option<usize>,

    #[arg(long, default_value_t = 1_000_000)]
    chunk: usize,

    #[arg(long, value_enum, default_value_t = BackendMode::CpuSingle)]
    backend: BackendMode,

    #[arg(long)]
    no_progress: bool,

    #[arg(long)]
    json: bool,

    #[arg(long)]
    benchmark_only: bool,

    #[arg(long)]
    list_backends: bool,
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err:#}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    if cli.list_backends {
        println!("{}", list_backends_text());
        return Ok(());
    }

    let Some(target) = cli.target else {
        bail!("--target is required unless --list-backends is specified");
    };
    let Some(max_digits) = cli.max_digits else {
        bail!("--max-digits is required unless --list-backends is specified");
    };

    let config = RunConfig {
        target,
        max_digits,
        chunk: cli.chunk,
        backend: cli.backend.into(),
        benchmark_only: cli.benchmark_only,
    };
    let cancel_requested = AtomicBool::new(false);

    let result = run_job(config, &cancel_requested, |event| {
        if cli.no_progress || cli.json {
            return;
        }

        match event {
            ProgressEvent::PhaseChanged { phase } => {
                eprintln!("phase={}", phase.as_str());
            }
            ProgressEvent::Progress {
                digits_computed,
                elapsed_seconds,
                digits_per_second,
            } => {
                eprintln!(
                    "digits_computed={digits_computed} elapsed={elapsed_seconds:.2}s speed={digits_per_second:.1} digits/sec"
                );
            }
            _ => {}
        }
    })?;

    if cli.json {
        println!("{}", result.as_json());
    } else {
        println!("{}", result.as_text());
    }

    Ok(())
}

fn list_backends_text() -> String {
    [
        "available backends:",
        "- cpu-single: available",
        "- cpu-multi: unavailable, not implemented",
        cuda_backend_status("cuda-compute"),
        cuda_backend_status("cuda-search-only"),
        feature_backend_status("hip", cfg!(feature = "hip")),
        feature_backend_status("opencl", cfg!(feature = "opencl")),
        feature_backend_status("vulkan", cfg!(feature = "vulkan")),
    ]
    .join("\n")
}

fn cuda_backend_status(name: &str) -> &'static str {
    match (name, cfg!(feature = "cuda")) {
        ("cuda-compute", true) => {
            "- cuda-compute: unavailable, cuda feature enabled but not implemented"
        }
        ("cuda-compute", false) => "- cuda-compute: unavailable, build with --features cuda",
        ("cuda-search-only", true) => {
            "- cuda-search-only: unavailable, cuda feature enabled but not implemented"
        }
        ("cuda-search-only", false) => {
            "- cuda-search-only: unavailable, build with --features cuda"
        }
        _ => unreachable!("unknown cuda backend"),
    }
}

fn feature_backend_status(name: &str, feature_enabled: bool) -> &'static str {
    match (name, feature_enabled) {
        ("hip", true) => "- hip: unavailable, hip feature enabled but not implemented",
        ("hip", false) => "- hip: unavailable, build with --features hip",
        ("opencl", true) => "- opencl: unavailable, opencl feature enabled but not implemented",
        ("opencl", false) => "- opencl: unavailable, build with --features opencl",
        ("vulkan", true) => "- vulkan: unavailable, vulkan feature enabled but not implemented",
        ("vulkan", false) => "- vulkan: unavailable, build with --features vulkan",
        _ => unreachable!("unknown backend"),
    }
}
