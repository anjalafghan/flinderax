# Transaction Analytics for "joy" (Indian personal finance app, Rust + SQLite, integer paise)

Scope: what analytics joy can run on bank and credit card transactions (from SMS alerts and CSV statements). For each use case: what it tells the user, approaches from heuristics to ML/LLM, data needed, accuracy pitfalls, and rough solo-developer effort. Effort estimates and design suggestions are my inferences unless they carry a citation.

Effort scale used in Inferences: S = 1-3 days, M = 1-2 weeks, L = 3+ weeks, for one developer already familiar with the joy codebase.

---

## 1. Merchant normalization and categorization of Indian narrations

### Takeaway
A layered pipeline works best for a solo developer: per-bank regex parsers pull out rail, reference and counterparty (VPA, name); a merchant alias table plus fuzzy matching normalizes the counterparty; rules and user corrections assign categories; an LLM or small classifier handles only the leftovers. Industry leaders (Plaid) use large fine-tuned transformer models, but published zero-shot LLM accuracy on terse bank descriptions is only about 60-73%, so rules plus user-correction learning should be the backbone. No public labeled dataset of Indian UPI narrations exists; only a synthetic one does.

### Cited Findings
**Indian narration formats**
- Example formats: NEFT (HDFC style) `NEFT CR:HDFC2268012345678 ABC CORP INV-2024-001`; ICICI style `NEFT-[UTR]-[name]-[ref]`; RTGS `RTGS CR:SBIN2268001234567 XYZ LTD ADVANCE`; UPI merchant `UPI/P2M/123456789012/john@oksbi/ORDER-891`; UPI P2P `UPI/P2P/[ref]/[name]`; IMPS `IMPS/9876543210/RAHUL KUMAR/9876` or `IMPS CR [ref]`; NACH `NACH/BATCH-20260315-001/HDFC0000001`; cheque `CLG/123456/HDFC0001234/20260315` — [Terra Insight: narration patterns India](https://www.terra-insight.com/insights/bank-statement-narration-patterns-india)
- Delimiters and prefixes differ by bank (HDFC uses slashes, ICICI hyphens), and formats can change after a core-banking upgrade, so per-bank parser configs are needed. Track match rates, because a parser built for an old format silently returns zero matches after a migration — [Terra Insight](https://www.terra-insight.com/insights/bank-statement-narration-patterns-india)
- Bank CSV exports often cut narrations to 50-100 characters, which can truncate references and names — [Terra Insight](https://www.terra-insight.com/insights/bank-statement-narration-patterns-india)
- The source's UTR length is self-contradictory: it says 22 characters, but its breakdown (4-char bank code + 2-digit year + 3-digit day + 7-digit sequence) adds up to 16. Validate against real data — [Terra Insight](https://www.terra-insight.com/insights/bank-statement-narration-patterns-india)
- A UPI narration typically holds rail, payment type, reference, remitter VPA or name, and description. The UPI reference is a 12-digit transaction ID, not a UTR — [AI Accountant: smart narration parsing](https://aiaccountant.com/blog/smart-narration-parsing-indian-statements)
- Key tokens to extract: DR/CR markers, UTR (NEFT/RTGS), RRN (IMPS), UPI transaction IDs, IFSC codes, masked account hints, VPAs. One suggested pipeline: pre-clean, tokenize, regex pass (UTR, RRN, IFSC, GSTIN, PAN), ML NER for names and categories, conflict resolver, standardizer, validator — [Terra Insight: narration classification](https://www.terra-insight.com/insights/bank-statement-narration-pattern-classification-india/); [AI Accountant](https://aiaccountant.com/blog/smart-narration-parsing-indian-statements) (vendor content; accuracy claims unverified)
- A hybrid design (rules for known patterns, ML to flag new ones) is recommended. VPA-based rules are robust because `merchant@ybl` identifies a vendor even when the narration text is garbled — [AI Accountant: categorization rules India](https://www.aiaccountant.com/blog/transaction-categorization-rules-india)
- A hobbyist 95-line Python UPI categorizer uses a regex to extract the merchant slug, then maps it to categories by keyword — [DEV Community](https://dev.to/automate-archit/build-a-upi-transaction-categorizer-in-95-lines-of-python-2g8f)

**VPA handle suffixes (identify the PSP app, not the merchant)**
- @ybl = Yes Bank handle, mainly PhonePe; @axl = Axis Bank handle used by PhonePe; @ibl = ICICI (per Paytm FAQ; one blog says IndusInd, which is less reliable); @okaxis, @oksbi, @okhdfcbank, @okicici = Google Pay partner-bank handles; @axisb = BHIM Axis Pay; @paytm = Paytm — [Wikipedia: List of UPI apps](https://en.wikipedia.org/wiki/List_of_UPI_Apps); [Paytm FAQ on @ibl](https://paytm.com/faqs/upi/which-banks-upi-id-is-ibl); [Oxigen: axl](https://www.oxigenwallet.com/upi/axl-in-upi/); [Chargebee: Razorpay UPI autopay handles](https://support.chargebee.com/support/solutions/articles/260014-razorpay-upi-autopay-supported-applications-and-their-handles)
- The suffix identifies the provider, and the prefix is the user's chosen ID — [Paytm FAQ](https://paytm.com/faqs/upi/why-does-upi-address-mean-vary-between-different-upi-apps)

**Industry approaches (Plaid)**
- Plaid's first ML categorization (2017) used word embeddings and neural networks — [Plaid: Making sense of messy data](https://plaid.com/blog/making-sense-of-messy-data/)
- Plaid's "Universal Transaction Categorization" merged four separate ML systems into one. V1 fine-tuned a BERT encoder, and V2 builds on an in-house transaction foundation model, "CLERT" — [Plaid Engineering](https://engineering.plaid.com/universal-transaction-categorization-how-plaid-unified-four-ml-systems-into-one-f08bf1eade54) (search snippet only; fetch returned 403)
- Plaid trains a transaction foundation model on anonymized network data with self-supervised learning, as shared infrastructure across products — [Plaid blog: transaction foundation model](https://plaid.com/blog/building-transaction-foundation-model-intelligent-finance/)
- Plaid's Dec 2025 upgrade reports up to 10% higher accuracy on primary categories and up to 20% on detailed subcategories versus the prior version. Labels came from "AI-assisted label generation" plus "targeted human review". The taxonomy gained a dozen-plus subcategories for income types, repayments, bank fees and transfers (e.g. "Gig Income"), selected via `personal_finance_category_version` — [Plaid: AI-enhanced categorization](https://plaid.com/blog/ai-enhanced-transaction-categorization/)
- Plaid has published a two-level personal finance category taxonomy, a reusable reference for a category list — [Plaid: categorization taxonomy](https://plaid.com/blog/transactions-categorization-taxonomy/)

**LLM and small-model accuracy evidence**
- On SME bank transactions, zero-shot GPT-4o reached 60.4% accuracy, 13 points below the authors' fine-tuned approach (~73%). The authors blame abbreviated, context-poor descriptions — [arXiv 2508.05425](https://arxiv.org/pdf/2508.05425)
- A vendor study (Digits) found no general-purpose LLM above 73% accuracy against its specialized system and a human accountant baseline — [Digits report](https://digits.com/downloads/beyond-the-hype-evaluating-llms-vs-digits-agl.pdf) (vendor; methodology needs scrutiny)
- An unpublished project reports ~92% for few-shot Llama 3.1 8B — [transaction-ai benchmarks](https://transaction-ai.readthedocs.io/en/latest/BENCHMARKS/) (not peer reviewed)
- Parafin argues that a two-agent LLM setup beats a single agent on financial classification — [Parafin blog](https://www.parafin.com/blog/why-two-llm-agents-beat-one-on-financial-classification)
- A DistilBERT US transaction classifier is published on Hugging Face (US-trained; it would need Indian fine-tuning) — [Hugging Face: DoDataThings](https://huggingface.co/DoDataThings/distilbert-us-transaction-classifier-v2/blob/main/README.md)

**Datasets**
- AgamiAI's Indian Bank Statement Synthetic Dataset (PDF + JSON) targets categorization and NER. It is meant for learning structure and format, not behaviour, and needs real-data validation — [Hugging Face: AgamiAI](https://huggingface.co/datasets/AgamiAI/Indian-Bank-Statements/blob/main/README.md)
- Setu argues that payment rail and method are themselves behavioural signals ("How you pay is who you are") — [Setu blog](https://setu.co/blog/how-you-pay-is-who-you-are)

### Inferences
- **What it tells the user:** where money goes (category totals), plus clean merchant names. Every later analysis (recurring, anomalies, forecasts) depends on this foundation.
- **Recommended joy pipeline (Rust):**
  1. *Parse (S-M):* per-source regex parsers (the `regex` crate) for each bank's SMS template and CSV narration. Output structured fields: `rail` (UPI/NEFT/IMPS/RTGS/NACH/POS/ATM/CARD/CHG/INT), `direction`, `ref_id` (UTR/RRN/UPI ref), `counterparty_vpa`, `counterparty_name`, `remark`. Store the raw narration so the parser can be re-run later.
  2. *Normalize (S):* lowercase, strip reference digits, city and country suffixes, and payment-aggregator prefixes on card descriptors (e.g. "RAZ*", "PAYU*", "CCAVENUE"; I could not verify a list, but this mirrors US "SQ *"/"TST*" practice). Use the VPA local part as the merchant key for UPI P2M (e.g. `swiggy.stores@axl` gives `swiggy`). The handle suffix is only the PSP and should not be the merchant.
  3. *Merchant alias table (S):* `merchant_alias(pattern, merchant_id)` in SQLite. Seed it with about 200 top Indian merchants (Swiggy, Zomato, Amazon, Flipkart, Uber, Ola, BigBasket, Blinkit, Zepto, IRCTC, Jio, Airtel, BESCOM/electricity boards, Netflix, Hotstar, LIC, mutual fund AMCs via BSE/NSE/CAMS/KFin NACH). Match unseen strings with fuzzy matching (Jaro-Winkler or trigram via the `strsim` crate) above a threshold.
  4. *Categorize (S):* priority order: user rule > merchant default category > keyword rules > model. Put P2P UPI to a person's name in "Transfers/People" unless the user tags it. This is a big India-specific bucket (rent, domestic help, splitting bills).
  5. *User-correction learning (S):* when a user recategorizes, upsert a rule keyed on the normalized merchant (and optionally an amount band), and offer "apply to all past". This alone probably drives accuracy for one user's data far above any generic model, since one person has a few hundred distinct merchants.
  6. *LLM fallback (M):* batch the uncategorized normalized merchants (not raw narrations, which contain names and account fragments) and ask an LLM for a category from a fixed enum via structured output. Cache the result in the alias table so each merchant is classified once. Given the 60-73% zero-shot evidence, show LLM picks as "suggested" for confirmation.
  7. *Embeddings/kNN (optional, M-L):* small sentence-embedding models (via ONNX runtime in Rust, or `fastembed-rs`) plus nearest-neighbour over the user's labeled merchants. Mostly useful at multi-user scale; for one user it adds little over the alias table.
- **Pitfalls:** truncated narrations; P2P versus P2M ambiguity (small merchants take payments on personal VPAs); aggregator VPAs (Razorpay/PayU/Cashfree) hiding the real merchant; self-transfers between one's own accounts and credit card bill payments, which will double count spend if not marked as transfers; refunds and reversals that must net against the original; and SMS and CSV both importing the same transaction (dedupe on ref_id, then on amount+date+account).
- **Effort:** parsing plus rules plus correction learning is M overall. LLM fallback is S-M. A trained classifier is L and probably not worth it for single-user data.

### Gaps
- No public, labeled, real Indian UPI/NEFT narration dataset was found, only the synthetic AgamiAI set.
- No published accuracy numbers for Indian narration categorization by Indian fintechs (CRED, Jupiter, Fi, Razorpay). I found no engineering blog posts from them on this topic in the time available.
- Card descriptor conventions for Indian payment aggregators (Razorpay/PayU/CCAvenue prefixes on card statements) are unverified.
- No authoritative NPCI list of VPA handles was opened.

---

## 2. Recurring and subscription detection

### Takeaway
Standard practice: group by normalized merchant, test inter-arrival gaps against canonical periods (weekly, biweekly, monthly, quarterly, yearly) within a tolerance, and require amount consistency within a percentage band. Add a known-subscriptions list that bypasses the statistics, and track status (active, late, stopped) against the expected next date. Price hikes appear as a step change in amount within an otherwise stable series. In India, NACH/e-mandate and UPI AutoPay narrations are strong explicit recurrence signals.

### Cited Findings
- Patents describe flagging a series as recurring when all date differences fit a daily, weekly, bi-weekly, tri-weekly or monthly trend. One approach fits a linear function to the timestamps and measures their variance from it — [USPTO 10902365: Identifying recurring series from transactional data](https://image-ppubs.uspto.gov/dirsearch-public/print/downloadPdf/10902365)
- An older patent identifies recurrence when consecutive gaps match one of a set of predetermined periods within a preset tolerance — [US5918216A](https://insight.rpxcorp.com/patent/US5918216A)
- Open-source subscription-detector algorithm: optional user grouping rules that rename varying merchant strings; 70+ built-in known-subscription patterns that match on a single occurrence and skip tolerance checks; otherwise group by payee (case-insensitive), require at least 2 occurrences, expenses only, complete months only (the current month is excluded to avoid false "missed" flags); at most one payment per calendar month, with gaps allowed; consecutive amounts within a 35% default tolerance (configurable, e.g. 10% strict or 50% relaxed); status ACTIVE if paid this month or within a 5-day grace period after the typical day-of-month, STOPPED if the last payment was 2+ months ago or the grace period passed — [subscription-detector algorithm](https://gigurra.github.io/subscription-detector/algorithm)
- Spade defines recurrence as spend on a consistent basis to a specific merchant, combining merchant data with the user's history — [Spade recurring transaction guide](https://docs.spade.com/reference/recurring-transaction-guide)
- Rocket Money imports the last 90 days from each linked account by default (via Plaid), tracks recurring charges over time and alerts on changes such as new charges. Missed items can be added manually (mobile app only) — [Rocket Money help: bills and subscriptions](https://help.rocketmoney.com/en/articles/2185531-managing-your-bills-and-subscriptions)
- A secondhand write-up says Rocket Money looks for similar amount, regular cadence and same merchant — [Northville Tech blog](https://northvilletech-site-2026.pages.dev/blog/how-does-rocket-money-find-subscriptions) (unverified, low-quality source)
- Copilot Money tracks recurring expenses but does not cancel subscriptions or negotiate bills — [Rocket Money vs Copilot](https://www.rocketmoney.com/learn/personal-finance/rocket-money-vs-copilot) (competitor marketing)
- Subaio publishes a description of its recurring-payment detection — [Subaio](https://subaio.com/?p=12719) (not fetched)

### Inferences
- **What it tells the user:** a list of subscriptions and bills with expected next date and amount; total monthly committed spend; "forgotten" subscriptions (low-engagement services still charging); price-hike alerts; missed or late bills. This feeds joy's existing bills and safe-to-spend features directly.
- **Algorithm for joy (S-M):**
  1. Group debits by `merchant_id` (after normalization), separately per account and card.
  2. Sort by date and compute gaps in days. For each candidate period P in {7, 14, 30.4, 91, 182, 365}, score = share of gaps within ±tolerance (e.g. ±3 days for monthly, ±1 for weekly, ±10 for yearly). Use the median gap for robustness, and accept when the score is at least 0.75 with at least 3 occurrences (2 for known merchants, or for yearly).
  3. Amount stability: coefficient of variation below about 0.15, or every amount within ±X% of the median. Separate "fixed" (Netflix) from "variable recurring" (electricity, mobile postpaid), which pass the period test but not the amount test. Both are useful.
  4. Explicit signals: NACH/ACH debits, UPI AutoPay mandate debits, SI (standing instruction) on cards, and EMI narrations mark recurrence immediately.
  5. Next expected date = last date + period, snapped to the typical day-of-month for monthly series. Status: upcoming, due, late (past expected + grace), stopped (missed 2 cycles).
  6. Price hike: the latest amount exceeds the trailing median by more than max(5%, ₹10) while the period is still stable. Alert once per change.
  7. Forgotten subscriptions: there is no usage data from bank feeds, so use proxies: yearly renewals arriving soon (warn 7 days ahead), free-trial conversion (a first charge after a ₹1 or ₹2 verification charge from the same merchant), and duplicate services in one category (two music or three OTT subscriptions).
- **Pitfalls:** month-length jitter (28-31 days) and weekend/holiday shifts; merchants that bill across several descriptors; the same subscription moving from card to UPI; annual plans with too few data points; groceries or fuel at a regular cadence falsely flagged (the amount-CV filter helps); salary credits are recurring inflows and should get the same treatment for income detection; 90-day lookbacks miss yearly subscriptions (use the full history).
- **Effort:** heuristic detector S-M; status tracking, alerts and UI M. ML scoring is unnecessary at single-user scale.

### Gaps
- Rocket Money and Copilot have no first-party technical write-ups of their detection algorithms; descriptions are secondhand.
- No academic benchmark of recurring-detection precision and recall was found.
- Indian UPI AutoPay and e-mandate narration formats across banks were not verified.

---

## 3. Anomaly detection (unusual spend, duplicates, fraud, fees/interest, failed-transaction refunds)

### Takeaway
For one user, rule-based and robust-statistics detectors (median/MAD z-scores per category or merchant, duplicate windows, keyword fee detection, refund-match timers) are more interpretable and practical than ML. RBI's failed-transaction turnaround-time (TAT) framework gives exact deadlines joy can track: UPI P2P failures must be auto-reversed by T+1 and merchant UPI failures by T+5, with ₹100/day compensation after that.

### Cited Findings
- RBI circular RBI/2019-20/67 (20 Sep 2019, effective 15 Oct 2019) harmonises turnaround time and customer compensation for failed transactions. UPI P2P (account debited, beneficiary not credited): auto-reversal by T+1, ₹100/day after T+1. UPI merchant payment (debited, merchant got no confirmation): auto-reversal within T+5, ₹100/day after T+5. T = day of transaction. The TAT is the "outer limit", and entities should aim for quicker resolution — [RBI notification 11693](https://www.rbi.org.in/Scripts/NotificationUser.aspx?Id=11693); [RBI common person page](https://rbi.org.in/CommonPerson/english/Scripts/Notification.aspx?Id=3074)
- Secondary sources conflict on calendar day versus working day compensation, and on whether the payout is automatic or must be claimed — [Zee News](https://embed2.zeenews.com/personal-finance/failed-upi-transactions-banks-to-pay-rs-100-per-day-penalty-for-delay-beyond-2-days-2352699.html); [Paytm blog](https://paytm.com/blog/payments/upi/bank-responsibility-for-failed-upi-transaction); [Ujjivan SFB blog](https://www.ujjivansfb.bank.in/banking-blogs/banking-services/upi-payment-failed-refund-process-and-rules)
- When the UTR is missing or truncated, amount-and-date matching can produce false positives, so add duplicate-detection rules — [Terra Insight](https://www.terra-insight.com/insights/bank-statement-narration-patterns-india)
- Plaid publishes how it reconciles pending and posted transactions, a key source of apparent duplicates — [Plaid Engineering: pending vs posted](https://engineering.plaid.com/how-plaid-reconciles-pending-and-posted-transactions-1301b09d1749) (not fetched)
- RBI: penal interest, late fees and related charges may be levied only on the outstanding amount after the due date, not on the total amount. Unpaid charges, levies and taxes may not be capitalised for interest — [Outlook Business coverage of RBI directions](https://www.outlookbusiness.com/news/rbi-s-new-guidelines-on-credit-cards-what-it-means-for-cardholders-know-details-here-news-192736); [RBI Master Direction](https://www.rbi.org.in/Scripts/BS_ViewMasterDirections.aspx?id=12300)

### Inferences
- **What it tells the user:** "you were charged twice", "this card charge is unusual for you", "you paid ₹X in bank fees, charges and interest this year", "your failed UPI payment of ₹Y has not been refunded, and the bank owes you ₹100/day from date D".
- **Detectors (each S unless noted):**
  - *Duplicate charge:* same merchant, same amount, same account, within N minutes or hours (e.g. 10 minutes for UPI/POS, 48 hours for card posting), with different ref_ids. Treat SMS+CSV pairs with the same ref_id as import duplicates, not charge duplicates.
  - *Failed-transaction refund tracker (M):* parse SMS like "debited ... failed" or "reversed/refund". When a debit has no matching success (or the user marks it failed), open a case with deadline T+1 (P2P) or T+5 (P2M). Match the credit by ref_id or by amount within ±0 paise in the window. Past the deadline, show the days overdue × ₹100 and the escalation path (bank, then the RBI Ombudsman via CMS). This is a strong India-specific feature.
  - *Merchant refund tracker:* same idea for e-commerce returns, using user-set expected windows.
  - *Unusual spend:* per category and per merchant robust z-score = (amount - median) / (1.4826 × MAD) over the trailing 6-12 months. Flag above about 3.5. Also flag first-ever merchants above a threshold, and category month-to-date pace against the trailing median.
  - *Fraud signals (heuristic only):* international or forex transactions when the user has none historically; card-not-present charges at odd hours; many small charges in quick succession (card testing); new merchant plus high amount; SMS OTP/"declined" bursts. Present these as "review this", never as "fraud".
  - *Fees and interest:* keyword rules on narrations ("CHRG", "CHG", "FEE", "GST", "SMS CHGS", "ANNUAL FEE", "LATE PAYMENT", "FIN CHG", "INTEREST", "MIN BAL", "DECLINE CHG", "FOREX MARKUP", "CASH ADV"), aggregated yearly. GST at 18% on bank fees is commonly charged in India (unverified here). On cards, check interest or late fees against the RBI rule that they apply only to the post-due-date outstanding.
  - *Isolation Forest or ML:* not worth it for single-user data. Robust stats are explainable.
- **Pitfalls:** legitimate repeat purchases (two coffees) look like duplicates, so require the same amount to the paisa plus a short window; pre-auth holds (fuel, hotels) followed by the final charge; the pending versus posted gap between SMS and statement; category baselines too thin early on (require at least 8-10 points before alerting); alert fatigue (cap weekly alerts, rank by ₹ impact).
- **Effort:** duplicate + fees + unusual spend S-M together; refund tracker M (SMS template parsing for failure and reversal messages is the main work).

### Gaps
- Whether the ₹100/day compensation is calendar or working day, and whether it is automatic, after any post-2021 amendments, is unverified.
- Exact narration strings for fees and interest across Indian banks were not collected from primary sources.
- No published evaluation of consumer-side fraud heuristics was found.

---

## 4. Cash-flow forecasting and low-balance warnings

### Takeaway
The most practical and explainable forecast is deterministic plus stochastic. Project known recurring inflows and outflows (salary, rent, SIPs, EMIs, subscriptions, card bill due amounts) on a calendar. Add discretionary spend as a daily burn rate drawn from the user's history, with an empirical band (e.g. bootstrap of past same-period spends) for P10/P50/P90. Then warn when the P10 path crosses a threshold before payday. Heavier ML (Prophet, gradient boosting with quantile loss) adds little for one user's data.

### Cited Findings
- A 2025 Barekeng study on banking cash flows compared residual bootstrap with quantile regression for prediction intervals on XGBoost/CatBoost/LightGBM. Residual bootstrap came closer to nominal 80% coverage with narrower intervals. Quantile regression gave wider intervals and tended to overstate uncertainty — [DOAJ: Barekeng Jul 2025](https://doaj.org/article/2a23d2ba21ff47288f8c3e459acebe21); [full text](https://ojs3.unpatti.ac.id/index.php/barekeng/article/download/16433/10817)
- Quantile regression can estimate upper percentiles of the cash distribution, but it is computationally heavy and data-hungry — [Kyriba blog](https://kyriba.com/blog/data-the-cfos-secret-weapon-for-cash-forecasting/)
- A consumer-oriented guide describes projecting balances from expected inflows and outflows, with irregular spending modeled by Prophet or LightGBM, and recommends showing confidence intervals rather than point estimates — [Freenance](https://freenance.io/financial-tools/ai-cashflow-forecast-2026-europe-3-6-12-month-runway-predictions-personal-finance-tools-comparison/) (vendor blog, unverified)
- Plaid Income v2 describes an engine for identifying income streams from transactions — [Plaid: new engine behind Plaid Income](https://plaid.com/blog/meet-the-new-engine-behind-plaid-income/) (not fetched)
- An FCA field experiment (Adams, Grubb, Kelly, Nieboer, Osborne, 2018) tested automatic enrollment into just-in-time and early-warning alerts against overdraft charges — [FCA Occasional Paper 40](https://www.fca.org.uk/publication/occasional-papers/occasional-paper-40.pdf) (not fetched; the effect sizes are not confirmed here)

### Inferences
- **What it tells the user:** "you'll have about ₹X (likely ₹A-₹B) on the day before salary", "your balance may dip below ₹5,000 around the 24th because the HDFC card bill (₹Y) and rent hit together", and "safe to spend per day until payday". This extends joy's existing safe-to-spend.
- **Method for joy (M):**
  1. Event calendar from the recurring detector (section 2) plus card cycles (statement total due on due date) plus goals (planned transfers) plus bills.
  2. Discretionary residual: daily non-recurring outflow history. Forecast = sum over future days of a sampled day-of-week/day-of-month-adjusted spend. Run about 1,000 bootstrap paths in Rust (fast) and take P10/P50/P90 balance per day. This is the residual-bootstrap idea, which the cited study found better calibrated than quantile regression.
  3. Seasonality: Indian-specific spikes (Diwali/festive season Oct-Nov, wedding season, school fees in Apr-Jun, insurance premiums, annual fees, advance tax dates 15 Jun/Sep/Dec/Mar). Model these as user-confirmed annual events rather than learning them statistically from 1-2 years of data.
  4. Low-balance warning: alert when P10 < min(user threshold, minimum balance requirement) before the next income date. Lead time of at least 3-5 days. Show the driving events.
  5. Backtest: each night, store yesterday's forecast and later compare to actual balance, tracking MAPE and band coverage (aim for ~80% of actuals inside P10-P90). Show "forecast accuracy" to build trust.
- **Data needed:** actual account balances (SMS alerts often include "Avl Bal"; use them to anchor and correct drift), a full recurring list, card statement amounts and due dates, and income dates.
- **Pitfalls:** missing accounts (cash, wallets, a second bank) make the forecast wrong without warning, so show coverage; ignoring transfers between own accounts causes double counting; salary date shifting at month-end and on holidays; credit-card spend affects bank balance only on the bill due date, not the swipe date, which joy's card cycles already model; bonus or irregular income should be excluded from the base forecast.
- **Effort:** deterministic projection S (much may exist already); bootstrap bands M; backtesting S.

### Gaps
- No production engineering write-ups from consumer fintechs (Copilot, Monarch, Indian apps) on balance forecasting method or accuracy were found.
- FCA alert-experiment effect sizes were not extracted.

---

## 5. Income volatility, emergency-fund adequacy, savings rate, spending trends, lifestyle creep

### Takeaway
Simple, well-defined metrics computed monthly in SQL are enough: income CV, months-of-expenses buffer, savings rate, category trend slopes, and spend growth against income growth (lifestyle creep). The JPMorgan Chase Institute found that about six weeks of take-home pay covers a simultaneous income dip and expense spike. That is a data-driven alternative to the 3-6 months rule of thumb, which many find unreachable.

### Cited Findings
- JPMorgan Chase Institute "Weathering Volatility 2.0" (Oct 2019), based on 6 million checking accounts over about 6 years, estimated buffers needed in weeks of take-home income: income dip alone ~2.8 weeks; expenditure spike alone ~2.6 weeks; simultaneous dip and spike (about once every 5.5 years) ~6.2 weeks. 65% of families lacked a buffer that size. The median family saw a 36% month-to-month change in income during the year. The Institute notes that the common 3-6 months guidance may feel unattainable — [JPMC Institute report](https://www.jpmorganchase.com/institute/all-topics/household-financial-health/report-weathering-volatility-2-a-monthly-stress-test-to-guide-saving); [full PDF](https://jpmorganchase.com/content/dam/jpmc/jpmorgan-chase-and-co/institute/pdf/institute-volatility-cash-buffer-report.pdf?os=a0)
- An earlier JPMC Institute insight proposed a monthly financial stress test — [JPMC Institute 2017 insight](https://www.jpmorganchase.com/content/dam/jpmc/jpmorgan-chase-and-co/institute/pdf/institute-insight-financial-stress-test.pdf)

### Inferences
- **Metrics (each S, mostly SQL views):**
  - *Income stability:* monthly income series (salary + other credits, excluding transfers and refunds). Coefficient of variation = stdev/mean over 12 months. Max drawdown month versus median. Share of income from the main source. Label: stable (CV < 0.1), moderate, volatile (> 0.3). These thresholds are my suggestion.
  - *Emergency-fund adequacy:* liquid balances (savings + sweep FD + liquid funds the user tags) divided by median monthly essential expenses (rent, utilities, groceries, EMIs, insurance) gives months of coverage. Show it against two targets: the JPMC-style stress buffer (~6 weeks of take-home pay) and the user's goal (3-6 months, higher for volatile income). Link to joy's goals feature.
  - *Savings rate:* (income − spending) / income per month, where SIPs, RD and PPF transfers count as saving, not spending. Treat card bill payments as transfers, with the spend counted on the card.
  - *Spending trends:* per category, a 3-month rolling mean against the same period last year; a Theil-Sen slope (robust) over 12 months; share-of-wallet changes.
  - *Lifestyle creep:* compare growth rates. If discretionary spend grew faster than income over 12-24 months (e.g. income +10% and dining/shopping +25%), flag it. Also flag a post-raise "new baseline": a step-change detection (CUSUM or a simple before/after mean test) on discretionary spend after a salary increase.
- **Pitfalls:** inflows that are not income (refunds, reimbursements, transfers from family, loans) inflate income; one-off large expenses distort means (use medians); incomplete account coverage; essential versus discretionary needs user confirmation per category.
- **Effort:** S-M total. Mostly SQL plus UI cards.

### Gaps
- No India-specific study of household income volatility or cash buffers comparable to the JPMC Institute work was found.
- The JPMC figures reflect US 2013-2018 data and may not transfer to Indian salaried or gig users.

---

## 6. Debt payoff, credit card optimization, credit score drivers

### Takeaway
Avalanche (highest rate first) is mathematically optimal, but evidence (Gal & McShane, JMR 2012) shows that closing individual accounts, which the snowball method encourages, predicts eventually eliminating debt. Offer both with a cost comparison. For Indian cards, RBI rules make revolving very costly: if the total due isn't paid in full, the interest-free period is lost and interest runs from the transaction date, so the top insight is "always pay the total due". Utilization and payment history are the dominant CIBIL drivers, but the weights are not officially published.

### Cited Findings
- Gal & McShane (Kellogg, Journal of Marketing Research 2012), using about 6,000 people in a debt-settlement program: consumers who paid off small balances first were more likely to eliminate all debt, even at a financial cost. Closing accounts, regardless of dollar balance, predicted successful debt elimination — [Kellogg News](https://www.kellogg.northwestern.edu/news_articles/2012/snowball-approach.aspx); [Kellogg blog](https://www.kellogg.northwestern.edu/news/blog/2012/08/07/the-snowball-approach-to-debt/)
- Some secondary sites misattribute this research (e.g. "Gal and Rick 2018") or cite unverifiable follow-ups — [helpertools blog](https://helpertools.pages.dev/blog/snowball-vs-avalanche-debt-payoff/) (low quality; treat with caution)
- RBI Master Direction (Credit Card and Debit Card – Issuance and Conduct, 2022, amended 2024): the interest-free credit period runs from the transaction date to the payment due date, available only if the full outstanding is paid by the due date. MITC must state that the interest-free period is suspended if any balance from the previous bill is outstanding — [RBI Master Direction](https://www.rbi.org.in/Scripts/BS_ViewMasterDirections.aspx?id=12300); [RBI amendment circular](https://website.rbi.org.in/documents/87730/39710850/1.+Circular+-++Amendment+to+MD.pdf); [Outlook Business](https://www.outlookbusiness.com/news/new-credit-card-rule-of-minimum-balance-will-help-you-from-falling-into-a-debt-trap-news-241671)
- If the total amount due is not cleared by the due date, the interest-free period is lost and interest may be levied from the transaction date on the outstanding amount — [Outlook Business](https://www.outlookbusiness.com/news/rbi-s-new-guidelines-on-credit-cards-what-it-means-for-cardholders-know-details-here-news-192736)
- Minimum-due terms must ensure no negative amortization, and unpaid charges, levies and taxes may not be capitalised for interest (effective 1 Oct 2022) — [Outlook Business](https://www.outlookbusiness.com/news/rbi-s-new-guidelines-on-credit-cards-what-it-means-for-cardholders-know-details-here-news-192736)
- A reported minimum-amount-due formula is the higher of (100% of interest, fees and taxes) or (5% of total due), plus past-due amounts. Other coverage says "usually 5%". Not confirmed against RBI text — [EMI Calculator](https://emicalculator.net/?p=6976); [OnePercentClub](https://news.onepercentclub.io/borrow/rbi-clarifies-what-happens-when-you-make-partial-credit-card-payment/3420/)
- RBI's FAQ on the Master Direction covers billing cycle changes. Cardholders may change the billing cycle at least once — [RBI FAQ PDF](https://website.rbi.org.in/documents/87730/71798953/FAQonMasterDirectionCreditCardandDebitCardIssuanceandConductDirections2022.pdf/8c1e1e27-0064-a371-254c-56d0646a7171); [Business Today](https://www.businesstoday.in/personal-finance/news/story/credit-card-payment-rule-how-many-times-can-you-change-billing-cycle-due-date-heres-what-rbi-says-422647-2024-03-23)
- CIBIL factor weights are not officially disclosed. Common estimates put payment history at about 30-35% and utilization at about 25-30%, with the rest on credit mix, duration and enquiries — [BankBazaar](https://bankbazaar.com/cibil/cibil-score-calculation.html); [Paisabazaar](https://www.paisabazaar.com/credit-score/cibil-score-calculation/); [Fibe](https://www.fibe.in/blogs/how-cibil-score-is-calculated/); [Axis Bank blog](https://www.axis.bank.in/blogs/credit-score/how-is-cibil-score-calculated)

### Inferences
- **Debt payoff planner (M):** inputs: each debt's balance, APR, minimum payment (card: from statement; loans: EMI) and a monthly budget for extra payments. Simulate month by month in integer paise (joy already uses paise; use i64 or i128 for interest accumulation and round per period consistently). Output total interest, payoff date, and number of accounts closed by month for avalanche, snowball, and a "hybrid" (snowball for balances under some threshold, then avalanche). Show the interest cost of choosing snowball, framed with the Gal & McShane finding.
- **Credit card insights (S-M each):**
  - *Utilization:* per card and overall balance/limit at statement date, since that is what is typically reported to bureaus (my understanding, not verified here). Nudge to pay down before the statement date if above about 30%. The 30% band is a common rule of thumb, not an official CIBIL threshold.
  - *Interest-free period optimizer:* for each card, days of free credit if a purchase is made today = (next statement date − today) + (due date − statement date). For big purchases, recommend the card with the longest current free window. This uses joy's existing card-cycle data directly.
  - *Revolve warning:* if a payment below the total due is detected, estimate interest charged from transaction dates (APR around 36-48% p.a. is common for Indian cards; unverified here, so use the user's card APR) and the loss of the interest-free period on new purchases.
  - *EMI versus revolve versus pay-in-full:* compare the total cost of card EMI conversion (interest + processing fee + GST on interest and fee) against revolving and against paying from savings, with an IRR or effective-annual-rate calculation. "No-cost EMI" often hides a processing fee or a lost discount. Make it a calculator.
  - *Best card per merchant/category:* needs a user-entered rewards table (card × category gives reward rate, caps, exclusions such as fuel, rent, wallet loads, government payments). Then compute "you'd have earned ₹X more last quarter using card B for dining". Rewards rules change often, so keep it user-maintained or a small curated JSON. L to maintain a curated database; S for a user-entered version.
  - *Fees:* annual fee waiver tracking (spend threshold progress within the card year), forex markup totals, and late fee and interest totals.
- **Credit score drivers (S):** joy can't see the score, but can show controllable drivers it observes: on-time payment streak for cards and EMIs, utilization per card and overall, and number of new cards or loans. It must say clearly that weights are estimates.
- **Pitfalls:** misidentifying card bill payments in bank statements (needed to mark transfers); multiple cards from one issuer sharing a limit; statement versus current balance; EMI-converted amounts that stay in the limit; APR and fee data the user must enter.

### Gaps
- Official CIBIL/Experian/CRIF India factor weights are not published.
- Exact current minimum-amount-due formulas per issuer and typical Indian card APRs were not verified from primary sources.
- I found no data on what Indian credit bureaus actually use as the reporting date for utilization.
- No Indian rewards-rule open dataset was found.

---

## 7. "Spending personality", behavioural insights and evidence on nudges

### Takeaway
Be modest. At-scale nudges work, but with small effects: the DellaVigna & Linos meta-study found nudge-unit trials averaged about 1.4 percentage points (8.1%) against about 8.7 pp (33.5%) in academic papers, mostly due to publication bias. Reminders reliably raise savings a little (about 6% in Karlan et al.). Defaults and automatic commitment (auto-transfers to goals) are the strongest levers. Personality labels are engagement features, not evidence-based interventions.

### Cited Findings
- DellaVigna & Linos, "RCTs to Scale" (NBER WP 27594; Econometrica 2022, 90(1):81-116): 126 trials (final version: 241 nudges) affecting over 23 million people. Nudge-unit average effect 1.4 pp (8.1%), against 8.7 pp (33.5%) in academic-journal nudges. About 70% of the gap is explained by selective publication worsened by low power. Median academic arm about 484 participants, against about 10,006 in nudge-unit trials — [NBER WP 27594](https://www.nber.org/system/files/working_papers/w27594/w27594.pdf); [Econometrica](https://onlinelibrary-wiley-com-443.webvpn.zafu.edu.cn/doi/10.3982/ECTA18709); [Berkeley draft](https://eml.berkeley.edu/~sdellavi/wp/NudgeToScale2021-04-19.pdf)
- Karlan, McConnell, Mullainathan & Zinman, "Getting to the Top of Mind": bank field experiments in Bolivia, Peru and the Philippines found monthly reminders raised savings by 6% and made people 3 pp more likely to hit their savings goal. Effects concentrated among time-inconsistent savers. SMS reminders have near-zero marginal cost — [J-PAL paper](https://www.povertyactionlab.org/media/file-research-paper/getting-top-mind-how-reminders-increase-saving); [NBER summary](https://img.nber.org/conferences/2010/SI2010/SAV/Executive%20Summary%20--%20Getting%20to%20the%20Top%20of%20Mind%20-.pdf)
- A Colombian youth study found reminder effects on savings persisted 8 months after the messages stopped. A Dominican Republic study found SMS did not increase accounts and reduced transactions, so context matters — [IPA PDF](https://poverty-action.org/node/32641/pdf); [IDEAS/RePEc: Personalized reminders Colombia](https://ideas.repec.org/p/bdp/dpaper/0073.html); [IPA](https://poverty-action.org/node/6366/pdf)
- The FCA tested auto-enrollment into overdraft alerts (just-in-time and early warnings) in a field experiment — [FCA OP 40](https://www.fca.org.uk/publication/occasional-papers/occasional-paper-40.pdf)
- Setu frames payment method as revealing behaviour ("How you pay is who you are") — [Setu](https://setu.co/blog/how-you-pay-is-who-you-are)

### Inferences
- **Evidence-informed nudges for joy (S each):**
  - *Timely reminders* before bill and card due dates (prevent late fees; evidence supports reminders).
  - *Defaults/automation:* suggest a default auto-transfer to goals on salary day ("pay yourself first"). Default effects are widely documented (Save More Tomorrow, auto-enrolment), though that literature was not retrieved here, so cite cautiously.
  - *Just-in-time low-balance alerts* (section 4), in line with the FCA's alert work.
  - *Salience of fees and interest:* a yearly "you paid ₹X in charges" card.
  - *Goal progress framing* and small wins (borrowing the account-closing insight from Gal & McShane for debts).
- **"Spending personality" (S-M):** rule-based archetypes from features: weekend/late-night share, small-ticket UPI frequency (quick-commerce and food delivery), impulse proxy (count of new merchants per month), subscription load, card versus UPI split, payday spending spike (share of month's discretionary spend in the 5 days after salary, a known "payday effect"). Keep labels descriptive and non-judgmental, and tie each to one actionable suggestion. Clustering (k-means on these features) only makes sense with many users; for one user, compare against their own past.
- **Pitfalls:** alert fatigue drowns out the small, real effects; moralizing labels reduce engagement; claims of big behaviour change are not supported by at-scale evidence (expect single-digit percentage effects); measure joy's own nudges with simple before/after or A/B once there are multiple users.

### Gaps
- Save More Tomorrow (Thaler & Benartzi) results and default-effect literature were not retrieved in this session.
- No India-specific RCTs on fintech nudges (e.g. from CRED, Jupiter, Fi) were found.
- CFPB research on financial well-being and alerts was not retrieved.
- FCA alert effect sizes were not extracted.

---

## 8. LLM-assisted insights: Q&A over transactions, monthly narratives, grounding

### Takeaway
Never let the LLM do arithmetic or invent figures. Use tool or function calling to a small set of vetted, parameterized Rust query functions (spend_by_category(period), top_merchants, compare_periods, recurring_list, forecast) that return exact numbers. The LLM only chooses tools and writes prose around returned values. Then verify that every number in the output appears in the tool results. Free-form text-to-SQL is possible against a read-only SQLite view, but is less reliable.

### Cited Findings
- The FinAI Data Assistant paper routes user requests to a small set of vetted, parameterized queries through OpenAI function calling instead of generating full SQL. Function calling "constrains the model to produce structured arguments for pre-registered capabilities", enabling determinism and safer integration. Generic text-to-SQL pipelines "often emit verbose or invalid queries" and still need extensive guardrails — [arXiv 2510.14162](https://arxiv.org/html/2510.14162v1)
- Text-to-SQL models are prone to inventing tables and fields and writing invalid SQL — [Medium: LLMs and SQL](https://medium.com/@fleszarjacek/llms-and-sql-7cc74738424a)
- FinSQL proposes a model-agnostic LLM text-to-SQL framework for financial analysis — [arXiv 2401.10506](https://arxiv.org/pdf/2401.10506); FinStat2SQL a pipeline for financial statements — [arXiv 2506.23273](https://arxiv.org/pdf/2506.23273)
- FinGround grounds financial answers in three stages: retrieve evidence, decompose the answer into atomic claims and verify each, then regenerate unsupported claims with citations — [arXiv 2604.23588](https://arxiv.org/pdf/2604.23588)
- No sources gave hallucination rates for personal-finance transaction Q&A specifically — (search finding; see Gaps)

### Inferences
- **Architecture for joy (M):**
  1. *Tool layer in Rust:* about 10-15 typed functions over SQLite with explicit parameters (date range, account, category, merchant), each returning JSON with amounts in paise plus pre-formatted ₹ strings (Indian lakh/crore grouping) so the LLM never formats or adds numbers itself.
  2. *LLM loop:* the user question goes to the LLM with tool definitions; it calls tools; the final answer is written only from tool output. Ask the model to cite tool-result keys or wrap each number in a placeholder like `{{t1.total}}`, which the server substitutes. That makes hallucinated numbers impossible.
  3. *Post-check:* regex every ₹ amount and percentage in the answer and confirm each exists in (or is a trivial derivation of) the tool results. Otherwise retry or drop the sentence (a lightweight version of FinGround's atomic-claim check).
  4. *Optional text-to-SQL escape hatch:* for questions the tools don't cover, let the LLM write a SELECT against a documented read-only view (e.g. `v_txn(date, amount_paise, direction, merchant, category, account, rail)`). Run it with a read-only SQLite connection (`SQLITE_OPEN_READONLY`), an `EXPLAIN` check, a row and time limit, and an authorizer callback denying writes and PRAGMAs (rusqlite supports `authorizer`). Show the SQL to the user for transparency.
  5. *Monthly narrative (S-M):* compute a fixed "facts pack" deterministically (income, spend, savings rate, top 3 category changes with ₹ and %, new subscriptions, price hikes, fees paid, anomalies, goal progress, forecast for next month). The LLM only turns the facts into 5-8 sentences of prose with placeholders. Cache per month. This is the safest high-value LLM feature.
  6. *Privacy:* send aggregates and normalized merchant names, not raw narrations with names, VPAs and account fragments. Make the LLM provider user-configurable, including a local model (e.g. via an Ollama-compatible endpoint) for privacy-minded users. India's DPDP Act 2023 is relevant for any multi-user deployment (not researched here).
- **Pitfalls:** time-relative phrases ("last month", "this FY", where the Indian FY runs Apr-Mar) need deterministic date resolution in tools; category ambiguity (is "Amazon" shopping or groceries?); LLM overconfidence in advice (add a disclaimer that it isn't investment advice); cost and latency (cache narratives; use a small model for routing).
- **Effort:** tool-calling Q&A M; monthly narrative S-M; guarded text-to-SQL M.

### Gaps
- No published hallucination or accuracy benchmarks for LLM Q&A over personal transaction ledgers were found.
- No engineering write-ups from Copilot, Monarch or Indian apps on their LLM assistants' grounding methods were retrieved.
- Regulatory constraints for LLM-generated financial guidance in India (SEBI investment-adviser rules, DPDP) were not researched.
