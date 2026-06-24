use anyhow::{bail, Result};

pub fn validate_yyyymmdd(value: &str) -> Result<()> {
    if value.len() != 8 || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        bail!("target must be exactly 8 digits in YYYYMMDD format");
    }

    let year: u32 = value[0..4].parse()?;
    let month: u32 = value[4..6].parse()?;
    let day: u32 = value[6..8].parse()?;

    if month == 0 || month > 12 {
        bail!("month must be in 01..12");
    }

    let max_day = days_in_month(year, month);
    if day == 0 || day > max_day {
        bail!("day is not valid for the given year and month");
    }

    Ok(())
}

fn days_in_month(year: u32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => 0,
    }
}

fn is_leap_year(year: u32) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

#[cfg(test)]
mod tests {
    use super::validate_yyyymmdd;

    #[test]
    fn accepts_valid_dates() {
        for value in ["20000229", "20240628", "19931203"] {
            assert!(validate_yyyymmdd(value).is_ok(), "{value}");
        }
    }

    #[test]
    fn rejects_invalid_dates() {
        for value in [
            "19930229", "19000229", "20241301", "20240631", "0628", "abc",
        ] {
            assert!(validate_yyyymmdd(value).is_err(), "{value}");
        }
    }
}
