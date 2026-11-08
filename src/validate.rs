//! Shared input bounds. Every money amount and time span that comes from a client goes through
//! here, so huge values can't overflow SQLite `SUM()`, wrap in Rust arithmetic, or push a date
//! past year 9999.

/// Largest money amount accepted from a client: ₹10^12 (stored in paise, so 10^14).
/// Far above any real balance, far below where `i64` sums can overflow.
pub const MAX_AMOUNT_PAISE: i64 = 100_000_000_000_000;

/// The same cap for the Dates side, which stores whole rupees: ₹10^12.
pub const MAX_AMOUNT_RUPEES: i64 = 1_000_000_000_000;

/// Dates side (whole rupees): `Ok(v)` when `0 <= v <= MAX_AMOUNT_RUPEES`.
pub fn rupees(v: i64) -> Result<i64, String> {
    if v < 0 {
        Err("Amount can't be negative.".into())
    } else if v > MAX_AMOUNT_RUPEES {
        Err("That amount is too large.".into())
    } else {
        Ok(v)
    }
}

/// Longest tenure / horizon in months that any planner input may span (100 years).
pub const MAX_MONTHS: i64 = 1200;

/// `Ok(v)` when `0 < v <= MAX_AMOUNT_PAISE`.
pub fn positive_amount(v: i64) -> Result<i64, String> {
    if v <= 0 {
        Err("Amount must be more than zero.".into())
    } else if v > MAX_AMOUNT_PAISE {
        Err("That amount is too large.".into())
    } else {
        Ok(v)
    }
}

/// `Ok(v)` when `0 <= v <= MAX_AMOUNT_PAISE`.
pub fn non_negative_amount(v: i64) -> Result<i64, String> {
    if v < 0 {
        Err("Amount can't be negative.".into())
    } else if v > MAX_AMOUNT_PAISE {
        Err("That amount is too large.".into())
    } else {
        Ok(v)
    }
}

/// `Ok(v)` when `|v| <= MAX_AMOUNT_PAISE` (signed values such as balances).
pub fn bounded_amount(v: i64) -> Result<i64, String> {
    if v.checked_abs().is_none_or(|a| a > MAX_AMOUNT_PAISE) {
        Err("That amount is too large.".into())
    } else {
        Ok(v)
    }
}

/// `Ok(m)` when `0 <= m <= MAX_MONTHS`.
pub fn months(m: i64) -> Result<i64, String> {
    if (0..=MAX_MONTHS).contains(&m) {
        Ok(m)
    } else {
        Err(format!("Months must be between 0 and {MAX_MONTHS}."))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounds() {
        assert!(positive_amount(0).is_err());
        assert!(positive_amount(-5).is_err());
        assert!(positive_amount(1).is_ok());
        assert!(positive_amount(MAX_AMOUNT_PAISE).is_ok());
        assert!(positive_amount(MAX_AMOUNT_PAISE + 1).is_err());
        assert!(non_negative_amount(0).is_ok());
        assert!(bounded_amount(i64::MIN).is_err());
        assert!(bounded_amount(-MAX_AMOUNT_PAISE).is_ok());
        assert!(months(1201).is_err());
        assert!(months(-1).is_err());
        assert!(months(360).is_ok());
        assert!(rupees(MAX_AMOUNT_RUPEES + 1).is_err());
        assert!(rupees(0).is_ok());
    }
}
