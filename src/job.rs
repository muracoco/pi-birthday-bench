use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

use anyhow::{bail, Result};

use crate::backend::{unavailable_backend_error, CpuMultiBackend, CpuSingleBackend, PiBackend};
use crate::pi::{compute_pi_fractional_digits, KNOWN_PI_FRACTIONAL_PREFIX};
use crate::result::{
    BackendMode, BenchmarkResult, ProgressEvent, RunConfig, RunPhase, VerificationStatus,
};
use crate::search::{search_pattern_with_options, SearchOptions};
use crate::system_info::collect_system_info;

const CPU_MULTI_VERIFY_DIGITS: usize = 1_000;

pub fn run_job<F>(
    config: RunConfig,
    cancel_requested: &AtomicBool,
    mut emit: F,
) -> Result<BenchmarkResult>
where
    F: FnMut(ProgressEvent),
{
    let config = resolve_runtime_config(config);

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
            let threads = config.threads.expect("cpu-multi threads are initialized");
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

fn resolve_runtime_config(mut config: RunConfig) -> RunConfig {
    if config.backend == BackendMode::CpuMulti && config.threads.is_none() {
        config.threads = Some(default_thread_count());
    }
    config
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
        range_start: 1,
        range_end: config.max_digits,
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
    let mut previous_digits_computed = 0usize;
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
            let range_start = previous_digits_computed.saturating_add(1);
            previous_digits_computed = digits_computed;
            emit(ProgressEvent::Progress {
                range_start,
                range_end: digits_computed,
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
    let system_info = collect_system_info();
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
        cpu_model: system_info.cpu_model,
        logical_cpu_count: system_info.logical_cpu_count,
        physical_cpu_count: system_info.physical_cpu_count,
        gpu_role: backend.gpu_role().as_str().to_owned(),
        memory_total_mb: system_info.memory_total_mb,
        memory_peak_mb: system_info.memory_peak_mb,
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
    use std::sync::atomic::{AtomicBool, Ordering};

    use super::{resolve_runtime_config, run_job, verify_generated_digits};
    use crate::result::{BackendMode, ProgressEvent, RunConfig, RunPhase, VerificationStatus};
    use crate::search::{search_pattern_with_options, SearchOptions};

    fn test_config(backend: BackendMode) -> RunConfig {
        RunConfig {
            target: "20000101".to_owned(),
            max_digits: 100,
            chunk: 25,
            backend,
            benchmark_only: false,
            threads: (backend == BackendMode::CpuMulti).then_some(1),
            verify: false,
        }
    }

    fn event_name(event: &ProgressEvent) -> &'static str {
        match event {
            ProgressEvent::Started { .. } => "started",
            ProgressEvent::PhaseChanged {
                phase: RunPhase::Validating,
            } => "validating",
            ProgressEvent::PhaseChanged {
                phase: RunPhase::ComputingPi,
            } => "computing_pi",
            ProgressEvent::PhaseChanged {
                phase: RunPhase::Searching,
            } => "searching",
            ProgressEvent::PhaseChanged { .. } => "other_phase",
            ProgressEvent::Progress { .. } => "progress",
            ProgressEvent::Completed(_) => "completed",
            ProgressEvent::Cancelled => "cancelled",
            ProgressEvent::Failed(_) => "failed",
        }
    }

    #[test]
    fn successful_job_emits_ordered_phases_and_one_terminal_event() {
        let cancel = AtomicBool::new(false);
        let mut events = Vec::new();

        let result = run_job(test_config(BackendMode::CpuSingle), &cancel, |event| {
            events.push(event)
        })
        .expect("job succeeds");

        let names = events.iter().map(event_name).collect::<Vec<_>>();
        assert_eq!(names[0..3], ["started", "validating", "computing_pi"]);
        assert!(names
            .windows(2)
            .any(|pair| pair == ["progress", "searching"]));
        assert_eq!(names.last(), Some(&"completed"));
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(
                    event,
                    ProgressEvent::Completed(_) | ProgressEvent::Cancelled
                ))
                .count(),
            1
        );
        assert_eq!(result.backend, "cpu-single");
        assert_eq!(result.digits_computed, 100);
    }

    #[test]
    fn cancelled_before_validation_emits_cancelled_once() {
        let cancel = AtomicBool::new(true);
        let mut events = Vec::new();

        let error = run_job(test_config(BackendMode::CpuSingle), &cancel, |event| {
            events.push(event)
        })
        .expect_err("job is cancelled");

        assert_eq!(error.to_string(), "cancelled");
        assert_eq!(
            events.iter().map(event_name).collect::<Vec<_>>(),
            ["started", "validating", "cancelled"]
        );
    }

    #[test]
    fn invalid_config_stops_after_validation_without_terminal_event() {
        let cancel = AtomicBool::new(false);
        let mut config = test_config(BackendMode::CpuSingle);
        config.chunk = 0;
        let mut events = Vec::new();

        let error =
            run_job(config, &cancel, |event| events.push(event)).expect_err("invalid config fails");

        assert_eq!(error.to_string(), "chunk must be greater than 0");
        assert_eq!(
            events.iter().map(event_name).collect::<Vec<_>>(),
            ["started", "validating"]
        );
    }

    #[test]
    fn unsupported_backend_does_not_start_pi_computation() {
        let cancel = AtomicBool::new(false);
        let mut events = Vec::new();

        let error = run_job(test_config(BackendMode::CudaCompute), &cancel, |event| {
            events.push(event)
        })
        .expect_err("GPU backend is unavailable");

        assert!(error
            .to_string()
            .contains("backend 'cuda-compute' is not available"));
        assert_eq!(
            events.iter().map(event_name).collect::<Vec<_>>(),
            ["started", "validating"]
        );
    }

    #[test]
    fn cpu_multi_without_threads_reports_resolved_thread_count() {
        let cancel = AtomicBool::new(false);
        let mut config = test_config(BackendMode::CpuMulti);
        config.threads = None;

        let result = run_job(config, &cancel, |_| {}).expect("job succeeds");

        assert!(result.threads.is_some_and(|threads| threads > 0));
    }

    #[test]
    fn runtime_config_only_defaults_cpu_multi_threads() {
        let single = resolve_runtime_config(test_config(BackendMode::CpuSingle));
        assert_eq!(single.threads, None);

        let mut explicit_multi = test_config(BackendMode::CpuMulti);
        explicit_multi.threads = Some(2);
        assert_eq!(resolve_runtime_config(explicit_multi).threads, Some(2));

        let mut default_multi = test_config(BackendMode::CpuMulti);
        default_multi.threads = None;
        assert!(resolve_runtime_config(default_multi)
            .threads
            .is_some_and(|threads| threads > 0));
    }

    #[test]
    fn cancellation_requested_during_search_is_terminal() {
        let cancel = AtomicBool::new(false);
        let mut events = Vec::new();

        let error = run_job(test_config(BackendMode::CpuSingle), &cancel, |event| {
            if matches!(
                event,
                ProgressEvent::PhaseChanged {
                    phase: RunPhase::Searching
                }
            ) {
                cancel.store(true, Ordering::Relaxed);
            }
            events.push(event);
        })
        .expect_err("search is cancelled");

        assert_eq!(error.to_string(), "cancelled");
        assert_eq!(events.iter().map(event_name).next_back(), Some("cancelled"));
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(
                    event,
                    ProgressEvent::Completed(_) | ProgressEvent::Cancelled
                ))
                .count(),
            1
        );
    }

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
            target: "20000101".to_owned(),
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
            target: "20000101".to_owned(),
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
