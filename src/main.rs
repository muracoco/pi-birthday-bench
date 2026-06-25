use std::process::ExitCode;
use std::sync::atomic::AtomicBool;

use anyhow::{bail, Result};
use clap::{Parser, ValueEnum};

use pi_birthday_bench::job::run_job;
use pi_birthday_bench::result::{BackendMode as CoreBackendMode, ProgressEvent, RunConfig};

#[derive(Debug, Clone, Copy, ValueEnum)]
enum BackendMode {
    CpuSingle,
}

impl From<BackendMode> for CoreBackendMode {
    fn from(value: BackendMode) -> Self {
        match value {
            BackendMode::CpuSingle => Self::CpuSingle,
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
        "- cuda-compute: unavailable, build with --features cuda",
        "- cuda-search-only: unavailable, build with --features cuda",
        "- hip: unavailable, not implemented",
        "- opencl: unavailable, not implemented",
        "- vulkan: unavailable, not implemented",
    ]
    .join("\n")
}
