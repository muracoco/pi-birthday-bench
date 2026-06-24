pub fn search_pattern_in_chunks(digits: &str, pattern: &str, chunk_size: usize) -> Option<usize> {
    if pattern.is_empty() || chunk_size == 0 {
        return None;
    }

    let overlap_len = pattern.len().saturating_sub(1);
    let mut offset = 0usize;
    let mut carry = String::new();

    while offset < digits.len() {
        let end = (offset + chunk_size).min(digits.len());
        let chunk = &digits[offset..end];
        let search_area = format!("{carry}{chunk}");
        let search_area_start_position = offset + 1 - carry.len();

        if let Some(index) = search_area.find(pattern) {
            return Some(search_area_start_position + index);
        }

        if overlap_len > 0 {
            let keep = overlap_len.min(search_area.len());
            carry = search_area[search_area.len() - keep..].to_owned();
        }

        offset = end;
    }

    None
}

#[cfg(test)]
mod tests {
    use crate::pi::compute_pi_fractional_digits;

    use super::search_pattern_in_chunks;

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
}
