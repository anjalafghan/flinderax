# Goal-based life financial planning for an Indian salaried individual or young couple (as of Nov 2026), for the "joy" app

Research date: 8 Nov 2026. Effective years: FY2025-26 (AY2026-27) and FY2026-27 (AY2027-28). The Income Tax Act 2025 replaces the 1961 Act from 1 Apr 2026, and sections are renumbered (e.g. 80CCD(2) becomes s.125, per one blog). App copy should show both the old and new section names during the transition.

Labels used below: **[OFFICIAL]** = government/regulator data or law; **[MARKET DATA]** = index or industry statistics; **[OPINION]** = rule of thumb or adviser/blog convention; **[MATH]** = standard finance formula, not a factual claim.

---

## 1. Inflation assumptions by category

### Takeaway
Headline CPI is ~5.5% now (Sep 2026, new 2024=100 series) and RBI projects 5.2% for FY27. The app should default general inflation to 6% (long-run planning convention), education and medical to 10%, wedding to ~8%, and house prices to ~6-7%. Only CPI is official; the category rates are market or opinion estimates.

### Cited Findings
- **[OFFICIAL]** CPI (combined) for Sep 2026 was 5.54% YoY (provisional), up from 4.83% in Aug. Rural 6.08%, urban 4.88%, food (CFPI) 7.74%. Inflation has risen each month since Apr 2026 (3.48%). — [Adda247 summarising MoSPI](https://currentaffairs.adda247.com/indias-retail-inflation-rises-to-5-54-in-september-2026-as-food-prices-surge/); [Finnovate](https://www.finnovate.in/learn/blog/india-cpi-inflation-september-2026); [MoSPI Sep 2026 press release PDF](https://www.mospi.gov.in/uploads/latestreleasesfiles/1791801549227-Press%20Release%20for%20the%20month%20of%20September,%202026.pdf). The headline number comes from secondary reports; I could not read it in the MoSPI PDF text directly.
- **[OFFICIAL]** The Oct 2026 CPI release is due 12 Nov 2026. — [MoSPI PDF](https://www.mospi.gov.in/uploads/latestreleasesfiles/1791801549227-Press%20Release%20for%20the%20month%20of%20September,%202026.pdf)
- **[OFFICIAL]** MoSPI moved the CPI base from 2012 to 2024=100 on 12 Feb 2026. The new basket has 358 items (was 299), with weights from HCES 2023-24. Jan 2026 inflation on the new series was 2.75%. Old and new series are not directly comparable, so MoSPI published linking factors and back series. — [Drishti IAS](https://www.drishtiias.com/daily-updates/daily-news-analysis/cpi-base-year-revised-to-2024/print_manually); [Upstox](https://upstox.com/learning-center/personal-finance/what-changed-in-indias-new-cpi-series-and-how-it-impacts-the-economy-and-inflation/article-1518/)
- **[OFFICIAL]** RBI MPC raised the repo rate 25 bps to 5.50% on 7 Oct 2026 (the first hike since Feb 2023) and changed its stance to "calibrated tightening". It now projects FY27 CPI at 5.2% (was 5.0%) and FY27 GDP growth at 7.1%. — [SCC Online](https://www.scconline.com/blog/post/2026/10/09/rbi-hikes-repo-rate-to-5-50-percent-october-2026/); [Swarajya](https://swarajyamag.com/economy/rbi-raises-repo-rate-to-55-per-cent-projects-fy27-growth-at-71-per-cent-and-inflation-at-52-per-cent). Before that the repo was 5.25% (held in Aug 2026). — [SCC Online Aug](https://www.scconline.com/blog/post/2026/08/10/rbi-repo-rate-unchanged-august-2026-neutral-stance/)
- **[OFFICIAL]** RBI's inflation target is 4% ±2% (the flexible inflation targeting framework). This is from background knowledge; I did not fetch a source for it this session.
- **[MARKET DATA]** Medical inflation: Aon projects Indian medical cost trend at 11.5% for 2026 and 10.1% for 2027. Net of ~4% general inflation, that is 7.4% and 6.1%. — [Telangana Today reporting Aon](https://telanganatoday.com/medical-costs-in-india-projected-to-rise-10-1pc-in-2027-aon). Other estimates put it at 12-14%. — [DSIJ](https://insights.dsij.in/dsijarticledetail/medical-inflation-is-running-at-12-14-annually-and-your-health-cover-is-probably-already-outdated-id010-56534)
- **[OPINION]** Education inflation is commonly put at 10-12% a year, about 2x CPI. No official series exists. — [Basunivesh](https://www.basunivesh.com/cost-of-education-in-india-2025-2040-fees-living-projections/); [Retirewise](https://www.retirewise.in/?p=3917)
- **[MARKET DATA]** Weddings: the WedMeGood Annual Wedding Report 2025-26 (survey of 2,000+ couples marrying Apr 2025-Mar 2026) found an average budget of about ₹39.5 lakh, ~8% higher YoY. Destination weddings averaged about ₹58 lakh (up from ₹51.1 lakh). One in four weddings was a destination wedding. City averages: Jaipur ₹73L, Delhi ₹38L, Bangalore/Hyderabad ₹37L, Mumbai ₹35L. 79% of couples paid from savings and 15% took loans. — [Free Press Journal](https://www.freepressjournal.in/business/2025-lavish-indian-weddings-thrive-with-8-higher-spending-despite-soaring-gold-prices); [Curly Tales](https://curlytales.com/india/ct-scoop/in-now-destination-weddings-jaipur-leads-indias-wedding-budget-spend-at-lakh-more/amp/); [Outlook Money](https://www.outlookmoney.com/banking/loan/big-fat-weddings-cost-couples-their-hard-earned-savings-only-15-per-cent-use-loans-report); [WedMeGood report page](https://www.wedmegood.com/blog/wedmegood-annual-wedding-industry-report-2025-2026/amp)
- **[OPINION]** Goal-specific inflation (house, car, education) is usually set above general inflation in goal calculators. — [ValueResearch](https://www.valueresearchonline.com/stories/17411/goal-driven-investing/)

### Inferences
- Suggested app defaults, all editable per goal: general/lifestyle 6%; education 10% (with 12% as a "conservative" toggle); medical 10% (Aon) up to 12%; wedding 8% (one survey data point, and the survey is skewed to affluent urban couples); car 4-5%; house price 6%. The car and house rates are my opinion; I found no source for them.
- The WedMeGood averages are skewed upward by affluent urban respondents. The app should let the user build a line-item wedding budget (venue, catering, outfits, jewellery, photography, decor) instead of anchoring on ₹39.5L.
- Because the CPI series changed in Feb 2026, any "long-run CPI average" the app shows must be computed from MoSPI's linked back series, or labelled as old-series (2012 base).

### Gaps
- I did not find a sourced long-run (10/20-year) CPI average. Earlier reporting puts the 2012-2024 old-series average at about 5.5-6%, but I could not verify it this session.
- There is no official education or housing inflation series in the results. The RBI House Price Index was not checked.
- Gold-price-linked jewellery inflation, which drives wedding costs, was not quantified.

---

## 2. Expected return assumptions by asset class (pre-tax, post-tax, real)

### Takeaway
Fixed-income rates are official and current: PPF 7.1%, EPF 8.25% (FY26), SSY 8.2%, NSC 7.7%. Equity's 20-year history is about 11.5-12.5% CAGR (Nifty 50 TRI). Planners commonly assume 10-12% for equity and 6-7% for debt, which gives real equity returns of roughly 4-6% at 6% inflation.

### Cited Findings
- **[OFFICIAL]** Small savings rates for Oct-Dec 2026 (Q3 FY27) are unchanged: PPF 7.1%, SSY 8.2%, NSC 7.7%. Sources disagree on whether this is the 9th or 10th quarter without a change. — [Business Today](https://www.businesstoday.in/personal-finance/story/small-savings-scheme-interest-rates-unchanged-for-q3-fy27-check-ppf-scss-ssy-rates-558892-2026-09-30); [Outlook Money](https://www.outlookmoney.com/retirement/pension/small-savings-rates-unchanged-ppf-at-710-per-cent-nsc-at-770-per-cent)
- **[OFFICIAL]** EPF interest for FY2025-26 is 8.25%, the same as FY24 and FY25. The CBT approved it on 2 Mar 2026, the Finance Ministry concurred, and EPFO issued the circular on 1 Jul 2026. The recent low was 8.10% (FY22). — [Business Today](https://www.businesstoday.in/personal-finance/story/epfo-approves-825-interest-rate-for-2025-26-518810-2026-03-02); [Angel One](https://www.angelone.in/news/personal-finance/epfo-interest-rate-set-at-8-25-for-fy26-government-confirms-final-approval)
- **[MARKET DATA]** Nifty 50 TRI 20-year CAGR is about 12.4% (NSE whitepaper, Mar 2026, as cited) — [Bajaj AMC](https://www.bajajamc.com/knowledge-centre/nifty-50-historical-returns); [Holistic Investment](https://www.holisticinvestment.in/nifty-50-prediction-2030-2035-2040-2045/). A May 2026 study gives about 11.4% — [Business Today](https://www.businesstoday.in/mutual-funds/story/can-indian-equities-and-mutual-funds-still-create-long-term-wealth-for-investors-531157-2026-05-13). One source says the 20-year rolling CAGR fell below 10% at FY26 end — [Finnovate](https://www.finnovate.in/learn/blog/nifty-20-year-cagr-below-10-percent-history). These sources conflict, depending on window and end date.
- **[MARKET DATA]** No rolling 7-year or 10-year Nifty 50 TRI period has had a negative return since inception. — [Business Today](https://www.businesstoday.in/markets/stocks/story/nifty-created-wealth-but-wall-street-created-more-what-20-years-of-data-reveal-536396-2026-06-11). The average rolling 10-year CAGR is about 14.0% (data to Aug 2026). — [Craytheon](https://craytheon.com/charts/nifty-50-rolling-returns/)
- **[OPINION]** Calculator defaults vary. Arthgyaan uses 11% equity and 3% debt, both post-tax (debt is very conservative). Another goal-SIP calculator uses ~12% for large-cap and ~14% for mid-cap. One model sets equity = inflation + risk premium and debt = inflation − offset. — [Arthgyaan calc](https://arthgyaan.com/calc); [sum.money](https://sum.money/in/goal-sip-calculator/); [Investyadnya](https://blog.investyadnya.in/assumptions-for-financial-goals-calculation/)
- **[OPINION]** Kuvera ties SIPs to named goals and shows a recommended portfolio during goal planning. I found no published Kuvera, ET Money or Scripbox return or inflation defaults. — [Kuvera blog 2019](https://kuvera.in/blog/nov-2019-kuvera-recommended-portfolio-update/amp/); [The Quint](https://www.thequint.com/save-and-grow/kuvera-wealth-management-the-power-of-goal-based-investing-demystified-heres-how-it-works)
- **[OFFICIAL]** SEBI's investor site hosts a goal planner calculator. — [SEBI Visual Goal Planner](https://investor.sebi.gov.in/calculators/Visual_Gold_Planner.html)
- **[MATH]** Real return = (1 + nominal) / (1 + inflation) − 1. — [Investyadnya](https://blog.investyadnya.in/assumptions-for-financial-goals-calculation/)

### Inferences (suggested app defaults; [OPINION] unless noted)
| Asset | Nominal pre-tax default | Tax treatment (see §6) | Approx. post-tax (30% slab) | Real at 6% CPI |
|---|---|---|---|---|
| Equity MF (large/flexi/index) | 10-11% (conservative vs ~12% history) | LTCG 12.5% above ₹1.25L | ~9.5-10% | ~3.5-4% |
| Mid/small-cap equity | 12% with higher volatility | same | ~11% | ~4.5% |
| Debt MF (bought after 1 Apr 2023) | 6.5-7% | slab rate | ~4.6-4.9% | ~-1 to -1.5% |
| Bank FD / RD (HDFC, Kotak) | prevailing card rate, ~6.5-7% | slab rate, interest taxed yearly | ~4.6-4.9% | negative |
| PPF | 7.1% (official, resets quarterly) | EEE | 7.1% | ~1% |
| EPF | 8.25% (official FY26) | EEE up to limits | 8.25% | ~2.1% |
| NPS (mixed) | 9-10% for 50-75% equity | 60% lump sum tax-free, annuity taxed | ~9% | ~3% |
| Gold (ETF/fund) | 7-8% long-run | LTCG after 24 months (verify) | ~7% | ~1-2% |
- Wherever possible, show returns as real and post-tax. For a 30%-slab user, FDs and debt funds have negative real post-tax returns at current inflation. This is a useful insight for "joy" to surface for RD/FD-heavy users.
- FD/RD rates for HDFC and Kotak should come from live card rates, not hard-coded values. The repo hike (Oct 2026) may push deposit rates up.

### Gaps
- No primary source on how SEBI RIAs or robo-advisers set capital market assumptions. Freefincal's robo template lets users enter their own returns; ET Money and Scripbox did not appear in results.
- Gold long-run return and current debt-fund YTMs were not sourced.
- Current HDFC/Kotak FD card rates were not fetched.

---

## 3. Goal planning maths

### Takeaway
Each goal needs these steps: inflate today's cost to the target year at the goal's inflation rate, subtract the future value of savings already earmarked, solve for the monthly SIP (flat or step-up) at a blended expected return, and step down the equity share over the last 3-5 years. These are standard formulas.

### Cited Findings
- **[MATH]** Inflate today's cost to the target year first, then solve for the SIP. Goal inflation for house, car and education should be higher than CPI. — [ValueResearch](https://www.valueresearchonline.com/stories/17411/goal-driven-investing/); [sum.money](https://sum.money/in/goal-sip-calculator/)
- **[MARKET DATA]** Arthgyaan backtests goal-based SIP investing in India, for both SIP and lump-sum cases. — [Arthgyaan](https://arthgyaan.com/blog/how-goal-based-investing-has-performed-in-the-past-in-india.html); [Arthgyaan lump sum](https://arthgyaan.com/blog/how-goal-based-investing-has-performed-in-the-past-in-india-for-lump-sum.html)

### Formulas ([MATH], standard; not source-dependent)
- Future goal cost: `FV_goal = PV_cost × (1 + g)^n`, where g is goal inflation and n is years.
- Future value of existing earmarked corpus: `FV_existing = C0 × (1 + r)^n`
- Gap: `G = max(0, FV_goal − FV_existing)`
- Monthly rate: `i = (1 + r)^(1/12) − 1` (the effective conversion; some apps use r/12 instead, which gives slightly different answers, so pick one and document it)
- Required flat SIP, paid at the start of each month (annuity-due, as mutual fund SIPs are): `SIP = G × i / [((1 + i)^N − 1) × (1 + i)]`, where N = 12n
- Step-up SIP with annual increase s: year k's monthly amount is `SIP_1 × (1 + s)^(k−1)`. Corpus = Σ_{k=1..n} SIP_1(1+s)^(k−1) × [((1+i)^12 − 1)/i × (1+i)] × (1+i)^(12(n−k)). Solve `SIP_1 = G / Σ(...)` in closed form or numerically.
- RD maturity (Indian banks compound quarterly): `M = Σ_{t=1..N} P × (1 + R/4)^(4 × (N − t + 1)/12)`. Per-bank rounding varies, so validate against the HDFC/Kotak calculators.
- FD maturity with quarterly compounding: `A = P × (1 + R/4)^(4n)`. TDS is 10% when interest exceeds the threshold. The FY26-27 threshold was not verified this session.
- Blended portfolio return: `r_p = w_e × r_e + w_d × r_d`. Recompute each year along the glide path.
- Real (inflation-adjusted) display value: `Real = FV / (1 + CPI)^n`
- Probability of success (optional what-if): Monte Carlo with lognormal returns. Equity σ is about 16-20%. That figure is my estimate, unsourced.

### Inferences (glide path and priority, [OPINION])
- Glide path by years to goal: more than 7 years, up to 70-80% equity; 5-7 years, 60%; 3-5 years, 40%; 1-3 years, 0-20% (move to debt, FD or liquid funds); under 1 year, 0% (liquid fund, FD or sweep). Implement it as systematic transfers, not lump-sum selling, and account for LTCG harvesting within the ₹1.25L exemption.
- Funding order (waterfall): (1) emergency fund, (2) term and health insurance premiums, (3) high-interest debt payoff, (4) near-term non-negotiable goals such as the wedding, (5) retirement (no loan exists for retirement), (6) child education, (7) house down payment, (8) car and lifestyle goals. Let users tag goals as needs or wants. The app allocates surplus by priority, then by deadline.
- Sinking funds for goals under 3 years should use RDs, FDs or liquid funds, which matches the user's existing RD/FD habit.

### Gaps
- No primary source for a standard Indian robo-adviser glide-path table.

---

## 4. Specific goals

### Takeaway
House: banks cap EMIs at about 50-60% of net income (FOIR), and advisers say keep all EMIs to 35-40%. Retirement: India-specific SWR research points to 3-3.5%, not 4%, so a multiplier of about 30-33x annual expenses. Wedding: survey average ~₹39.5L, mostly paid from savings.

### Cited Findings
**House**
- **[MARKET DATA / OPINION]** Banks use FOIR (fixed obligations to income ratio) of 50-60% of net income for salaried borrowers. Some pages cite 50% for ₹50k-1L net, 55% for ₹1-2L, and 60-65% above ₹2L. A calculator page lists HDFC at 50% standard and 55-65% for high earners (unverified against HDFC policy). Advisers suggest 35-40% of net income across all EMIs. — [Finnovate](https://www.finnovate.in/learn/blog/home-loan-emi-affordability-india); [Godrej Capital](https://www.godrejcapital.com/media-blog/knowledge-centre/what-is-foir); [Studio Matrx](https://www.studiomatrx.org/guides/home-loan-affordability-india-2026); [emicalculator.net](https://emicalculator.net/how-much-loan-can-i-get/)
- **[OFFICIAL]** Home-loan interest deduction under s.24(b) is up to ₹2L for self-occupied property, and is only available in the old regime. — [Business Today / Budget 2026 summary search](https://www.businesstoday.in/personal-finance/tax/story/tax-slabs-fy-2026-27-what-budget-2026-changed-for-individual-taxpayers-and-which-regime-works-best-514044-2026-02-01); [Housing.com](https://housing.com/news/tax-benefits-on-joint-home-loan/)

**Retirement and SWR**
- **[MARKET DATA]** Raju and Saraogi, "Balancing Acts: Safe withdrawal rates in the Indian context" (data 2000-2023), found a 40% equity allocation gave the best SWR, at about 3%. They recommend an SWR of 3-3.5% at 95% confidence. — [Business Standard](https://www.pressreader.com/india/business-standard/20240304/282149296266369); [SSRN copy on Scribd](https://www.scribd.com/document/597540633/SSRN-id4216077)
- **[OPINION]** An adviser argues Indian retirees should drop the 4% rule. Indian index history is short (Sensex since 1979, about 45 years), which barely covers one retirement. — [Business Today](https://www.businesstoday.in/personal-finance/retirement-planning/story/senior-savings-at-risk-advisor-says-indian-retirees-must-ditch-the-4-withdrawal-rule-485445-2025-07-21)
- **[OPINION]** Freefincal's retirement corpus example uses a first-year withdrawal rate of about 3.31% as the "bare minimum ideal corpus". — [Freefincal](https://freefincal.com/what-is-the-ideal-corpus-for-retirement-in-india/). Basunivesh discusses whether 3.5% is better than 4% for India. — [Basunivesh](https://www.basunivesh.com/safe-withdrawal-rate-india/)

**Wedding** — see the §1 WedMeGood data. 79% of couples paid from savings and 15% borrowed. — [Outlook Money](https://www.outlookmoney.com/banking/loan/big-fat-weddings-cost-couples-their-hard-earned-savings-only-15-per-cent-use-loans-report)

**Education** — 10-12% inflation ([OPINION]); see §1 sources and [Basunivesh cost projections 2025-2040](https://www.basunivesh.com/cost-of-education-in-india-2025-2040-fees-living-projections/).

### Inferences
- **Why 4% may not hold in India:** inflation is higher and more volatile (4-6% versus about 2-3% in the US data behind the 4% rule), the equity data history is short, sequence risk and currency risk exist, and retirements can run 35-50 years for FIRE. App default: SWR 3.0-3.5%, which means a corpus of about 28-33x first-year annual expenses. Use a FIRE preset of 3% (33x) or lower, ideally 40-50x for early retirement before 45 ([OPINION], a common Freefincal-style stance).
- **Retirement corpus formula ([MATH]):** expenses at retirement `E_R = E_today × (1+g)^n`. Corpus with inflation-indexed withdrawals over T years at post-retirement return r_post: `Corpus = E_R × [1 − ((1+g)/(1+r_post))^T] / (r_post − g) × (1 + r_post)` (annuity-due growing at g). If r_post = g, then Corpus = E_R × T. Show it beside the SWR multiplier (1/SWR).
- **House rules for the app (rules of thumb, [OPINION]):** (a) total EMIs ≤ 35-40% of combined take-home pay, with the bank's FOIR shown as a hard ceiling; (b) down payment of at least 20%, because the RBI LTV cap is 75-90% depending on loan size (from background knowledge, unverified this session); (c) price ≤ about 5x gross annual household income (common blog heuristic, unsourced); (d) rent vs buy: compare rental yield (Indian metros are often 2-3%) against the home-loan rate (about 8-9%). If the price-to-annual-rent ratio is above about 25-30x, renting and investing the difference usually wins. These numbers are unsourced.
- **EMI formula ([MATH]):** `EMI = P × i × (1+i)^N / ((1+i)^N − 1)`, with i = annual rate / 12.
- **Car ([OPINION], a common Indian blog adaptation of the US 20/4/10 rule):** at least 20% down, a loan of no more than 4 years, and an EMI of no more than 10-15% of take-home pay. Unsourced this session.

### Gaps
- There is no Indian "28/36" rule. The nearest equivalent is bank FOIR, which is a lender ceiling, not advice.
- The RBI LTV caps and current home-loan rates after the Oct 2026 repo hike were not fetched.
- I did not read the full Raju-Saraogi paper (glide paths, failure-rate tables).

---

## 5. Emergency fund, term insurance and health insurance sizing

### Takeaway
Term cover: start at 10-15x annual income plus outstanding loans (15-20x with dependents), refined by HLV. Emergency fund (opinion): 6 months of expenses including EMIs, 9-12 for single-income or variable income. I found no good sources for super top-up sizing.

### Cited Findings
- **[OPINION]** Term cover is commonly set at 10-15x annual income plus outstanding loans. 15-20x is suggested for families with children, loans or education goals. — [Policybazaar](https://www.policybazaar.com/term-insurance/articles/how-much-term-insurance-do-i-need/); [PolicyX](https://www.policyx.com/term-insurance/articles/term-insurance-coverage/); [Aditya Birla Sun Life](https://lifeinsurance.adityabirlacapital.com/articles/term-insurance/adequate-budget-for-term-insurance)
- **[OPINION]** HLV (human life value) is the present value of future earnings minus personal consumption. It replaces the lost savings or contribution of the earner. Inputs are income, expenses, loans and age. A claim that IRDAI "recommends" HLV is unverified. — [Ditto](https://www.joinditto.in/articles/life-insurance/maximum-sum-assured-in-term-insurance/); [India CSR](https://indiacsr.in/four-ways-to-determine-the-ideal-term-insurance-coverage-for-you/)
- **[MARKET DATA]** Medical cost trend is 10-11.5% (Aon), so health cover erodes quickly. — [Telangana Today/Aon](https://telanganatoday.com/medical-costs-in-india-projected-to-rise-10-1pc-in-2027-aon)

### Inferences ([MATH] and [OPINION])
- **HLV formula:** `HLV = PV over working years of (income_t − self_consumption_t − taxes_t)` discounted at (r − income growth). Then add loans, add the PV of the dependants' future goals (child education, spouse's retirement shortfall), and subtract existing liquid assets and existing cover. A simpler needs-based version: `Cover = (annual household expenses × years to replace, inflation-adjusted PV) + loans + goals PV − liquid assets − existing cover`.
- **Emergency fund:** `EF = months × (monthly essential expenses + EMIs + insurance premiums)`. Default 6 months; 9-12 for single-income households or variable pay; 3-6 for a dual-income couple with stable jobs. Keep it in a sweep FD or liquid fund across HDFC and Kotak. These figures are unsourced conventions.
- **Health insurance sizing (opinion, unsourced this session):** a metro family floater base of about ₹10L, plus a super top-up of ₹25L-₹1Cr with a deductible equal to the base sum insured. Re-test every 3-5 years against 10% medical inflation. Keep employer group cover separate, because it ends when the job ends.

### Gaps
- No IRDAI or insurer guidance on super top-up deductibles or sizing was found.
- No sourced Indian emergency fund guideline (for example from SEBI investor education) was found.

---

## 6. Tax optimisation, FY2025-26 and FY2026-27

### Takeaway
Budget 2026 did not change slabs. The new regime (default) is nil to ₹4L, then 5/10/15/20/25/30% in ₹4L steps to ₹24L+. The 87A rebate of up to ₹60k gives zero tax up to ₹12L taxable, or ₹12.75L salary after the ₹75k standard deduction. For salaried users, the old regime usually wins only when deductions are about ₹4L+ at high incomes. Equity LTCG stays at 12.5% above ₹1.25L and STCG at 20%. Debt MFs bought after Apr 2023 are taxed at slab rate. Budget 2026 changes: STT on F&O raised, buybacks taxed as capital gains, and the new Income Tax Act 2025 takes effect from 1 Apr 2026.

### Cited Findings
- **[OFFICIAL]** New regime slabs for FY26 and FY27 (unchanged by Budget 2026): 0-4L nil; 4-8L 5%; 8-12L 10%; 12-16L 15%; 16-20L 20%; 20-24L 25%; above 24L 30%. The 87A rebate (up to ₹60,000) makes taxable income up to ₹12L tax-free. The standard deduction is ₹75,000, so salary up to ₹12.75L is tax-free. — [Business Today](https://www.businesstoday.in/personal-finance/tax/story/tax-slabs-fy-2026-27-what-budget-2026-changed-for-individual-taxpayers-and-which-regime-works-best-514044-2026-02-01); [Motilal Oswal](https://www.motilaloswal.com/learning-centre/2026/2/new-vs-old-tax-regime-after-budget-2026-which-one-makes-sense-now)
- **[OFFICIAL]** Old regime for under-60s: 0-2.5L nil; 2.5-5L 5%; 5-10L 20%; above 10L 30%. Senior exemption ₹3L, super-senior ₹5L. Old-regime standard deduction is ₹50,000. — [Business Today](https://www.businesstoday.in/personal-finance/tax/story/tax-slabs-fy-2026-27-what-budget-2026-changed-for-individual-taxpayers-and-which-regime-works-best-514044-2026-02-01); [Basunivesh deductions list](https://www.basunivesh.com/all-tax-deductions-old-vs-new-regime-full-list-fy-2026-27/)
- **[OFFICIAL]** The Income Tax Act 2025 is effective 1 Apr 2026, with the new regime as default (opt out to choose old). — [Bajaj Finserv](https://www.bajajfinserv.in/income-tax-changes-for-fy-2026-27); [BankBazaar](https://mobikwik.bankbazaar.com/tax/income-tax-slabs.html)
- **[OFFICIAL]** Employer NPS contribution under 80CCD(2) (reportedly s.125 of the new Act) is deductible up to 14% of basic + DA in the new regime. For private employees in the old regime it is 10% (sources differ). — [Fincalculator](https://fincalculator.in/blog/new-vs-old-tax-regime-fy2026-27-guide); [Taxguru checklist](https://taxguru.in/income-tax/old-vs-new-tax-regime-fy-2026-27-salaried-employees-checklist.html); [Taxheal](https://www.taxheal.com/list-of-deductions-allowed-in-new-tax-regime-in-ay-2026-27.html)
- **[OPINION]** Break-even: at higher salaries the old regime wins only if HRA + 80C + 80D + home-loan interest exceed about ₹4L. One table gives about ₹2.5L of deductions as break-even at ₹8L gross, though the new regime is nil there anyway. — [Basunivesh](https://www.basunivesh.com/new-tax-regime-vs-old-regime-who-wins-in-fy-2026-27/); [Fincalculator break-even guide](https://fincalculator.in/blog/old-vs-new-tax-regime-breakeven-2026); [calcguru](https://calcguru.in/?p=280)
- **[OFFICIAL]** Capital gains (since 23 Jul 2024, unchanged in Budget 2026): equity/equity MF LTCG (held more than 12 months) is 12.5% on gains above ₹1.25L per year; STCG is 20%. Holding periods are unchanged. — [Angel One](https://www.angelone.in/news/taxation/union-budget-2026-ltcg-tax-unchanged-at-12-5-stt-on-commodity-futures-hiked); [Sahi](https://www.sahi.com/blogs/ltcg-tax-mutual-funds-stocks-budget-2026)
- **[OFFICIAL]** Debt MFs bought on or after 1 Apr 2023 get no LTCG treatment or indexation and are taxed at slab rate. Budget 2026 did not restore indexation. One source wrongly claims a uniform 12.5% for debt. — [Bajaj Finserv](https://www.bajajfinserv.in/what-is-ltcg-tax-rate); [Angel One](https://www.angelone.in/news/taxation/union-budget-2026-ltcg-tax-unchanged-at-12-5-stt-on-commodity-futures-hiked)
- **[OFFICIAL]** Budget 2026 raised STT on futures from 0.02% to 0.05%, and on options to 0.15% (sources differ on the options detail). Buyback proceeds are taxed as capital gains for shareholders from FY27. Equity delivery and MF STT are unchanged. — [Free Press Journal](https://www.freepressjournal.in/business/big-announcement-for-stock-market-traders-stt-hiked-in-budget-2026-fo-tax-raised-to-005-015); [Fyers](https://fyers.in/blog/union-budget-2026-highlights/); [Angel One MF STT](https://www.angelone.in/news/mutual-funds/union-budget-2026-how-change-in-stt-going-to-impact-mutual-funds)

### Inferences
- **New-regime tax engine ([MATH] on the official slabs):** taxable = gross salary − ₹75,000 − 80CCD(2) employer NPS. Apply the slabs. If taxable ≤ ₹12L, tax is 0 (rebate). Just above ₹12L, apply marginal relief so tax ≤ (taxable − ₹12L). Add 4% health and education cess, plus surcharge above ₹50L. Example: ₹20L salary gives taxable ₹19.25L. Slab tax = 20k + 40k + 60k + 65k = ₹1,85,000, so ₹1,92,400 with cess. (My calculation; verify against the official calculator.)
- **Old-regime deductions to model:** 80C ₹1.5L (EPF employee share, PPF, ELSS, home-loan principal, tuition, life premium); 80CCD(1B) NPS extra ₹50k; 80D health premium ₹25k self/family plus ₹25k for parents (₹50k if senior); HRA exemption = minimum of (actual HRA, rent − 10% of basic+DA, 50% of basic for metro or 40% for non-metro); s.24(b) ₹2L. These limits are from background knowledge, not re-verified this session against the 2025 Act text.
- **Regime-picker feature:** compute both regimes from the user's actual deduction data and show the break-even deduction amount: `D* such that Tax_old(gross − 50k − D*) = Tax_new(gross − 75k)`.
- **LTCG harvesting nudge:** each FY, sell and rebuy equity MF units to realise up to ₹1.25L of gains tax-free. This is especially useful for spouses, since each has a separate ₹1.25L exemption.
- **RD/FD:** interest is taxed at slab rate yearly. Show users the post-tax yield against PPF (EEE) and debt MFs.

### Gaps
- I did not verify from CBDT/Income Tax Department primary text the 87A rebate mechanics (one source says the rebate covers up to ₹12.75L), marginal relief, the HRA metro list under the new Act (reports that Bengaluru, Hyderabad, Pune and Ahmedabad were added are unconfirmed), the 80CCD(2) old-regime cap, or the new section numbers.
- Gold ETF/fund LTCG holding period (believed to be 24 months at 12.5%) was not verified this session.

---

## 7. Couples-specific planning

### Takeaway
A joint home loan where both partners are co-owners and co-borrowers lets each claim ₹2L (s.24(b)) and ₹1.5L (80C) under the old regime, which doubles household deductions. Benefits are lost for a partner on the new regime. Shared goals should be funded in proportion to income.

### Cited Findings
- **[OFFICIAL]** Each co-owner who is also a co-borrower can claim up to ₹2L interest under s.24(b) and ₹1.5L principal under 80C, the latter within the overall 80C cap. A co-borrower who is not a co-owner can claim interest but not principal. Claims follow ownership and repayment share. — [Housing.com](https://housing.com/news/tax-benefits-on-joint-home-loan/); [Taxguru](https://taxguru.in/income-tax/tax-benefits-home-loan-joint-owners.html); [Arthgyaan](https://arthgyaan.com/blog/home-loan-tax-benefits-you-get-when-you-buy-a-property-jointly-with-your-spouse.html)
- **[OFFICIAL]** If a co-owner chooses the new regime, 80C, 80EEA and s.24(b) benefits for self-occupied property are not available to them. Interest on let-out property is still allowed. 80EEA applied only to loans sanctioned between Apr 2019 and Mar 2022. — [Housing.com](https://housing.com/news/claim-tax-benefits-joint-home-loans); [BankBazaar blog](https://blog.bankbazaar.com/tax-benefits-from-a-joint-home-loan/)
- **[MARKET DATA]** 79% of couples fund weddings from savings and 15% borrow. — [Outlook Money](https://www.outlookmoney.com/banking/loan/big-fat-weddings-cost-couples-their-hard-earned-savings-only-15-per-cent-use-loans-report)

### Inferences ([OPINION] and [MATH])
- **Income-proportional contributions:** partner A's share = `Income_A / (Income_A + Income_B)` × the joint goal SIP. Also offer 50/50 and custom splits. Use post-tax take-home pay, not CTC.
- **Joint FOIR:** banks combine co-applicant incomes, so combined eligibility rises. The app should still apply the 35-40% comfort rule to combined take-home pay.
- **Regime per person:** each spouse picks a regime separately. The app should optimise jointly. For example, the higher earner stays on the old regime to use home-loan deductions while the other uses the new regime, with ownership split to match.
- **Wedding budget:** many Indian weddings are partly family-funded. Model contributors (self, partner, each family) and line items, and keep the wedding fund in RD, FD or a liquid fund once under 18-24 months away. Avoid equity for wedding money inside 3 years.
- **Separate exemptions:** each spouse gets a separate ₹1.25L LTCG exemption, 80C/80D limits and 87A rebate eligibility, so distributing investments matters. Note that clubbing of income (s.64) applies to money gifted to a spouse. This is from background knowledge, not verified this session.
- **Insurance after marriage:** add the spouse to the family floater, recompute term cover (HLV) now that there is a dependant, and update nominees on HDFC/Kotak accounts, MFs, EPF and NPS.

### Gaps
- No research found on Indian couples' money-pooling norms or recommended splitting conventions.
- Clubbing provisions under the new Act were not verified.
