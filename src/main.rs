use std::process::ExitCode;
use std::time::Instant;

use anyhow::{bail, Context, Result};
use clap::{Parser, ValueEnum};

use pi_birthday_bench::date::validate_yyyymmdd;
use pi_birthday_bench::pi::compute_pi_fractional_digits;
use pi_birthday_bench::search::search_pattern_in_chunks;

#[derive(Debug, Clone, Copy, ValueEnum)]
enum BackendMode {
    CpuSingle,
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

    validate_yyyymmdd(&cli.target).with_context(|| format!("invalid --target '{}'", cli.target))?;

    if cli.max_digits == 0 {
        bail!("--max-digits must be greater than 0");
    }
    if cli.chunk == 0 {
        bail!("--chunk must be greater than 0");
    }

    match cli.backend {
        BackendMode::CpuSingle => run_cpu_single(&cli),
    }
}

fn run_cpu_single(cli: &Cli) -> Result<()> {
    let start = Instant::now();
    let digits = compute_pi_fractional_digits(cli.max_digits)?;
    let first_position = search_pattern_in_chunks(&digits, &cli.target, cli.chunk);
    let elapsed = start.elapsed();
    let elapsed_seconds = elapsed.as_secs_f64();
    let digits_per_second = if elapsed_seconds > 0.0 {
        cli.max_digits as f64 / elapsed_seconds
    } else {
        0.0
    };
    let chunks_processed = cli.max_digits.div_ceil(cli.chunk);

    if !cli.no_progress {
        eprintln!(
            "backend=cpu-single target={} digits_computed={} elapsed={:.2}s speed={:.1} digits/sec",
            cli.target, cli.max_digits, elapsed_seconds, digits_per_second
        );
    }

    println!("target: {}", cli.target);
    println!("found: {}", first_position.is_some());
    match first_position {
        Some(position) => println!("first_position: {position}"),
        None => println!("first_position: null"),
    }
    println!("backend: cpu-single");
    println!("algorithm: chudnovsky_binary_splitting");
    println!("digits_computed: {}", cli.max_digits);
    println!("elapsed_seconds: {:.6}", elapsed_seconds);
    println!("digits_per_second: {:.1}", digits_per_second);
    println!("chunks_processed: {chunks_processed}");

    Ok(())
}
