//! India income-tax tracker. Pure: no DB, no clock. Money is INTEGER paise.
//!
//! The financial year (FY) runs April to March and is written `2026-27`.
//!
//! # The slabs are data, and must be checked every Budget
//!
//! [`FY_RULES`] holds one table per FY. The 2025-26 and 2026-27 rows are:
//!
//! * **New regime:** standard deduction 75,000; 0-4L 0%, 4-8L 5%, 8-12L 10%, 12-16L 15%,
//!   16-20L 20%, 20-24L 25%, above 24L 30%; section 87A rebate makes tax 0 up to 12L taxable
//!   income, with marginal relief just above it.
//! * **Old regime:** standard deduction 50,000 plus the deductions claimed (capped at their
//!   limits) plus HRA; 0-2.5L 0%, 2.5-5L 5%, 5-10L 20%, above 10L 30%; 87A rebate up to 5L.
//! * 4% health and education cess on both. No surcharge, no employer NPS (80CCD(2)), no
//!   capital gains: this is a salary-income estimate to compare the two regimes, not a return.
//!
//! A new Budget changes these. An FY without its own row falls back to the latest earlier row and
//! the response says so.

use serde::Serialize;
use time::{Date, Month};

use super::cycle::{add_months, clamped};

pub const SECTIONS: [&str; 6] = ["80C", "80CCD1B", "80D", "HRA", "24B", "other"];

pub fn valid_section(s: &str) -> bool {
    SECTIONS.contains(&s)
}

/// Yearly cap of a section, in paise. `None` = no cap we track.
pub fn limit_for(section: &str) -> Option<i64> {
    match section {
        "80C" => Some(150_000_00),
        "80CCD1B" => Some(50_000_00),
        // 50,000 for senior-citizen parents; we don't ask, so the general limit is used.
        "80D" => Some(25_000_00),
        "24B" => Some(200_000_00),
        _ => None,
    }
}

/// Shown beside 80D.
pub fn section_note(section: &str) -> Option<&'static str> {
    (section == "80D").then_some("The limit is 50,000 if your parents are senior citizens.")
}

// ---------- financial year ----------

/// The year an FY starts in: 1 April 2026 and 31 March 2027 are both FY 2026-27.
pub fn fy_start_year(d: Date) -> i32 {
    if (d.month() as u8) >= 4 {
        d.year()
    } else {
        d.year() - 1
    }
}

pub fn fy_label(start: i32) -> String {
    format!("{start:04}-{:02}", (start + 1).rem_euclid(100))
}

/// `'2026-27'` -> 2026. The second half must be the year after the first.
pub fn parse_fy(s: &str) -> Option<i32> {
    let (a, b) = s.split_once('-')?;
    if a.len() != 4 || b.len() != 2 {
        return None;
    }
    let (start, end): (i32, i32) = (a.parse().ok()?, b.parse().ok()?);
    (start >= 2000 && start < 2200 && (start + 1).rem_euclid(100) == end).then_some(start)
}

/// First and last day of the FY.
pub fn fy_bounds(start: i32) -> (Date, Date) {
    (
        Date::from_calendar_date(start, Month::April, 1).expect("valid"),
        Date::from_calendar_date(start + 1, Month::March, 31).expect("valid"),
    )
}

// ---------- slabs (data) ----------

/// `(upper bound in rupees or None for "and above", rate in percent)`, lowest slab first.
type Slabs = &'static [(Option<i64>, i64)];

pub struct RegimeRules {
    /// Rupees.
    pub standard_deduction: i64,
    pub slabs: Slabs,
    /// Taxable income (rupees) up to which section 87A makes the tax zero.
    pub rebate_up_to: i64,
    /// Above `rebate_up_to`, tax is capped at the excess over it.
    pub marginal_relief: bool,
}

pub struct FyRules {
    pub fy_start: i32,
    pub new: RegimeRules,
    pub old: RegimeRules,
}

const NEW_SLABS_2025: Slabs = &[
    (Some(4_00_000), 0),
    (Some(8_00_000), 5),
    (Some(12_00_000), 10),
    (Some(16_00_000), 15),
    (Some(20_00_000), 20),
    (Some(24_00_000), 25),
    (None, 30),
];
const OLD_SLABS: Slabs = &[(Some(2_50_000), 0), (Some(5_00_000), 5), (Some(10_00_000), 20), (None, 30)];

const fn rules(fy_start: i32) -> FyRules {
    FyRules {
        fy_start,
        new: RegimeRules { standard_deduction: 75_000, slabs: NEW_SLABS_2025, rebate_up_to: 12_00_000, marginal_relief: true },
        old: RegimeRules { standard_deduction: 50_000, slabs: OLD_SLABS, rebate_up_to: 5_00_000, marginal_relief: false },
    }
}

/// One row per FY, oldest first. CHECK EVERY BUDGET (see the module docs).
pub const FY_RULES: &[FyRules] = &[rules(2025), rules(2026)];

pub const CESS_PCT: i64 = 4;

/// The rules for `fy_start`, and whether they are that FY's own (`false` = borrowed).
pub fn rules_for(fy_start: i32) -> (&'static FyRules, bool) {
    if let Some(r) = FY_RULES.iter().find(|r| r.fy_start == fy_start) {
        return (r, true);
    }
    let r = FY_RULES.iter().rev().find(|r| r.fy_start < fy_start).unwrap_or(&FY_RULES[0]);
    (r, false)
}

/// Tax on `taxable` paise after the rebate, before cess.
pub fn tax_before_cess(r: &RegimeRules, taxable: i64) -> i64 {
    let taxable = taxable.max(0);
    let mut tax = 0;
    let mut lower = 0;
    for (upper, pct) in r.slabs {
        let top = upper.map_or(taxable, |u| (u * 100).min(taxable));
        if top > lower {
            tax += (top - lower) * pct / 100;
        }
        match upper {
            Some(u) => lower = u * 100,
            None => break,
        }
    }
    let limit = r.rebate_up_to * 100;
    if taxable <= limit {
        0
    } else if r.marginal_relief {
        tax.min(taxable - limit)
    } else {
        tax
    }
}

pub fn with_cess(tax: i64) -> i64 {
    tax + tax * CESS_PCT / 100
}

/// Deductions claimed for the year, per section, in paise (uncapped).
#[derive(Debug, Clone, Copy, Default)]
pub struct Claims {
    pub s80c: i64,
    pub s80ccd1b: i64,
    pub s80d: i64,
    pub s24b: i64,
    pub hra: i64,
}

impl Claims {
    pub fn set(&mut self, section: &str, paise: i64) {
        match section {
            "80C" => self.s80c = paise,
            "80CCD1B" => self.s80ccd1b = paise,
            "80D" => self.s80d = paise,
            "24B" => self.s24b = paise,
            "HRA" => self.hra = paise,
            _ => {}
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Regime {
    pub old_tax_paise: i64,
    pub new_tax_paise: i64,
    /// `old` | `new` (ties go to `new`)
    pub better: &'static str,
    pub note: String,
    /// Total old-regime deductions (80C, 80CCD1B, 80D, 24B, HRA; not the standard deduction) at
    /// which the old regime first becomes cheaper. `None` = the new regime wins at any level.
    pub break_even_deductions_paise: Option<i64>,
    /// `break_even - claimed`, never below zero. `None` when there is no break-even.
    pub deductions_needed_paise: Option<i64>,
}

/// Old-regime tax with cess, when `deductions` (paise, on top of the standard deduction) are claimed.
fn old_tax_with_deductions(fy: &FyRules, gross: i64, deductions: i64) -> i64 {
    with_cess(tax_before_cess(&fy.old, gross - fy.old.standard_deduction * 100 - deductions))
}

/// The smallest total old-regime deduction (paise, excluding the standard deduction) at which the
/// old regime's tax is strictly below the new regime's, for a `gross` salary in paise. Ties go to
/// the new regime, as in [`compare`]. `None` when the new regime wins even with every rupee of
/// salary deducted (for example when the new regime's rebate makes its tax zero).
pub fn break_even_deductions(gross: i64, fy_start: i32) -> Option<i64> {
    let (fy, _) = rules_for(fy_start);
    let new_tax = with_cess(tax_before_cess(&fy.new, gross - fy.new.standard_deduction * 100));
    // Old-regime tax never rises as deductions rise, so "old wins" is monotone in `d`.
    let old_wins = |d: i64| old_tax_with_deductions(fy, gross, d) < new_tax;
    if old_wins(0) {
        return Some(0);
    }
    let mut hi = gross.max(0);
    if !old_wins(hi) {
        return None;
    }
    let mut lo = 0;
    while hi - lo > 1 {
        let mid = lo + (hi - lo) / 2;
        if old_wins(mid) {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    Some(hi)
}

/// Compares the two regimes for a year's `gross_paise` salary.
pub fn compare(gross: i64, claims: Claims, fy_start: i32) -> Regime {
    let (fy, own) = rules_for(fy_start);
    let cap = |section: &str, v: i64| limit_for(section).map_or(v, |l| v.min(l));
    let claimed = cap("80C", claims.s80c)
        + cap("80CCD1B", claims.s80ccd1b)
        + cap("80D", claims.s80d)
        + cap("24B", claims.s24b)
        + claims.hra;
    let old_deductions = fy.old.standard_deduction * 100 + claimed;
    let new_tax = with_cess(tax_before_cess(&fy.new, gross - fy.new.standard_deduction * 100));
    let old_tax = with_cess(tax_before_cess(&fy.old, gross - old_deductions));
    let break_even = break_even_deductions(gross, fy_start);

    let mut note = format!(
        "Estimate for salary income only, using the {} slabs (new regime: 75,000 standard deduction, no tax up to 12L taxable; \
         old regime: 50,000 standard deduction, 87A rebate up to 5L), plus 4% cess. No surcharge, employer NPS or capital gains. \
         Slabs change in most Budgets: check them every year.",
        fy_label(fy.fy_start)
    );
    if !own {
        note.push_str(&format!(" There is no slab table for {} yet, so this uses {}'s.", fy_label(fy_start), fy_label(fy.fy_start)));
    }
    if gross <= 0 {
        note.push_str(" Add your salary under income to get a real comparison.");
    }
    Regime {
        old_tax_paise: old_tax,
        new_tax_paise: new_tax,
        better: if old_tax < new_tax { "old" } else { "new" },
        note,
        break_even_deductions_paise: break_even,
        deductions_needed_paise: break_even.map(|b| (b - claimed).max(0)),
    }
}

// ---------- long-term equity gains ----------

/// Yearly exemption for long-term gains on listed equity, in paise (1.25 lakh).
pub const LTCG_EXEMPTION_PAISE: i64 = 1_25_000_00;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct LtcgHarvest {
    pub exemption_paise: i64,
    /// Realised gains so far, capped at the exemption.
    pub used_paise: i64,
    pub left_paise: i64,
    /// Days from `today` to 31 March, counting today. Zero once the FY is over.
    pub days_left_in_fy: i64,
    pub note: String,
}

/// Whole rupees with Indian grouping: `1_25_000_00` -> `1,25,000`.
fn inr_text(paise: i64) -> String {
    let s = (paise / 100).to_string();
    if s.len() <= 3 {
        return s;
    }
    let (mut rest, last3) = s.split_at(s.len() - 3);
    let mut groups = Vec::new();
    while rest.len() > 2 {
        let (head, tail) = rest.split_at(rest.len() - 2);
        groups.push(tail);
        rest = head;
    }
    groups.push(rest);
    groups.reverse();
    format!("{},{}", groups.join(","), last3)
}

/// How much of the long-term equity gains exemption is left for `fy_start`, given the gains already
/// realised (paise). Pure: the caller supplies `today`.
pub fn ltcg_harvest(realised_ltcg_paise: i64, fy_start: i32, today: Date) -> LtcgHarvest {
    let exemption = LTCG_EXEMPTION_PAISE;
    let realised = realised_ltcg_paise.max(0);
    let used = realised.min(exemption);
    let left = exemption - used;
    let (_, end) = fy_bounds(fy_start);
    let days_left = if today > end { 0 } else { (end - today).whole_days() + 1 };
    let end_text = format!("31 Mar {}", end.year());
    let note = if days_left == 0 {
        format!("This FY ended on {end_text}, so its exemption has lapsed.")
    } else if left == 0 {
        let over = if realised > exemption { " Gains above it are taxable." } else { "" };
        format!("You've used this year's ₹{} exemption.{over}", inr_text(exemption))
    } else {
        format!(
            "Book up to ₹{} of long-term equity gains before {end_text} tax-free, and reinvest.",
            inr_text(left)
        )
    };
    LtcgHarvest { exemption_paise: exemption, used_paise: used, left_paise: left, days_left_in_fy: days_left, note }
}

// ---------- tagged fixed payments ----------

/// A fixed payment tagged to a section.
#[derive(Debug, Clone)]
pub struct Payment {
    pub amount: i64,
    pub day: u8,
    pub yearly: bool,
    pub due_month: Option<u8>,
    pub active: bool,
    /// `year * 12 + month0` bounds, as in `planner::month_index`.
    pub start_month: Option<i32>,
    pub end_month: Option<i32>,
}

/// `(paid so far, projected to March)` for the FY, in paise. A monthly payment counts once for
/// every due date inside the FY (clamped to short months); one due today counts as paid. A yearly
/// payment with an unknown month is not counted.
pub fn payments_in_fy(p: &Payment, fy_start: i32, today: Date) -> (i64, i64) {
    if !p.active {
        return (0, 0);
    }
    let (mut paid, mut projected) = (0, 0);
    for k in 0..12 {
        let (y, m) = add_months(fy_start, Month::April, k);
        let idx = y * 12 + (m as i32 - 1);
        if p.start_month.is_some_and(|s| idx < s) || p.end_month.is_some_and(|e| idx > e) {
            continue;
        }
        if p.yearly && p.due_month != Some(m as u8) {
            continue;
        }
        if clamped(y, m, p.day) <= today {
            paid += p.amount;
        } else {
            projected += p.amount;
        }
    }
    (paid, projected)
}

// ---------- reminders ----------

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Reminder {
    pub date: Date,
    pub text: String,
    /// Negative once the date has passed.
    pub days_left: i64,
}

pub fn reminders(fy_start: i32, today: Date) -> Vec<Reminder> {
    let d = |y: i32, m: Month, day: u8| Date::from_calendar_date(y, m, day).expect("valid");
    let end = fy_start + 1;
    let mut items = vec![
        (d(end, Month::January, 15), "Submit investment proofs to your employer (usually January)".to_string()),
        // The return for FY 2026-27 is filed by July 2027, after the FY ends.
        (d(end, Month::July, 31), format!("File your return by 31 Jul {end}")),
    ];
    for (date, label) in [
        (d(fy_start, Month::June, 15), "15 Jun"),
        (d(fy_start, Month::September, 15), "15 Sep"),
        (d(fy_start, Month::December, 15), "15 Dec"),
        (d(end, Month::March, 15), "15 Mar"),
    ] {
        items.push((date, format!("Advance tax is due {label} if you have income beyond salary")));
    }
    items.sort_by_key(|(date, _)| *date);
    items.into_iter().map(|(date, text)| Reminder { date, text, days_left: (date - today).whole_days() }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::date;

    fn inr(rupees: i64) -> i64 {
        rupees * 100
    }

    #[test]
    fn new_regime_is_free_up_to_12_lakh() {
        let (fy, _) = rules_for(2026);
        assert_eq!(tax_before_cess(&fy.new, inr(12_00_000)), 0);
        assert_eq!(with_cess(tax_before_cess(&fy.new, inr(12_00_000))), 0);
    }

    #[test]
    fn new_regime_16_lakh() {
        // 4-8L 5% = 20,000; 8-12L 10% = 40,000; 12-16L 15% = 60,000.
        let (fy, _) = rules_for(2026);
        assert_eq!(tax_before_cess(&fy.new, inr(16_00_000)), inr(1_20_000));
        assert_eq!(with_cess(inr(1_20_000)), inr(1_24_800));
    }

    #[test]
    fn new_regime_marginal_relief_just_above_12_lakh() {
        let (fy, _) = rules_for(2026);
        // Slab tax on 12L is 60,000, so 12.10L would be 61,500; relief caps it at the 10,000 over 12L.
        assert_eq!(tax_before_cess(&fy.new, inr(12_10_000)), inr(10_000));
        // 13L: slab tax 75,000 is below the 1,00,000 excess, so no relief applies.
        assert_eq!(tax_before_cess(&fy.new, inr(13_00_000)), inr(75_000));
    }

    #[test]
    fn old_regime_10_lakh() {
        // 2.5-5L 5% = 12,500; 5-10L 20% = 1,00,000.
        let (fy, _) = rules_for(2026);
        assert_eq!(tax_before_cess(&fy.old, inr(10_00_000)), inr(1_12_500));
        assert_eq!(with_cess(inr(1_12_500)), inr(1_17_000));
    }

    #[test]
    fn old_regime_rebate_to_5_lakh_only() {
        let (fy, _) = rules_for(2026);
        assert_eq!(tax_before_cess(&fy.old, inr(5_00_000)), 0);
        assert_eq!(tax_before_cess(&fy.old, inr(5_10_000)), inr(12_500 + 2_000), "no marginal relief in the old regime");
    }

    #[test]
    fn compare_picks_the_cheaper_regime() {
        // 18L salary, full 1.5L 80C + 50k NPS + 25k health + 2L home loan interest + 1L HRA.
        let claims = Claims { s80c: inr(1_50_000), s80ccd1b: inr(50_000), s80d: inr(25_000), s24b: inr(2_00_000), hra: inr(1_00_000) };
        let r = compare(inr(18_00_000), claims, 2026);
        // New: taxable 17.25L -> 20k+40k+60k + 1.25L*20% = 25k -> 1,45,000 -> 1,50,800.
        assert_eq!(r.new_tax_paise, inr(1_50_800));
        // Old: taxable 18L - 50k - 1.5L - 50k - 25k - 2L - 1L = 12.25L -> 12,500 + 1,00,000 + 2.25L*30% = 67,500 -> 1,80,000 -> 1,87,200.
        assert_eq!(r.old_tax_paise, inr(1_87_200));
        assert_eq!(r.better, "new");
        assert!(r.note.contains("2026-27"));
        // Over-claims are capped: 80C of 5L counts only 1.5L.
        let capped = compare(inr(18_00_000), Claims { s80c: inr(5_00_000), ..claims }, 2026);
        assert_eq!(capped.old_tax_paise, r.old_tax_paise);
    }

    #[test]
    fn unknown_fy_borrows_the_latest_table() {
        let r = compare(inr(10_00_000), Claims::default(), 2031);
        assert!(r.note.contains("no slab table for 2031-32"), "{}", r.note);
        assert_eq!(compare(inr(10_00_000), Claims::default(), 2026).new_tax_paise, r.new_tax_paise);
    }

    #[test]
    fn financial_year_boundaries() {
        assert_eq!(fy_start_year(date!(2027 - 03 - 31)), 2026);
        assert_eq!(fy_start_year(date!(2027 - 04 - 01)), 2027);
        assert_eq!(fy_start_year(date!(2026 - 04 - 01)), 2026);
        assert_eq!(fy_label(2026), "2026-27");
        assert_eq!(fy_label(2099), "2099-00");
        assert_eq!(parse_fy("2026-27"), Some(2026));
        assert_eq!(parse_fy("2026-28"), None);
        assert_eq!(parse_fy("26-27"), None);
        assert_eq!(parse_fy("2099-00"), Some(2099));
        assert_eq!(fy_bounds(2026), (date!(2026 - 04 - 01), date!(2027 - 03 - 31)));
    }

    fn sip(day: u8) -> Payment {
        Payment { amount: 5_000_00, day, yearly: false, due_month: None, active: true, start_month: None, end_month: None }
    }

    #[test]
    fn monthly_payments_count_due_dates_up_to_today() {
        // Due on the 5th: Apr..Nov 2026 = 8 paid by 8 Nov, Dec..Mar = 4 projected.
        assert_eq!(payments_in_fy(&sip(5), 2026, date!(2026 - 11 - 08)), (8 * 5_000_00, 4 * 5_000_00));
        // Due on the 10th: Nov's isn't due yet.
        assert_eq!(payments_in_fy(&sip(10), 2026, date!(2026 - 11 - 08)), (7 * 5_000_00, 5 * 5_000_00));
        // Due today counts as paid.
        assert_eq!(payments_in_fy(&sip(8), 2026, date!(2026 - 11 - 08)).0, 8 * 5_000_00);
        // Before the FY: all projected; after: all paid.
        assert_eq!(payments_in_fy(&sip(5), 2026, date!(2026 - 03 - 31)), (0, 12 * 5_000_00));
        assert_eq!(payments_in_fy(&sip(5), 2026, date!(2027 - 04 - 01)), (12 * 5_000_00, 0));
        // The 31st is clamped to February's 28th.
        assert_eq!(payments_in_fy(&sip(31), 2026, date!(2027 - 02 - 28)).0, 11 * 5_000_00);
    }

    #[test]
    fn payments_respect_range_yearly_and_inactive() {
        let mut p = sip(5);
        p.start_month = Some(2026 * 12 + 9); // Oct 2026
        p.end_month = Some(2026 * 12 + 11); // Dec 2026
        assert_eq!(payments_in_fy(&p, 2026, date!(2026 - 11 - 08)), (2 * 5_000_00, 5_000_00));
        let y = Payment { yearly: true, due_month: Some(2), amount: 24_000_00, ..sip(10) };
        assert_eq!(payments_in_fy(&y, 2026, date!(2026 - 11 - 08)), (0, 24_000_00));
        assert_eq!(payments_in_fy(&y, 2026, date!(2027 - 02 - 10)), (24_000_00, 0));
        assert_eq!(payments_in_fy(&Payment { yearly: true, due_month: None, ..sip(10) }, 2026, date!(2026 - 11 - 08)), (0, 0));
        assert_eq!(payments_in_fy(&Payment { active: false, ..sip(5) }, 2026, date!(2026 - 11 - 08)), (0, 0));
    }

    #[test]
    fn reminders_are_dated_and_sorted() {
        let r = reminders(2026, date!(2026 - 11 - 08));
        assert_eq!(r.len(), 6);
        assert!(r.windows(2).all(|w| w[0].date <= w[1].date));
        let dec = r.iter().find(|x| x.date == date!(2026 - 12 - 15)).unwrap();
        assert_eq!(dec.days_left, 37);
        let filing = r.last().unwrap();
        assert_eq!((filing.date, filing.text.as_str()), (date!(2027 - 07 - 31), "File your return by 31 Jul 2027"));
        assert!(r.iter().any(|x| x.text.starts_with("Submit investment proofs")));
        assert!(r.iter().find(|x| x.date == date!(2026 - 06 - 15)).unwrap().days_left < 0);
    }

    #[test]
    fn break_even_for_18_lakh_is_exact() {
        // New: 17.25L taxable -> 1,45,000 tax, 1,50,800 with cess.
        // Old: taxable = 18L - 50k - d. Above 10L the tax is 1,12,500 + 30% of the excess, so the
        // old tax before cess must fall to 1,45,000 (1,50,800 with cess), i.e. taxable must reach
        // 11,08,333.33. Deductions therefore have to pass 18L - 50k - 11,08,333.33 = 6,41,666.67.
        let be = break_even_deductions(inr(18_00_000), 2026);
        assert_eq!(be, Some(64_166_667));
        let d = be.unwrap();
        let hra_only = |hra| Claims { hra, ..Claims::default() };
        assert_eq!(compare(inr(18_00_000), hra_only(d), 2026).better, "old");
        assert_eq!(compare(inr(18_00_000), hra_only(d - 1), 2026).better, "new");
    }

    #[test]
    fn break_even_none_when_new_regime_is_free() {
        // 10L gross: 9.25L taxable in the new regime, under the 12L rebate, so its tax is 0.
        assert_eq!(break_even_deductions(inr(10_00_000), 2026), None);
        let r = compare(inr(10_00_000), Claims::default(), 2026);
        assert_eq!((r.break_even_deductions_paise, r.deductions_needed_paise), (None, None));
    }

    #[test]
    fn deductions_needed_is_the_gap_to_break_even() {
        // The claims total 5.25L (80C 1.5L, NPS 50k, 80D 25k, home loan 2L, HRA 1L), against 6,41,666.67.
        let claims = Claims { s80c: inr(1_50_000), s80ccd1b: inr(50_000), s80d: inr(25_000), s24b: inr(2_00_000), hra: inr(1_00_000) };
        let r = compare(inr(18_00_000), claims, 2026);
        assert_eq!(r.break_even_deductions_paise, Some(64_166_667));
        assert_eq!(r.deductions_needed_paise, Some(64_166_667 - inr(5_25_000)));
        assert_eq!(r.deductions_needed_paise, Some(1_16_66_667));
        // Once past the break-even, nothing is needed and the old regime wins.
        let past = compare(inr(18_00_000), Claims { hra: inr(7_00_000), ..Claims::default() }, 2026);
        assert_eq!((past.better, past.deductions_needed_paise), ("old", Some(0)));
    }

    #[test]
    fn old_regime_tax_never_rises_with_deductions() {
        let mut prev = i64::MAX;
        let (fy, _) = rules_for(2026);
        for d in (0..=inr(20_00_000)).step_by(inr(25_000) as usize) {
            let t = old_tax_with_deductions(fy, inr(18_00_000), d);
            assert!(t <= prev, "tax rose at deductions {d}");
            prev = t;
        }
    }

    #[test]
    fn ltcg_harvest_maths() {
        let today = date!(2026 - 11 - 08);
        let none = ltcg_harvest(0, 2026, today);
        assert_eq!(none.exemption_paise, 1_25_000_00);
        assert_eq!((none.used_paise, none.left_paise), (0, 1_25_000_00));
        // 8 Nov to 31 Mar is 143 days; counting today gives 144.
        assert_eq!(none.days_left_in_fy, 144);
        assert!(none.note.starts_with("Book up to ₹1,25,000 of long-term equity gains before 31 Mar 2027"), "{}", none.note);

        let part = ltcg_harvest(1_00_000_00, 2026, today);
        assert_eq!((part.used_paise, part.left_paise), (1_00_000_00, 25_000_00));
        assert!(part.note.contains("Book up to ₹25,000"), "{}", part.note);

        let over = ltcg_harvest(3_00_000_00, 2026, today);
        assert_eq!((over.used_paise, over.left_paise), (1_25_000_00, 0));
        assert!(over.note.contains("used this year's ₹1,25,000 exemption. Gains above it are taxable."), "{}", over.note);

        assert_eq!(ltcg_harvest(-5_00, 2026, today).used_paise, 0);
        assert_eq!(ltcg_harvest(0, 2026, date!(2027 - 03 - 31)).days_left_in_fy, 1);
        let ended = ltcg_harvest(0, 2026, date!(2027 - 04 - 01));
        assert_eq!(ended.days_left_in_fy, 0);
        assert!(ended.note.contains("ended on 31 Mar 2027"), "{}", ended.note);
    }

    #[test]
    fn inr_text_uses_indian_grouping() {
        assert_eq!(inr_text(0), "0");
        assert_eq!(inr_text(999_00), "999");
        assert_eq!(inr_text(1_000_00), "1,000");
        assert_eq!(inr_text(25_000_00), "25,000");
        assert_eq!(inr_text(1_25_000_00), "1,25,000");
        assert_eq!(inr_text(inr(1_00_00_000)), "1,00,00,000");
    }
}
