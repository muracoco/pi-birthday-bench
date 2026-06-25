use anyhow::{bail, Result};
use rayon::ThreadPoolBuilder;
use rug::{ops::Pow, Complete, Integer};

const DIGITS_PER_TERM: usize = 14;
const GUARD_DIGITS: usize = 20;
const A: i64 = 13_591_409;
const B: i64 = 545_140_134;
const C3_OVER_24: i64 = 10_939_058_860_032_000;

pub const KNOWN_PI_FRACTIONAL_PREFIX: &str = "\
14159265358979323846264338327950288419716939937510\
58209749445923078164062862089986280348253421170679";

pub fn compute_pi_fractional_digits(requested_digits: usize) -> Result<String> {
    compute_pi_fractional_digits_with(requested_digits, |terms| binary_split(0, terms as u32))
}

pub fn compute_pi_fractional_digits_parallel(
    requested_digits: usize,
    threads: usize,
) -> Result<String> {
    if threads == 0 {
        bail!("threads must be greater than 0");
    }
    if threads == 1 {
        return compute_pi_fractional_digits(requested_digits);
    }

    let pool = ThreadPoolBuilder::new().num_threads(threads).build()?;
    pool.install(|| {
        compute_pi_fractional_digits_with(requested_digits, |terms| {
            binary_split_parallel(0, terms as u32)
        })
    })
}

fn compute_pi_fractional_digits_with<F>(requested_digits: usize, split: F) -> Result<String>
where
    F: FnOnce(usize) -> (Integer, Integer, Integer),
{
    if requested_digits == 0 {
        return Ok(String::new());
    }

    let internal_digits = requested_digits + GUARD_DIGITS;
    let terms = internal_digits / DIGITS_PER_TERM + 2;
    let (_, q, t) = split(terms);

    if t == 0 {
        bail!("failed to compute pi: zero divisor");
    }

    let scale = Integer::from(10).pow(internal_digits as u32);
    let sqrt_arg = Integer::from(10005) * Integer::from(&scale).pow(2);
    let sqrt = sqrt_arg.sqrt();
    let mut scaled_pi = q;
    scaled_pi *= 426_880;
    scaled_pi *= sqrt;
    scaled_pi /= t;

    let mut digits = scaled_pi.to_string();
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

    let p: Integer = (&p1 * &p2).complete();
    let q: Integer = (&q1 * &q2).complete();
    let mut t: Integer = &q2 * t1;
    t += &p1 * t2;

    (p, q, t)
}

fn binary_split_parallel(a: u32, b: u32) -> (Integer, Integer, Integer) {
    if b - a <= 16 {
        return binary_split(a, b);
    }

    let mid = (a + b) / 2;
    let ((p1, q1, t1), (p2, q2, t2)) = rayon::join(
        || binary_split_parallel(a, mid),
        || binary_split_parallel(mid, b),
    );

    let p: Integer = (&p1 * &p2).complete();
    let q: Integer = (&q1 * &q2).complete();
    let mut t: Integer = &q2 * t1;
    t += &p1 * t2;

    (p, q, t)
}

#[cfg(test)]
mod tests {
    use super::{
        compute_pi_fractional_digits, compute_pi_fractional_digits_parallel,
        KNOWN_PI_FRACTIONAL_PREFIX,
    };

    #[test]
    fn first_100_fractional_digits_match_known_prefix() {
        assert_eq!(
            compute_pi_fractional_digits(100).unwrap(),
            KNOWN_PI_FRACTIONAL_PREFIX
        );
    }

    #[test]
    fn parallel_digits_match_single_thread_digits() {
        assert_eq!(
            compute_pi_fractional_digits_parallel(1_000, 2).unwrap(),
            compute_pi_fractional_digits(1_000).unwrap()
        );
    }

    #[test]
    fn parallel_with_one_thread_matches_single_thread_digits() {
        assert_eq!(
            compute_pi_fractional_digits_parallel(1_000, 1).unwrap(),
            compute_pi_fractional_digits(1_000).unwrap()
        );
    }
}
