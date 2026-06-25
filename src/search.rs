pub fn search_pattern_in_chunks(digits: &str, pattern: &str, chunk_size: usize) -> Option<usize> {
    search_pattern_with_options(
        digits,
        pattern,
        SearchOptions {
            chunk_size,
            benchmark_only: false,
        },
        || false,
        |_| {},
    )
    .first_position
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SearchOptions {
    pub chunk_size: usize,
    pub benchmark_only: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SearchOutcome {
    pub first_position: Option<usize>,
    pub chunks_processed: usize,
    pub cancelled: bool,
}

pub fn search_pattern_with_options<C, P>(
    digits: &str,
    pattern: &str,
    options: SearchOptions,
    mut cancel_requested: C,
    mut progress: P,
) -> SearchOutcome
where
    C: FnMut() -> bool,
    P: FnMut(usize),
{
    if pattern.is_empty() || options.chunk_size == 0 {
        return SearchOutcome {
            first_position: None,
            chunks_processed: 0,
            cancelled: false,
        };
    }

    let overlap_len = pattern.len().saturating_sub(1);
    let mut offset = 0usize;
    let mut carry = String::new();
    let mut chunks_processed = 0usize;
    let mut first_position = None;

    while offset < digits.len() {
        if cancel_requested() {
            return SearchOutcome {
                first_position,
                chunks_processed,
                cancelled: true,
            };
        }

        let end = (offset + options.chunk_size).min(digits.len());
        let chunk = &digits[offset..end];
        let search_area = format!("{carry}{chunk}");
        let search_area_start_position = offset + 1 - carry.len();

        if first_position.is_none() {
            if let Some(index) = search_area.find(pattern) {
                first_position = Some(search_area_start_position + index);
                chunks_processed += 1;
                progress(end);

                if !options.benchmark_only {
                    return SearchOutcome {
                        first_position,
                        chunks_processed,
                        cancelled: false,
                    };
                }
            } else {
                chunks_processed += 1;
                progress(end);
            }
        } else {
            chunks_processed += 1;
            progress(end);
        }

        if overlap_len > 0 {
            let keep = overlap_len.min(search_area.len());
            carry = search_area[search_area.len() - keep..].to_owned();
        }

        offset = end;
    }

    SearchOutcome {
        first_position,
        chunks_processed,
        cancelled: false,
    }
}

#[cfg(test)]
mod tests {
    use crate::pi::compute_pi_fractional_digits;

    use super::{search_pattern_in_chunks, search_pattern_with_options, SearchOptions};

    #[test]
    fn finds_known_mmdd_positions() {
        let digits = compute_pi_fractional_digits(200).unwrap();

        assert_eq!(search_pattern_in_chunks(&digits, "0628", 50), Some(71));
        assert_eq!(search_pattern_in_chunks(&digits, "0812", 50), Some(146));
        assert_eq!(search_pattern_in_chunks(&digits, "1027", 50), Some(163));
        assert_eq!(search_pattern_in_chunks(&digits, "1117", 50), Some(153));
        assert_eq!(search_pattern_in_chunks(&digits, "1105", 50), Some(174));
    }

    #[test]
    fn finds_match_across_chunk_boundary() {
        assert_eq!(search_pattern_in_chunks("1234567890", "4567", 5), Some(4));
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
        assert!(!outcome.cancelled);
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
        assert!(!outcome.cancelled);
        assert_eq!(progress, vec![2, 4, 6, 8, 10]);
    }

    #[test]
    fn reports_cancellation_before_next_chunk() {
        let mut checks = 0usize;

        let outcome = search_pattern_with_options(
            "1234567890",
            "90",
            SearchOptions {
                chunk_size: 2,
                benchmark_only: true,
            },
            || {
                checks += 1;
                checks > 2
            },
            |_| {},
        );

        assert_eq!(outcome.first_position, None);
        assert_eq!(outcome.chunks_processed, 2);
        assert!(outcome.cancelled);
    }
}
