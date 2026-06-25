use std::process::ExitCode;
use std::sync::atomic::AtomicBool;

use anyhow::{bail, Result};
use clap::{Parser, ValueEnum};

use pi_birthday_bench::backend::backend_list_text;
use pi_birthday_bench::job::run_job;
use pi_birthday_bench::result::{BackendMode as CoreBackendMode, ProgressEvent, RunConfig};

#[derive(Debug, Clone, Copy, ValueEnum)]
enum BackendMode {
    CpuSingle,
    #[value(name = "cpu-multi")]
    CpuMulti,
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
            BackendMode::CpuMulti => Self::CpuMulti,
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
    verify: bool,

    #[arg(long)]
    threads: Option<usize>,

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
        println!("{}", backend_list_text());
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
        threads: cli.threads,
        verify: cli.verify,
    };
    let cancel_requested = AtomicBool::new(false);
    let mut progress_config: Option<RunConfig> = None;

    let result = run_job(config, &cancel_requested, |event| {
        if cli.no_progress || cli.json {
            return;
        }

        match event {
            ProgressEvent::Started { config } => {
                progress_config = Some(config);
            }
            ProgressEvent::PhaseChanged { phase } => {
                eprintln!("phase={}", phase.as_str());
            }
            ProgressEvent::Progress {
                range_start,
                range_end,
                digits_computed,
                elapsed_seconds,
                digits_per_second,
            } => {
                let backend = progress_config
                    .as_ref()
                    .map(|config| config.backend.as_str())
                    .unwrap_or("unknown");
                let target = progress_config
                    .as_ref()
                    .map(|config| config.target.as_str())
                    .unwrap_or("unknown");
                let chunk = progress_config
                    .as_ref()
                    .map(|config| config.chunk.to_string())
                    .unwrap_or_else(|| "unknown".to_owned());
                let threads = progress_config
                    .as_ref()
                    .and_then(|config| config.threads)
                    .map(|threads| threads.to_string())
                    .unwrap_or_else(|| "null".to_owned());

                eprintln!(
                    "backend={backend} target={target} range={range_start}..{range_end} digits_computed={digits_computed} elapsed_seconds={elapsed_seconds:.2} digits_per_second={digits_per_second:.1} chunk={chunk} threads={threads}"
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
