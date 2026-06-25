use std::process::ExitCode;
use std::sync::atomic::AtomicBool;

use anyhow::Result;
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
    #[arg(long)]
    target: String,

    #[arg(long)]
    max_digits: usize,

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
    let config = RunConfig {
        target: cli.target,
        max_digits: cli.max_digits,
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
