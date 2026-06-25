use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

use anyhow::{bail, Result};

use crate::pi::compute_pi_fractional_digits;
use crate::result::{BackendMode, BenchmarkResult, ProgressEvent, RunConfig, RunPhase};

pub fn run_job<F>(
    config: RunConfig,
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
        BackendMode::CpuSingle => run_cpu_single(config, cancel_requested, emit),
    }
}

fn run_cpu_single<F>(
    config: RunConfig,
    cancel_requested: &AtomicBool,
    mut emit: F,
) -> Result<BenchmarkResult>
where
    F: FnMut(ProgressEvent),
{
    let start = Instant::now();

    emit(ProgressEvent::PhaseChanged {
        phase: RunPhase::ComputingPi,
    });
    let digits = compute_pi_fractional_digits(config.max_digits)?;
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

    emit(ProgressEvent::PhaseChanged {
        phase: RunPhase::Searching,
    });
    let search = search_pattern_in_chunks_cancellable(
        &digits,
        &config.target,
        config.chunk,
        cancel_requested,
        |digits_computed| {
            let elapsed_seconds = start.elapsed().as_secs_f64();
            emit(ProgressEvent::Progress {
                digits_computed,
                elapsed_seconds,
                digits_per_second: speed(digits_computed, elapsed_seconds),
            });
        },
    );

    let Some(first_position) = search else {
        emit(ProgressEvent::Cancelled);
        bail!("cancelled");
    };

    let elapsed_seconds = start.elapsed().as_secs_f64();
    let result = BenchmarkResult {
        target: config.target,
        found: first_position.is_some(),
        first_position,
        backend: config.backend.as_str().to_owned(),
        algorithm: "chudnovsky_binary_splitting".to_owned(),
        digits_computed: config.max_digits,
        elapsed_seconds,
        digits_per_second: speed(config.max_digits, elapsed_seconds),
        chunks_processed: config.max_digits.div_ceil(config.chunk),
    };

    emit(ProgressEvent::Completed(result.clone()));
    Ok(result)
}

fn speed(digits: usize, elapsed_seconds: f64) -> f64 {
    if elapsed_seconds > 0.0 {
        digits as f64 / elapsed_seconds
    } else {
        0.0
    }
}

fn search_pattern_in_chunks_cancellable<F>(
    digits: &str,
    pattern: &str,
    chunk_size: usize,
    cancel_requested: &AtomicBool,
    mut progress: F,
) -> Option<Option<usize>>
where
    F: FnMut(usize),
{
    if pattern.is_empty() || chunk_size == 0 {
        return Some(None);
    }

    let overlap_len = pattern.len().saturating_sub(1);
    let mut offset = 0usize;
    let mut carry = String::new();

    while offset < digits.len() {
        if cancel_requested.load(Ordering::Relaxed) {
            return None;
        }

        let end = (offset + chunk_size).min(digits.len());
        let chunk = &digits[offset..end];
        let search_area = format!("{carry}{chunk}");
        let search_area_start_position = offset + 1 - carry.len();

        if let Some(index) = search_area.find(pattern) {
            return Some(Some(search_area_start_position + index));
        }

        if overlap_len > 0 {
            let keep = overlap_len.min(search_area.len());
            carry = search_area[search_area.len() - keep..].to_owned();
        }

        offset = end;
        progress(offset);
    }

    Some(None)
}
