//! Shared input bounds. Every money amount and time span that comes from a client goes through
//! here, so huge values can't overflow SQLite `SUM()`, wrap in Rust arithmetic, or push a date
//! past year 9999.

/// Largest money amount accepted from a client: ₹10^12 (stored in paise, so 10^14).
/// Far above any real balance, far below where `i64` sums can overflow.
pub const MAX_AMOUNT_PAISE: i64 = 100_000_000_000_000;

/// An amount as a person types it ("1200", "₹1,200", "1 200.50") in integer paise: no more than two
/// decimals, not negative, within the cap. Money is paise everywhere in the software; this is the
/// one place rupees typed into a form become paise.
pub fn paise_from_text(text: &str) -> Result<i64, String> {
    let wrong = || "That amount didn't look right.".to_string();
    let t: String = text.trim().trim_start_matches('₹').chars().filter(|c| !matches!(c, ',' | ' ')).collect();
    if t.is_empty() || t.starts_with('-') {
        return Err(if t.starts_with('-') { "Amount can't be negative.".into() } else { wrong() });
    }
    let (whole, frac) = match t.split_once('.') {
        Some((w, f)) => (w, f),
        None => (t.as_str(), ""),
    };
    let digits = |s: &str| s.chars().all(|c| c.is_ascii_digit());
    if (whole.is_empty() && frac.is_empty()) || !digits(whole) || !digits(frac) || frac.len() > 2 {
        return Err(wrong());
    }
    let rupees: i64 = if whole.is_empty() { 0 } else { whole.parse().map_err(|_| "That amount is too large.".to_string())? };
    let paise: i64 = match frac.len() {
        0 => 0,
        1 => frac.parse::<i64>().map_err(|_| wrong())? * 10,
        _ => frac.parse::<i64>().map_err(|_| wrong())?,
    };
    let total = rupees.checked_mul(100).and_then(|r| r.checked_add(paise)).ok_or_else(|| "That amount is too large.".to_string())?;
    if total > MAX_AMOUNT_PAISE {
        return Err("That amount is too large.".into());
    }
    Ok(total)
}

/// Paise as the plain text an input field holds: "1200" or "1200.50" (no symbol, no commas).
pub fn plain_rupees(paise: i64) -> String {
    let (r, p) = (paise / 100, (paise % 100).abs());
    if p == 0 { r.to_string() } else { format!("{r}.{p:02}") }
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
    }

    #[test]
    fn typed_amounts_become_paise() {
        for (typed, paise) in [("1200", 120_000), ("₹1,200", 120_000), ("1 200.5", 120_050), ("1200.50", 120_050), ("0.07", 7), (".5", 50), ("0", 0), ("  42  ", 4_200)] {
            assert_eq!(paise_from_text(typed), Ok(paise), "{typed}");
        }
    }

    #[test]
    fn odd_amounts_are_refused_with_a_reason() {
        for bad in ["", "abc", "12.345", "1.2.3", "1e5", "₹", "."] {
            assert_eq!(paise_from_text(bad).unwrap_err(), "That amount didn't look right.", "{bad:?}");
        }
        assert_eq!(paise_from_text("-5").unwrap_err(), "Amount can't be negative.");
        assert_eq!(paise_from_text("99999999999999999999").unwrap_err(), "That amount is too large.");
        assert_eq!(paise_from_text("1000000000001").unwrap_err(), "That amount is too large.");
        assert!(paise_from_text("1000000000000").is_ok(), "the cap itself is fine");
    }

    #[test]
    fn plain_text_for_a_field_round_trips() {
        for paise in [0, 7, 50, 100, 120_050, 99_999_999] {
            assert_eq!(paise_from_text(&plain_rupees(paise)), Ok(paise));
        }
        assert_eq!(plain_rupees(120_000), "1200");
        assert_eq!(plain_rupees(120_050), "1200.50");
    }
}
