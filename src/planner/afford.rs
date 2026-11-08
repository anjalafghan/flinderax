//! "Can I afford it?": pay now, or on EMI. The answer is yes, yes-but-from-savings, wait until a
//! date, or no with why, plus what it really costs. Pure: the caller says whether a spend on a
//! given date leaves any month short (it runs the projection for that).

use serde::{Deserialize, Serialize};
use time::Date;

use super::cycle::{add_months, clamped};

/// GST on a processing fee (18%).
const GST_BPS: i64 = 1_800;

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum Pay {
    Now,
    Emi {
        months: i64,
        /// Annual interest; 0 for a "no-cost" EMI.
        #[serde(default)]
        rate_bps: i64,
        /// Processing fee (GST is added on top).
        #[serde(default)]
        fee_paise: i64,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Answer {
    Yes,
    YesFromSavings,
    Wait,
    No,
}

#[derive(Debug, Clone, Serialize)]
pub struct Verdict {
    pub answer: Answer,
    pub headline: String,
    pub reasons: Vec<String>,
    /// Safe to Spend after paying now (negative = beyond it).
    pub safe_to_spend_after_paise: i64,
    /// EMI only.
    pub monthly_paise: i64,
    /// Everything you'd pay in the end.
    pub total_cost_paise: i64,
    /// Interest + fees + GST over paying now.
    pub extra_cost_paise: i64,
    pub wait_until: Option<Date>,
}

/// The standard reducing-balance EMI (rounded up to the rupee).
pub fn emi_monthly(principal: i64, rate_bps: i64, months: i64) -> i64 {
    if months <= 0 {
        return principal;
    }
    if rate_bps <= 0 {
        return (principal + months - 1) / months;
    }
    let r = rate_bps as f64 / 10_000.0 / 12.0;
    let f = (1.0 + r).powi(months as i32);
    let m = principal as f64 * r * f / (f - 1.0);
    ((m / 100.0).ceil() * 100.0) as i64
}

/// What a fee costs with GST.
pub fn with_gst(fee: i64) -> i64 {
    fee + fee * GST_BPS / 10_000
}

/// Decides. `short_if_spent_on(date)` returns the worst monthly shortfall a lump of `amount` on
/// that date causes over the projection (`None` = every month stays covered). `rest_savings` is
/// what you save beyond the SIP each month (an EMI comes out of it first).
pub fn afford(
    amount: i64,
    pay: Pay,
    today: Date,
    safe_to_spend: i64,
    rest_savings: i64,
    horizon_months: i64,
    short_if_spent_on: impl Fn(Date) -> Option<i64>,
    money: impl Fn(i64) -> String,
) -> Verdict {
    match pay {
        Pay::Now => {
            let after = safe_to_spend - amount;
            let base = Verdict {
                answer: Answer::Yes,
                headline: String::new(),
                reasons: vec![],
                safe_to_spend_after_paise: after,
                monthly_paise: 0,
                total_cost_paise: amount,
                extra_cost_paise: 0,
                wait_until: None,
            };
            if after >= 0 {
                return Verdict {
                    headline: "Yes, it fits".into(),
                    reasons: vec![format!("{} is still safe to spend until payday after it", money(after))],
                    ..base
                };
            }
            let Some(short) = short_if_spent_on(today) else {
                return Verdict {
                    answer: Answer::YesFromSavings,
                    headline: "Yes, but partly from savings".into(),
                    reasons: vec![
                        format!("{} more than is safe to spend until payday", money(-after)),
                        "No month ahead runs short, so it's savings you'd be using, not bills money".into(),
                    ],
                    ..base
                };
            };
            for k in 1..horizon_months {
                let (y, m) = add_months(today.year(), today.month(), k as i32);
                let date = clamped(y, m, 1);
                if short_if_spent_on(date).is_none() {
                    return Verdict {
                        answer: Answer::Wait,
                        headline: format!("Wait until {}", month_label(date)),
                        reasons: vec![
                            format!("Buying it now leaves a month {} short", money(short)),
                            format!("From {} nothing runs short", month_label(date)),
                        ],
                        wait_until: Some(date),
                        ..base
                    };
                }
            }
            Verdict {
                answer: Answer::No,
                headline: "Not right now".into(),
                reasons: vec![
                    format!("It leaves a month {} short, and it still would in {horizon_months} months", money(short)),
                    "Save for it as a goal instead".into(),
                ],
                ..base
            }
        }
        Pay::Emi { months, rate_bps, fee_paise } => {
            let monthly = emi_monthly(amount, rate_bps, months);
            let fees = with_gst(fee_paise);
            let total = monthly * months + fees;
            let extra = total - amount;
            let mut reasons = vec![format!("{} a month for {months} months", money(monthly))];
            reasons.push(if extra > 0 {
                format!("Costs {} more than paying now (interest, fee and GST)", money(extra))
            } else {
                "No extra cost over paying now".into()
            });
            let (answer, headline) = if monthly <= rest_savings {
                reasons.push(format!("It fits in the {} you save beyond your SIP each month", money(rest_savings)));
                (Answer::Yes, "Yes, on EMI".to_string())
            } else {
                reasons.push(format!(
                    "It's {} a month more than you save beyond your SIP, so it would cut your spending money",
                    money(monthly - rest_savings.max(0))
                ));
                (Answer::No, "Not on this EMI".to_string())
            };
            Verdict {
                answer,
                headline,
                reasons,
                safe_to_spend_after_paise: safe_to_spend - fees,
                monthly_paise: monthly,
                total_cost_paise: total,
                extra_cost_paise: extra,
                wait_until: None,
            }
        }
    }
}

fn month_label(d: Date) -> String {
    const MON: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
    format!("{} {}", MON[d.month() as usize - 1], d.year())
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::date;

    fn rupees(p: i64) -> String {
        format!("₹{}", p / 100)
    }
    const TODAY: Date = date!(2026 - 11 - 08);

    #[test]
    fn emi_maths() {
        // ₹1,00,000 at 12% for 12 months is ₹8,885 a month (rounded up to the rupee).
        assert_eq!(emi_monthly(10_000_000, 1_200, 12), 888_500);
        assert_eq!(emi_monthly(1_200_000, 0, 12), 100_000);
        assert_eq!(with_gst(19_900), 23_482);
    }

    #[test]
    fn now_within_safe_to_spend_is_yes() {
        let v = afford(500_000, Pay::Now, TODAY, 800_000, 0, 6, |_| Some(1), rupees);
        assert_eq!((v.answer, v.safe_to_spend_after_paise), (Answer::Yes, 300_000));
    }

    #[test]
    fn now_beyond_safe_but_no_month_short_is_from_savings() {
        let v = afford(1_500_000, Pay::Now, TODAY, 500_000, 0, 6, |_| None, rupees);
        assert_eq!(v.answer, Answer::YesFromSavings);
    }

    #[test]
    fn now_that_leaves_a_month_short_waits_for_the_first_safe_month() {
        let v = afford(5_000_000, Pay::Now, TODAY, 0, 0, 6, |d| (d < date!(2027 - 01 - 01)).then_some(200_000), rupees);
        assert_eq!((v.answer, v.wait_until), (Answer::Wait, Some(date!(2027 - 01 - 01))));
        assert_eq!(v.headline, "Wait until Jan 2027");
        let never = afford(5_000_000, Pay::Now, TODAY, 0, 0, 6, |_| Some(200_000), rupees);
        assert_eq!(never.answer, Answer::No);
    }

    #[test]
    fn no_cost_emi_still_counts_the_fee_and_gst() {
        let v = afford(6_000_000, Pay::Emi { months: 6, rate_bps: 0, fee_paise: 19_900 }, TODAY, 0, 1_500_000, 6, |_| None, rupees);
        assert_eq!((v.answer, v.monthly_paise, v.extra_cost_paise), (Answer::Yes, 1_000_000, 23_482));
        let tight = afford(6_000_000, Pay::Emi { months: 6, rate_bps: 0, fee_paise: 0 }, TODAY, 0, 500_000, 6, |_| None, rupees);
        assert_eq!(tight.answer, Answer::No);
    }
}
