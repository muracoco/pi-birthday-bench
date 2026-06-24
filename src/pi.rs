use anyhow::{bail, Result};
use rug::{ops::Pow, Integer};

const DIGITS_PER_TERM: usize = 14;
const GUARD_DIGITS: usize = 20;
const A: i64 = 13_591_409;
const B: i64 = 545_140_134;
const C3_OVER_24: i64 = 10_939_058_860_032_000;

pub fn compute_pi_fractional_digits(requested_digits: usize) -> Result<String> {
    if requested_digits == 0 {
        return Ok(String::new());
    }

    let internal_digits = requested_digits + GUARD_DIGITS;
    let terms = internal_digits / DIGITS_PER_TERM + 2;
    let (_, q, t) = binary_split(0, terms as u32);

    if t == 0 {
        bail!("failed to compute pi: zero divisor");
    }

    let scale = Integer::from(10).pow(internal_digits as u32);
    let sqrt_arg = Integer::from(10005) * Integer::from(&scale).pow(2);
    let sqrt = sqrt_arg.sqrt();
    let scaled_pi = q * 426_880 * sqrt / t;

    let mut digits = scaled_pi.to_string_radix(10);
    if digits.len() < internal_digits + 1 {
        let padding = "0".repeat(internal_digits + 1 - digits.len());
        digits = format!("{padding}{digits}");
    }

    let fractional = digits
        .get(1..1 + requested_digits)
        .ok_or_else(|| anyhow::anyhow!("computed pi output was shorter than requested"))?;
    Ok(fractional.to_owned())
}

fn binary_split(a: u32, b: u32) -> (Integer, Integer, Integer) {
    if b - a == 1 {
        if a == 0 {
            return (Integer::from(1), Integer::from(1), Integer::from(A));
        }

        let a_int = Integer::from(a);
        let p = Integer::from(6 * a as i64 - 5)
            * Integer::from(2 * a as i64 - 1)
            * Integer::from(6 * a as i64 - 1);
        let q = Integer::from(&a_int).pow(3) * C3_OVER_24;
        let mut t = Integer::from(&p) * (A + B * a as i64);
        if a % 2 == 1 {
            t = -t;
        }
        return (p, q, t);
    }

    let mid = (a + b) / 2;
    let (p1, q1, t1) = binary_split(a, mid);
    let (p2, q2, t2) = binary_split(mid, b);

    let p = &p1 * &p2;
    let q = &q1 * &q2;
    let t = &q2 * t1 + &p1 * t2;

    (p, q, t)
}

#[cfg(test)]
mod tests {
    use super::compute_pi_fractional_digits;

    #[test]
    fn first_100_fractional_digits_match_known_prefix() {
        let expected = "\
14159265358979323846264338327950288419716939937510\
58209749445923078164062862089986280348253421170679";
        assert_eq!(compute_pi_fractional_digits(100).unwrap(), expected);
    }
}
