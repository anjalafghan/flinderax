# India Personal Financial Data Access for a Self-Hosted App ("joy", Rust backend), as of Nov 2026

Context: solo developer, personal/couples use, banks with HDFC and Kotak, holds credit cards, SIPs/MFs, RDs, FDs. Every source is rated for whether it works for an **individual** or needs a **registered or regulated business**.

## 1. RBI Account Aggregator (AA) framework: how it works, who can be an FIU, cost, coverage

### Takeaway
The AA network is mature in 2026: 17 AAs, about 176 FIPs, about 1,020 FIUs, and Sahamati recognised as SRO on 5 June 2026. An FIU must be regulated by RBI, SEBI, IRDAI or PFRDA, so **an individual or hobby app cannot become an FIU**. TSPs such as Setu only provide technology to licensed FIUs. They do not lend you a licence. For joy, AA is not a direct path unless the user gets data through a regulated partner app's export.

### Cited Findings
- Roles: FIP (data holder such as a bank, RTA or depository), AA (consent manager that routes encrypted data and holds no decryption key), FIU (regulated data consumer), TSP (technology vendor) and Central Registry (Sahamati) — [ecorpit.com, 2026](https://ecorpit.com/account-aggregator-integration-fintech-builders-2026/)
- ReBIT consent flow: `POST /Consent` → `/Consent/create` → `/read` → `/accept` → `/Consent/Notification` → `/Consent/fetch` → `/revoke`. Routing uses an `x-request-meta` header carrying the Central Registry ID. The artefact sets FI type, purpose and duration. Accounts must be discovered and linked before consent. Specs are published at api.rebit.org.in and certification is tied to a specific spec version — [ecorpit.com](https://ecorpit.com/account-aggregator-integration-fintech-builders-2026/)
- **FIU eligibility: the FIU must be regulated by RBI, SEBI, IRDAI or PFRDA. A fintech without an NBFC, lending or investment-advisory licence cannot become an FIU directly. It must get a licence or work with a regulated FIU partner** — [ecorpit.com](https://ecorpit.com/account-aggregator-integration-fintech-builders-2026/)
- Setu's go-live checklist starts with the FIU's regulatory licence and requires Sahamati certification. Setu submits the licence and certificate to Sahamati for the FIU — [Setu docs: go-live](https://docs.setu.co/data/account-aggregator/licenses-and-go-live/go-live); [Setu docs: get started](https://docs.setu.co/data/account-aggregator/v1/get-started)
- Certification is done by a Sahamati-empanelled auditor and covers security, API schema, FI schema and Central Registry integration. Live FIUs must run quarterly self-tests and get a CISA-auditor IS audit every two years — [ecorpit.com](https://ecorpit.com/account-aggregator-integration-fintech-builders-2026/)
- An RBI-regulated entity that joins as an FIU must also join as an FIP if it holds qualifying data. This "FIP-first" rule dates from November 2023 — [ecorpit.com](https://ecorpit.com/account-aggregator-integration-fintech-builders-2026/)
- Network scale: 1,120 live regulated entities, 176 FIPs, 1,020 FIUs, 17 operational AAs, more than 450M consents fulfilled and 294M linked accounts — [ecorpit.com](https://ecorpit.com/account-aggregator-integration-fintech-builders-2026/). Sahamati reports about 3.8 crore financial services in FY26 — [Sahamati Varta June 2026](https://sahamati.org.in/varta-the-sahamati-newsletter-june-2026/)
- RBI recognised Sahamati as the AA ecosystem SRO on 5 June 2026 — [ecorpit.com](https://ecorpit.com/account-aggregator-integration-fintech-builders-2026/); [pulse.bot](https://www.pulse.bot/banking/news/rbi-names-sahamati-sro-cementing-governance-of-indias-account-aggregator-network-4f8c011e-1492-4960-a294-c8ec40885b22/)
- AA lists: Sahamati maintains the AA list and a daily FIP-AA mapping (updated 09:55 IST). AMCs take part through their RTAs (CAMS, KFintech) — [Sahamati AAs](https://sahamati.org.in/account-aggregators/); [Sahamati FIP-AA Mapping](https://sahamati.org.in/fip-aa-mapping)
- Data-type coverage (CASParser data, March 2026, cited in the article):
  - Savings accounts: 72 banks, singly held only.
  - Equities, MF units, ETFs, AIFs and REITs: via NSDL and CDSL, on 13 AAs each, with demat transaction history capped at two years.
  - MF folios: via CAMS and KFin, on 13 AAs.
  - GSTR-1/3B: via GSTN.
  - **Fixed and recurring deposits: only about 40% of banks.**
  - Insurance: 57 insurers.
  - Bonds, G-Secs, EPF and PPF: proposed, with no FIPs.
  - **Excluded: joint accounts, NRE/NRO accounts, and non-proprietor current accounts.**
  — [ecorpit.com](https://ecorpit.com/account-aggregator-integration-fintech-builders-2026/)
- NSDL is live as an FIP — [NSDL](https://nsdl.com/financial-information-provider). CDSL lists its FIP partners — [CDSL](https://www.cdslindia.com/Investors/FIP.html)
- Pricing:
  - Sahamati publishes no standard price and participants negotiate costs. CASParser estimates a first-year FIU build at ₹5–25 lakh or more over 5–10 months — [ecorpit.com](https://ecorpit.com/account-aggregator-integration-fintech-builders-2026/)
  - A standardised fee model was reported to start on 1 Nov 2026, split between FIP, AA and Sahamati and billed once through the AA — [caalley.com](https://caalley.com/news-updates/indian-news/explained-what-is-account-aggregator-framework-and-how-does-it-work); see also [Sahamati Economic Model Committee](https://sahamati.org.in/economic-model-committee/)
  - Sahamati annual membership: ₹5L (Category A), ₹1.25L (B), ₹25K (C), plus GST — [Sahamati Member Fee Policy](https://sahamati.org.in/wp-content/uploads/2025/04/Sahamati-Member-Fee-Policy-2.pdf)
  - Unverified third-party estimates: Setu at about ₹10–25 per fetch, Finvu at about ₹3–6 per account — [productgrowth.in Setu](https://productgrowth.in/tools/banking-api/setu/); [productgrowth.in Finvu](https://productgrowth.in/tools/banking-api/finvu/). These numbers conflict and are low quality.
- DPDP: consent manager registration falls around Nov 2026 and full compliance around May 2027. Sahamati's position is that AAs can register as sector-specific consent managers — [ecorpit.com](https://ecorpit.com/account-aggregator-integration-fintech-builders-2026/)

### Inferences
- **Individual: not feasible as an FIU.** The licence gate (NBFC, SEBI RIA, IRDAI or PFRDA entity), certification, audits and ₹5–25L build cost rule it out. TSPs (Setu, Finarkein, FinBox, Perfios) sell to licensed FIUs and do not change eligibility.
- Setu's sandbox and the ReBIT JSON/XML FI schemas are still useful as a **data model reference**. Mapping joy's ledger schema to the ReBIT Deposit/TermDeposit/RecurringDeposit/MutualFunds schemas would make a later pivot to AA or a regulated partner cheap.
- Joint accounts are excluded, which matters for a couples app. Shared HDFC or Kotak joint accounts would not come through AA even with a licence.
- Even with a licence, FD/RD coverage at about 40% of banks means HDFC or Kotak term-deposit coverage is uncertain without checking the mapping page.

### Gaps
- Not confirmed from a primary source: HDFC and Kotak FIP status by FI type (deposit, TD, RD). Check the Sahamati FIP-AA mapping page directly. My background knowledge says both large private banks went live as FIPs years ago for savings accounts, but this is unverified here.
- The fee amounts under the 1 Nov 2026 standardised model were not found.
- No primary source found on whether consumer AA apps (Finvu, OneMoney, Anumati) let an individual export their own fetched data as a file. If they do, that would be a legitimate individual path.
- NPS live as an FI type was not confirmed. The data-type list above does not mention NPS.
- Pine Labs/Setu AA licence change (Agya): single low-quality source, unverified.

## 2. Bank SMS alerts (HDFC, Kotak, credit cards)

### Takeaway
SMS is the freshest source (near real time), but it is incomplete and fragile. HDFC skips UPI SMS below ₹100 debit and ₹500 credit. Formats are undocumented. Play Store blocks READ_SMS for most apps. For a self-hosted personal app, a sideloaded APK or a forwarding app that sends SMS to joy's endpoint is the practical approach, and Play policy does not apply to sideloaded apps.

### Cited Findings
- HDFC stopped SMS alerts for UPI payments up to ₹100 sent and ₹500 received from 25 June 2024. Email alerts continue for all transactions — [Business Today](https://www.businesstoday.in/industry/banks/story/hdfc-bank-to-stop-sms-alerts-for-upi-payments-up-to-rs-100-email-notifications-to-continue-431258-2024-05-29); [Business Standard](https://www.business-standard.com/finance/personal-finance/hdfc-bank-to-stop-sms-alerts-for-upi-payments-up-to-rs-100-check-details-124052901252_1.html)
- Google Play policy: SMS and Call Log permissions are restricted to default SMS, Phone or Assistant handlers. Temporary exceptions exist for listed core functions, including "SMS-based financial transactions" (UPI, verification) and "SMS-based money management" (budget tracking), subject to review — [Play Console Help 10208820](https://support.google.com/googleplay/android-developer/answer/10208820?hl=en); [Play Console Help 9047303](https://support.google.com/googleplay/android-developer/answer/9047303)
- In practice exceptions are often denied. A personal-finance developer was rejected as "not qualifying", and Bluecoins dropped SMS features to stay on Play — [dev.to](https://dev.to/zeta_byte/ive-been-working-on-a-personal-finance-app-called-finvantage-and-i-recently-hit-a-roadblock-4mjh); [Bluecoins](https://bluecoinsapp.com/?p=9286)
- Play policy updates were previewed for July 2026. The preview covers READ_CALL_LOG verification and is unclear on finance exceptions — [Play Console Help 17225965](https://support.google.com/googleplay/android-developer/answer/17225965)
- SMS Retriever API reads OTPs without inbox access, so it is useless for transaction parsing — [lendsqr blog](https://blog.lendsqr.com/does-android-still-allow-lenders-to-scrap-sms-from-phones-for-scoring/)
- TRAI header suffixes: telcos append -P (promotional), -S (service), -T (transactional) or -G (government) to sender headers based on the DLT template category. Enforcement started around 6 May 2025, and COAI said all telcos had implemented it by July 2025 — [MediaNama May 2025](https://www.medianama.com/2025/05/223-suffixes-commercial-sms-headers-spam/); [MediaNama July 2025](https://www.medianama.com/2025/07/223-telcos-suffixes-commercial-messages-users/); [India TV](https://www.indiatvnews.com/technology/news/understanding-sms-suffixes-what-s-g-p-and-t-mean-and-how-they-help-you-avoid-scams-2025-08-29-1005732)
- Bank templates are registered on DLT, and variable parts are scrubbed against the template — [MSG91 DLT scrubbing](https://msg91.com/help/dlt-template-scrubbing-filtering-rules)

### Inferences
- Sender filtering: match on a header like `XX-HDFCBK-S` / `-T` or `XX-KOTAKB-S`, where the operator/circle prefix varies. Strip the prefix and suffix, then whitelist the core ID (for example HDFCBK, KOTAKB). Accept only `-S`/`-T` to reject `-P` spam and phishing lookalikes. The exact header IDs come from background knowledge and should be verified against real inboxes.
- Because messages come from DLT templates, each bank has a finite set of templates. Build a versioned regex or template table per sender and log unmatched messages for review instead of guessing. HDFC formats seen in the wild (for example "Sent Rs.X From HDFC Bank A/C *1234 To ... On dd/mm/yy Ref ...") were **not verified from a source** here.
- Pitfalls:
  - Missing small UPI SMS.
  - Duplicate alerts (SMS plus email).
  - Credit card "spent" alerts vs later reversals or refunds.
  - Card alerts show the masked last four digits only.
  - Balances appear on some templates only.
  - Date formats vary (dd-mm-yy, dd-MMM-yy).
  - Amounts use "Rs." vs "INR" and Indian lakh comma grouping.
  - OTP and promo noise.
  - Templates change silently.
- Self-hosted path: a personal sideloaded Android app with READ_SMS or RECEIVE_SMS, or an open-source SMS-forwarder app (for example the "SMS Forwarder" / "Android SMS Gateway" style apps on F-Droid or GitHub), POSTs to the Rust endpoint over Tailscale or HTTPS. Play rules do not apply to sideloads. iOS cannot read SMS, but iOS Shortcuts automations can forward messages on newer iOS. The iOS part is unverified.

### Gaps
- No primary or published sample templates found for HDFC, Kotak or card issuers (HDFC, Kotak, ICICI, SBI Card, Axis). Collect from your own inbox.
- Whether HDFC's ₹100/₹500 thresholds changed after 2024 is not confirmed. Kotak's SMS threshold policy was not found.

## 3. Bank statements (HDFC, Kotak netbanking; password PDFs; email statements, Gmail API)

### Takeaway
Statement downloads are the most reliable bulk source for an individual: free, complete, and including joint accounts. HDFC netbanking offers PDF, Excel, Text, Delimited and MSMoney formats, so **prefer the Delimited or Excel export over PDF parsing**. Monthly emailed PDF statements are password-protected and can be pulled automatically via Gmail API in "testing" mode at personal scale.

### Cited Findings
- HDFC NetBanking statement download formats: PDF, Excel, Text, Delimited and MSMoney. Email statements are PDF only — [HDFC NetBanking FAQ](https://www.hdfc.bank.in/need-help/net-banking-faqs); [HDFC blog](https://www.hdfc.bank.in/blogs/digital-banking-1/download-account-statement-via-netbanking)
- HDFC says the password for the statement PDF is the numeric Customer ID. Third-party guides give name-plus-DOB formats, which conflict and probably apply to other HDFC products such as credit card statements — [HDFC NetBanking FAQ](https://www.hdfc.bank.in/need-help/net-banking-faqs); contradicted by [ClearTax](https://cleartax.in/s/hdfc-bank-statement-password), [bankconv](https://bankconv.helpscoutdocs.com/article/16-hdfc-statement-password)
- Gmail API: `gmail.readonly` is a restricted scope. Unverified apps using restricted or sensitive scopes are capped at 100 new users, and the CASA security assessment is required for production use beyond test users — [Google Cloud OAuth FAQ](https://support.google.com/cloud/answer/13463817); [unipile](https://www.unipile.com/gmail-api-scopes-guide/)
- Sources conflict on whether a local-only or personal app avoids the assessment entirely. In practice, staying in "Testing" status with yourself and your partner as test users avoids it — [Google Security community thread](https://security.googlecloudcommunity.com/google-security-operations-2/what-happened-to-local-app-gmail-api-access-7138); [singhamandeep](https://singhamandeep.com/category/google-youtube-api/)

### Inferences
- **Individual: fully feasible.**
  - Download HDFC Delimited or XLS manually, monthly, and import it.
  - Auto-ingest emailed PDF statements and card statements via Gmail API with your own OAuth client in Testing mode. Note that testing-mode refresh tokens expire after 7 days (background knowledge, unverified here).
  - Alternatively, use IMAP with an app password, or a Gmail filter that forwards to a self-hosted mailbox, which avoids OAuth verification.
- Rust tooling: `lopdf`/`pdf` crates can decrypt standard PDF encryption. `pdf-extract`, or shelling out to `qpdf --password=... --decrypt` plus `pdftotext -layout`, work for text extraction. For tables, Python `camelot` or `pdfplumber` as a sidecar is more mature than Rust options. Treat XLS via `calamine` (Rust) as the preferred bank path.
- Kotak: background knowledge says netbanking offers PDF/XLS/CSV statement download and a password-protected e-statement. This is **not verified** here.

### Gaps
- Kotak netbanking export formats and e-statement password rule were not found in sources.
- Credit card statement password formats (HDFC, Kotak cards) were not verified. Third-party guides mention name-plus-DOB formats.
- The exact column layout of the HDFC Delimited export was not documented in sources.

## 4. Mutual funds: CAMS/KFintech CAS, MF Central, NSDL/CDSL CAS

### Takeaway
CAS is the best MF source for an individual. One PDF covers all folios across all AMCs, it is free on request, and the mature open-source parser `casparser` (MIT, Python) handles CAMS/KFintech detailed and summary CAS plus NSDL/CDSL demat CAS. Ingest by email: request it, the PDF arrives in Gmail, then parse it with a casparser sidecar. MF Central's CAS uses a different template that casparser v1.0 does not support.

### Cited Findings
- casparser supports CAMS and KFintech detailed and summary CAS (mailback and portal), plus NSDL and CDSL demat CAS. Output is a pydantic model, JSON or CSV, and it includes a capital-gains report in Schedule 112A format. It is MIT-licensed, installed with `pip install -U casparser`, and v1.0 is built on pypdfium2 — [casparser GitHub](https://github.com/codereverser/casparser)
- casparser limitations: it rejects re-printed or re-rendered PDFs and needs the original issuer file. MF Central statements are out of scope for v1.0. ISIN data may need `casparser-isin --update`. Non-empty `parse_warnings` means the data may be unreliable — [casparser GitHub](https://github.com/codereverser/casparser)
- A detailed CAS can cover the current FY, the previous FY, or a specific period of up to 365 days — [AMFI CAS page](https://www.amfiindia.com/investor-corner/investor-center/cas.html)
- MF Central: log in with PAN and mobile plus OTP, then use CAS/Statement and pick summary or detailed and a date range. The user sets the PDF password at request time — [inkl](https://www.inkl.com/news/how-to-download-cas-from-mfcentral); [InCred](https://www.incredmoney.com/knowledge-center/share-market/how-to-download-consolidated-account-statement-cas-step-by-step-instructions/)
- Depository CAS passwords: CDSL uses the uppercase PAN, NSDL the 8-digit Client ID (per a third-party guide) — [InCred](https://www.incredmoney.com/knowledge-center/share-market/how-to-download-consolidated-account-statement-cas-step-by-step-instructions/)
- Platforms sync via an "MF Central CAS API" with OTP. No public developer spec was found, and this appears to be available to registered intermediaries — [Geojit support](https://support.geojit.com/support/solutions/articles/89000018852-how-to-fetch-my-central-consolidated-account-statement-cas-from-fundsgenie-)
- MF folios are also available via AA (CAMS and KFin as FIPs) to licensed FIUs — [ecorpit.com](https://ecorpit.com/account-aggregator-integration-fintech-builders-2026/)

### Inferences
- **Individual: fully feasible.** Request the CAMS "mailback" detailed CAS. Background knowledge says it is requested on the CAMS website, covers CAMS plus KFintech funds, and is emailed in minutes with a user-chosen password. Then fetch it via the Gmail API and parse it with a casparser subprocess, either a Python sidecar or a `uv`-run script that emits JSON for Rust to consume.
- SEBI/NSDL/CDSL send a monthly demat-plus-MF CAS automatically when there are transactions. This is a free, zero-effort feed if email ingestion exists.
- SIP NAV valuation between statements: use AMFI's daily NAV file (NAVAll.txt) to mark-to-market. This comes from background knowledge and is not sourced here.
- The MF Central API is **business-only** in practice.

### Gaps
- No official MF Central developer API documentation or eligibility criteria found.
- The CAMS mailback URL and current KFintech request flow were not verified in this session.

## 5. EPFO, NPS, PPF, credit reports, Form 26AS/AIS

### Takeaway
All of these are individual-accessible manual downloads behind OTP or login, with no public APIs for individuals. The useful ones are the AIS JSON export (machine-readable tax view: interest, dividends, MF and securities transactions, TDS) and the EPFO passbook PDF. Credit reports give one free full report per bureau per year. Treat these as quarterly or annual manual imports.

### Cited Findings
- EPFO passbook: needs an activated UAN. Use passbook.epfindia.gov.in, select the Member ID, View and then Download Passbook as PDF. UMANG is an alternative with UAN plus OTP. Blockers are an unactivated UAN and Aadhaar or mobile mismatch — [Upstox 2026](https://upstox.com/learning-center/personal-finance/uan-passbook-guide-2026-check-epf-balance-online/article-1643/); [BankBazaar](https://bankbazaar.com/saving-schemes/epf-e-passbook.html)
- One source reports EPFO portals went offline around 26 June 2026 for a database consolidation and upgrade. This is a single source and unverified — [fermor.in](https://fermor.in/news/epfo-login-uan-passbook)
- EPF and PPF are "proposed" FI types in AA with no FIPs live, so they are not available via AA — [ecorpit.com](https://ecorpit.com/account-aggregator-integration-fintech-builders-2026/)
- AIS: download from the e-filing portal (Services, then AIS) in PDF or JSON, and CSV per some guides. The offline AIS Utility imports the JSON — [TaxGuru](https://taxguru.in/income-tax/annual-information-statement.html); [Quicko](https://learn.quicko.com/ais-annual-information-statement); [TaxGuru AIS utility guide](https://taxguru.in/income-tax/annual-information-statement-ais-user-guide-ais-utility.html)
- From AY 2023-24, Form 26AS on TRACES shows only TDS/TCS. Interest, securities and other data moved to AIS — [Quicko](https://learn.quicko.com/ais-annual-information-statement)
- Credit reports:
  - RBI requires each credit information company (CIBIL, Experian, Equifax, CRIF High Mark) to give one free full credit report with score per calendar year — [TaxGuru](https://taxguru.in/rbi/provide-free-annual-credit-report-individuals-rbi.html); [Deccan Herald](https://www.deccanherald.com/amp/story/business%2Fprovide-full-credit-report-cics-2084087)
  - Lenders must report to bureaus fortnightly (15th and month-end) from 1 Jan 2025 under RBI/2024-25/60 — [RBI notification](https://website.rbi.org.in/web/rbi/-/notifications/frequency-of-reporting-of-credit-information-by-credit-institutions-to-credit-information-companies); [Taxmann](https://taxmann.com/post/blog/rbi-directs-credit-information-companies-institutions-to-update-credit-information-fortnightly-or-at-shorter-intervals)
  - A claim of moving to weekly reporting is uncorroborated — [billcut](https://www.billcut.com/blogs/rbi-weekly-credit-reporting/)

### Inferences
- **Individual: feasible, manual.**
  - AIS JSON is the highest-value import here. It cross-checks bank FD/RD interest (TDS) and MF redemptions and dividends, and gives a once-a-year reconciliation of joy's ledger against the tax department's view.
  - EPFO passbook PDF can be parsed with a simple table extractor. Expect annual interest credits only.
  - Credit report PDFs are mainly useful for a list of open card and loan accounts and their limits.
- Bureau APIs (CIBIL, Experian) are B2B and need a membership or licensed entity, so they are **business-only**.

### Gaps
- NPS: no sources gathered this session on CRA statement downloads (Protean CRA, KFintech CRA, CAMS CRA) or formats. Background knowledge: each CRA portal offers a transaction statement PDF after PRAN login, and NPS was not confirmed as live on AA. Needs follow-up.
- PPF: no source gathered. PPF sits with the operating bank or post office, and statements appear in the bank's netbanking (HDFC lists PPF in its account summary per background knowledge, unverified).
- The exact AIS JSON schema was not documented in sources found.

## 6. Email parsing of transaction emails and card statements

### Takeaway
Email is the best complement to SMS for an individual. HDFC sends email alerts for **all** UPI transactions, including those below the SMS thresholds, and card statements and CAS arrive by email. Gmail API in Testing mode with your own and your partner's accounts, or IMAP, is feasible without Google verification at personal scale.

### Cited Findings
- HDFC keeps email alerts for all UPI transactions while SMS skips small ones — [Business Today](https://www.businesstoday.in/industry/banks/story/hdfc-bank-to-stop-sms-alerts-for-upi-payments-up-to-rs-100-email-notifications-to-continue-431258-2024-05-29)
- The Gmail restricted-scope 100-user unverified cap applies, so personal use fits under it — [Google Cloud OAuth FAQ](https://support.google.com/cloud/answer/13463817)
- Emailed bank statements are PDF only (HDFC) — [HDFC NetBanking FAQ](https://www.hdfc.bank.in/need-help/net-banking-faqs)

### Inferences
- Architecture: use Gmail label or query filters (`from:alerts@hdfcbank.net`, `from:...kotak...`, `subject:"Consolidated Account Statement"`). The Rust worker uses the `google-gmail1` or `reqwest` REST client with incremental `history.list` sync, then routes messages:
  - HTML alert to an HTML-to-text extractor plus per-sender templates.
  - PDF attachment to decrypt and parse (casparser for CAS, a bank-specific parser for statements).
  - Dedupe against SMS-sourced records by amount, date and account suffix.
- Sender email addresses above come from background knowledge, so verify them from your inbox.
- Couples: each partner authorises their own Gmail as a test user. Joint accounts generate alerts to the primary holder's email.

### Gaps
- No verified sample email templates for HDFC, Kotak or card issuers were found.
- Gmail Testing-mode token expiry (7 days) is not confirmed by a source in this session.

## Summary feasibility matrix (inference from above)

| Source | Data | Format | Freshness | Cost | Individual? |
|---|---|---|---|---|---|
| Account Aggregator (direct FIU) | Deposits, some FD/RD, MF, demat, insurance, GST | ReBIT JSON/XML | Near real time | ₹5–25L build plus fees | **No: regulated FIU needed** |
| AA via TSP (Setu, Finarkein, FinBox, Perfios) | Same | JSON | Near real time | Per-fetch fees | **No: TSP serves licensed FIUs** |
| SMS alerts (sideloaded app or forwarder) | Debits/credits, card spends | Free text (DLT templates) | Seconds | Free | Yes (not via Play Store) |
| Netbanking export (HDFC XLS/Delimited; Kotak) | Full account history incl. joint | XLS/CSV/PDF | Manual, on demand | Free | Yes |
| Emailed statements and alerts (Gmail API/IMAP) | Alerts, monthly statements, card statements | HTML, password PDF | Minutes to monthly | Free | Yes (Testing mode, 100 users or fewer) |
| CAMS/KFin CAS plus casparser | All MF folios, transactions | Password PDF to JSON | On request or monthly | Free | Yes |
| NSDL/CDSL CAS | Demat plus MF | Password PDF | Monthly | Free | Yes |
| MF Central API | MF | API | Real time | n/a | No (intermediaries) |
| AIS (JSON) / 26AS | Tax-reported income, TDS, securities | JSON/PDF/CSV | Annual or quarterly | Free | Yes (manual) |
| EPFO passbook | EPF balance, contributions | PDF | Monthly contributions, annual interest | Free | Yes (manual) |
| Credit report | Accounts, limits, score | PDF | Fortnightly bureau updates | 1 free per bureau per year | Yes (manual); bureau API business-only |
