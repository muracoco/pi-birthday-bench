use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

use anyhow::{bail, Result};

use crate::backend::{unavailable_backend_error, CpuMultiBackend, CpuSingleBackend, PiBackend};
use crate::pi::{compute_pi_fractional_digits, KNOWN_PI_FRACTIONAL_PREFIX};
use crate::result::{
    BackendMode, BenchmarkResult, ProgressEvent, RunConfig, RunPhase, VerificationStatus,
};
use crate::search::{search_pattern_with_options, SearchOptions};

const CPU_MULTI_VERIFY_DIGITS: usize = 1_000;

pub fn run_job<F>(
    mut config: RunConfig,
    cancel_requested: &AtomicBool,
    mut emit: F,
) -> Result<BenchmarkResult>
where
    F: FnMut(ProgressEvent),
{
    emit(ProgressEvent::Started {
        config: config.clone(),
    });
    emit(ProgressEvent::PhaseChanged {
        phase: RunPhase::Validating,
    });
    config.validate()?;

    if cancel_requested.load(Ordering::Relaxed) {
        emit(ProgressEvent::Cancelled);
        bail!("cancelled");
    }

    match config.backend {
        BackendMode::CpuSingle => run_backend(config, &CpuSingleBackend, cancel_requested, emit),
        BackendMode::CpuMulti => {
            let threads = config.threads.unwrap_or_else(default_thread_count);
            config.threads = Some(threads);
            let backend = CpuMultiBackend { threads };
            run_backend(config, &backend, cancel_requested, emit)
        }
        BackendMode::CudaCompute
        | BackendMode::CudaSearchOnly
        | BackendMode::Hip
        | BackendMode::OpenCl
        | BackendMode::Vulkan => {
            unavailable_backend_error(config.backend)?;
            unreachable!("unavailable backend unexpectedly passed availability check")
        }
    }
}

fn run_backend<B, F>(
    config: RunConfig,
    backend: &B,
    cancel_requested: &AtomicBool,
    mut emit: F,
) -> Result<BenchmarkResult>
where
    B: PiBackend,
    F: FnMut(ProgressEvent),
{
    debug_assert!(backend.is_available());
    let start = Instant::now();

    emit(ProgressEvent::PhaseChanged {
        phase: RunPhase::ComputingPi,
    });
    let digits = backend.compute_digits(config.max_digits)?;
    let elapsed_seconds = start.elapsed().as_secs_f64();
    emit(ProgressEvent::Progress {
        digits_computed: config.max_digits,
        elapsed_seconds,
        digits_per_second: speed(config.max_digits, elapsed_seconds),
    });

    if cancel_requested.load(Ordering::Relaxed) {
        emit(ProgressEvent::Cancelled);
        bail!("cancelled");
    }

    let verification_status = verify_generated_digits(&config, &digits)?;

    if cancel_requested.load(Ordering::Relaxed) {
        emit(ProgressEvent::Cancelled);
        bail!("cancelled");
    }

    emit(ProgressEvent::PhaseChanged {
        phase: RunPhase::Searching,
    });
    let search = search_pattern_with_options(
        &digits,
        &config.target,
        SearchOptions {
            chunk_size: config.chunk,
            benchmark_only: config.benchmark_only,
        },
        || cancel_requested.load(Ordering::Relaxed),
        |digits_computed| {
            let elapsed_seconds = start.elapsed().as_secs_f64();
            emit(ProgressEvent::Progress {
                digits_computed,
                elapsed_seconds,
                digits_per_second: speed(digits_computed, elapsed_seconds),
            });
        },
    );

    if search.cancelled {
        emit(ProgressEvent::Cancelled);
        bail!("cancelled");
    }

    let elapsed_seconds = start.elapsed().as_secs_f64();
    let result = BenchmarkResult {
        target: config.target,
        found: search.first_position.is_some(),
        first_position: search.first_position,
        backend: backend.name().to_owned(),
        algorithm: "chudnovsky_binary_splitting".to_owned(),
        digits_computed: config.max_digits,
        elapsed_seconds,
        digits_per_second: speed(config.max_digits, elapsed_seconds),
        chunks_processed: search.chunks_processed,
        threads: config
            .threads
            .filter(|_| config.backend == BackendMode::CpuMulti),
        gpu_role: backend.gpu_role().as_str().to_owned(),
        verification_status,
    };

    emit(ProgressEvent::Completed(result.clone()));
    Ok(result)
}

fn default_thread_count() -> usize {
    std::thread::available_parallelism()
        .map(usize::from)
        .unwrap_or(1)
}

fn speed(digits: usize, elapsed_seconds: f64) -> f64 {
    if elapsed_seconds > 0.0 {
        digits as f64 / elapsed_seconds
    } else {
        0.0
    }
}

fn verify_generated_digits(config: &RunConfig, digits: &str) -> Result<VerificationStatus> {
    if !config.verify {
        return Ok(VerificationStatus::Skipped);
    }

    verify_known_prefix(digits)?;

    if config.backend == BackendMode::CpuMulti {
        let compare_digits = digits.len().min(CPU_MULTI_VERIFY_DIGITS);
        let single_digits = compute_pi_fractional_digits(compare_digits)?;
        if digits[..compare_digits] != single_digits {
            bail!("verification failed: cpu-multi output differs from cpu-single");
        }
    }

    Ok(VerificationStatus::Passed)
}

fn verify_known_prefix(digits: &str) -> Result<()> {
    let prefix_len = digits.len().min(KNOWN_PI_FRACTIONAL_PREFIX.len());
    if digits[..prefix_len] != KNOWN_PI_FRACTIONAL_PREFIX[..prefix_len] {
        bail!("verification failed: generated pi prefix does not match known prefix");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::verify_generated_digits;
    use crate::result::{BackendMode, RunConfig, VerificationStatus};
    use crate::search::{search_pattern_with_options, SearchOptions};

    #[test]
    fn normal_search_stops_when_pattern_is_found() {
        let mut progress = Vec::new();

        let outcome = search_pattern_with_options(
            "1234567890",
            "34",
            SearchOptions {
                chunk_size: 2,
                benchmark_only: false,
            },
            || false,
            |digits_computed| progress.push(digits_computed),
        );

        assert_eq!(outcome.first_position, Some(3));
        assert_eq!(outcome.chunks_processed, 2);
        assert_eq!(progress, vec![2, 4]);
    }

    #[test]
    fn benchmark_only_continues_after_pattern_is_found() {
        let mut progress = Vec::new();

        let outcome = search_pattern_with_options(
            "1234567890",
            "34",
            SearchOptions {
                chunk_size: 2,
                benchmark_only: true,
            },
            || false,
            |digits_computed| progress.push(digits_computed),
        );

        assert_eq!(outcome.first_position, Some(3));
        assert_eq!(outcome.chunks_processed, 5);
        assert_eq!(progress, vec![2, 4, 6, 8, 10]);
    }

    #[test]
    fn verification_passes_for_known_prefix() {
        let config = RunConfig {
            target: "20240628".to_owned(),
            max_digits: 10,
            chunk: 10,
            backend: BackendMode::CpuSingle,
            benchmark_only: false,
            threads: None,
            verify: true,
        };

        assert_eq!(
            verify_generated_digits(&config, "1415926535").unwrap(),
            VerificationStatus::Passed
        );
    }

    #[test]
    fn verification_fails_for_bad_prefix() {
        let config = RunConfig {
            target: "20240628".to_owned(),
            max_digits: 10,
            chunk: 10,
            backend: BackendMode::CpuSingle,
            benchmark_only: false,
            threads: None,
            verify: true,
        };

        assert!(verify_generated_digits(&config, "0000000000").is_err());
    }
}
