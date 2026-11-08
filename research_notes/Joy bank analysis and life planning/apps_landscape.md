# Personal finance apps landscape (global + India), 2024–2026: transaction analysis and life-planning features

Context: research for "joy", a self-hosted Indian personal finance and couples date-planning web app. It already has safe-to-spend until payday, bills and commitments, credit card billing cycles, goals and sinking funds, what-if projections, "can I afford it", bank SMS ingest, CSV import, net worth history, a tax panel, a month-close review and a shared date jar.

Source-quality warning: most 2026 "review" results from search are affiliate or SEO pages, some apparently AI-generated (budgettools-net.pages.dev, joingerald.com, getfinny.app, fincomparelab, pocketclear.app and similar). Several are written by competitors, for example getfinny.app and pocketclear.app. These notes use them only for facts that recur across sources, and say so where a claim rests on one weak source. Official vendor pages fetched: useorigin.com and the Copilot help centre. Monarch's /features URL returned a 404. Of the well-known review outlets, only The Penny Hoarder and NerdWallet/Experian (for Honeydue) surfaced. Wirecutter and The Verge did not.

## Global apps: signature analysis features, praise and complaints

### Takeaway
By 2026 the leading global apps all have automatic categorisation, recurring/subscription detection, budgets and net worth. They differ in four ways: (a) how they treat recurring bills in "left to spend" (Copilot), (b) partner sharing (Monarch, Origin), (c) AI assistants and forecasting (Origin, Monarch), and (d) budgeting philosophy (YNAB's zero-based method vs Monarch's guardrail-style budgets). Self-hosted Actual Budget competes on envelope budgeting plus rules and schedules.

### Cited Findings
**Copilot Money**
- During onboarding, Copilot detects likely recurring transactions for the user to review, organises them into a monthly view, and lets users create new recurrings from any previous transaction — [Copilot help: Quick start](https://help.copilot.money/en/articles/11157550-quick-start-guide)
- Copilot deliberately keeps recurring transactions out of the Dashboard's "left to spend" chart, so large bills don't look like overspending — [Copilot vs Rocket Money (vendor page)](https://www.copilot.money/compare/rocket-money)
- Recurring handling is the feature reviewers most often praise. Budget rollover is optional, and budgets can be rebalanced monthly or immediately — [Penny Hoarder Copilot review](https://www.thepennyhoarder.com/?p=225952); [budgettools review (low quality)](https://budgettools-net.pages.dev/copilot-money-review.html)
- Copilot is Apple-only with no Android app; a web version launched in December 2025 — [budgettools review (low quality)](https://budgettools-net.pages.dev/copilot-money-review.html)
- One source quotes pricing of about $13/month or $95/year — [getfinny (competitor blog)](https://getfinny.app/blog/copilot-money-pricing-2026)

**Monarch Money**
- Monarch costs about $14.99/month or $99.99/year and has no free tier after a 7-day trial. The Penny Hoarder describes a two-tier "Core"/"Plus" structure; other sources name a single "Premium" tier (conflicting) — [Penny Hoarder Monarch review](https://www.thepennyhoarder.com/budgeting/monarch-money-review/); [subger](https://subger.com/en/service/monarch-money)
- One unverified source ties a $199.99/yr "Plus" tier to retirement forecasting — [fincomparelab](https://www.fincomparelab.com/reviews/monarch-money-review/)
- Monarch includes a partner seat on Core. Two people log in with separate emails and see the same shared budget. Its budget targets work as guardrails rather than YNAB-style active allocation — [Penny Hoarder couples apps 2026](https://www.thepennyhoarder.com/budgeting/best-budgeting-apps-couples/?aff_id=401); [Gerald Monarch vs YNAB](https://joingerald.com/learn/financial-wellness/monarch-vs-ynab-comparison)
- Reviews mention Plaid sync, investment tracking, about 13,000 institutions (March 2026), shared dashboards and detailed net worth — [Gerald review](https://joingerald.com/learn/financial-wellness/monarch-money-review-2026-app)
- A competitor blog calls Monarch's AI assistant "genuinely useful" and says Monarch closed the categorisation-accuracy gap with Copilot in a 2026 update — [getfinny (competitor, single source)](https://getfinny.app/blog/monarch-money-review-2026)

**YNAB**
- YNAB costs about $109/year. Investopedia's March 2026 evaluation named it the overall winner among budgeting apps — [pulse.bot roundup](https://www.pulse.bot/personal-finance/news/best-budgeting-apps-for-march-2026-8e3908eb-7888-442c-8875-08e5c04a4036)
- YNAB is the zero-based budgeting pick and described as most likely to produce measurable change within 90 days. Sources conflict on couples support: one says shared credentials are needed, another lists couples support — [switchwize YNAB vs Monarch vs Rocket](https://www.switchwize.com/learn/ynab-vs-monarch-vs-rocket-money)

**Rocket Money**
- Rocket Money is subscription-focused, with Financial Goals, "Smart Savings" and streamlined net worth tracking — [Rocket Money vs YNAB (vendor)](https://rocketmoney.com/learn/personal-finance/rocket-money-vs-ynab)
- Rocket Money assumes one user per account and has no shared budget — [switchwize](https://www.switchwize.com/learn/ynab-vs-monarch-vs-rocket-money)
- Premium costs about $7–14/month (pay-what-you-want range). A 2026 roundup notes $19.99 AI tiers becoming a baseline in the category — [pulse.bot roundup](https://www.pulse.bot/personal-finance/news/best-budgeting-apps-for-march-2026-8e3908eb-7888-442c-8875-08e5c04a4036)

**Origin (useorigin.com), official page**
- The AI advisor gives proactive alerts and suggestions, and "Ask Origin" answers personalised questions about the user's finances — [Origin](https://useorigin.com/)
- Origin offers transaction tracking, custom budgets and subscription management, plus forward-looking forecasting and scenario planning — [Origin](https://useorigin.com/)
- It also offers a Financial Health Score with prioritised action items, tax-strategy suggestions (dependent-care FSA, HSA), estate planning, investing and a cash account — [Origin](https://useorigin.com/)
- It gives credit guidance such as lowering card utilisation before the statement closes, which is a billing-cycle-aware tip — [Origin](https://useorigin.com/)
- Partners can be invited to budget together at no extra cost on the membership — [Origin](https://useorigin.com/)

**Self-hosted and indie**
- Actual Budget is an envelope budgeter. It has Schedules for recurring transactions, Rules for payee renaming and category assignment, bank sync via GoCardless (EU/UK) or SimpleFIN (US/CA), and OFX/QFX/CSV and YNAB import — [elfhosted Actual listing](https://store.elfhosted.com/product/actual-budget/); [Lunch Money comparison (vendor)](https://lunchmoney.app/compare/actual-budget-alternative)
- One blogger runs a monthly Claude-based review of Actual data that suggests new categorisation rules — [personal blog](https://personal-site-9gz.pages.dev/posts/2025/you-need-an-actual-budget)
- Lunch Money is hosted only, not self-hostable. It costs $10/month with no tiers, uses Plaid sync, and its recurring items take custom schedules (e.g. twice a month) and auto-match new transactions — [Lunch Money recurring](https://lunchmoney.app/features/recurring-expenses/); [getfinny Lunch Money review](https://getfinny.app/blog/lunch-money-review-2026)
- Lunch Money users have requested recurring/indefinite budgets, which is still "in progress" — [Lunch Money feedback board](https://feedback.lunchmoney.app/budget/p/set-a-recurring-budget)
- Maybe Finance, the open-source self-hosted app, is reported archived — [dev.to Actual vs Maybe](https://dev.to/selfhostingsh/actual-budget-vs-maybe-1e5m)
- Firefly III remains a self-hosted option — [dev.to Firefly III 2026](https://dev.to/royce_fabbd83cb268312e928/self-host-firefly-iii-personal-finance-manager-2026-4ndi)

### Inferences
- Copilot's rule that bills don't count against "left to spend" is the same idea as joy's safe-to-spend-until-payday (income minus committed bills). That suggests joy's core metric matches a best-in-class pattern.
- Origin is the clearest "planning, not just tracking" product: AI Q&A, health score, scenarios, tax and estate. Joy's "can I afford it" plus what-if projections already overlap with Origin's scenario layer, and a health score with ranked action items is a cheap add.
- Origin's "lower utilisation before statement closing" tip fits joy's existing credit card billing-cycle data well.
- Rule-suggestion workflows (Actual plus LLM) and recurring auto-match with flexible schedules (Lunch Money) are well-liked mechanisms that suit a self-hosted app.

### Gaps
- I could not reach an official Monarch features page (404). Its cash-flow forecast, flex budgeting and AI assistant specifics are unverified from primary sources.
- I found no official 2025–2026 changelogs for YNAB or Rocket Money. Reddit sentiment for the global apps was not captured with citations.

## India: features, shutdowns and pivots

### Takeaway
India has split into two data pipelines: Account Aggregator (AA) consent-based linking (CRED Money, INDmoney, Fold) and on-device SMS parsing (axio/Walnut and a wave of 2026 privacy-first indie apps). Neobanks have retreated. Fi wound down banking in March 2026 and pivoted to enterprise AI. Investment-led apps (INDmoney, Kuvera, ET Money) dominate goal planning and tax harvesting, and CRED owns credit card management (it acquired Kuvera in 2024).

### Cited Findings
**Fi Money (epiFi): shutdown and pivot**
- In the week of 11 March 2026, customers were emailed that banking services on the Fi app would be discontinued. Federal Bank cited "business re-alignment": "Our partnership with Fi is ending." New users can no longer open accounts, and existing Federal Bank accounts stay active but move to the FedMobile app — [TechCrunch, 11 Mar 2026](https://techcrunch.com/2026/03/11/india-neobank-fi-winds-down-banking-services-on-its-platform)
- Fi was founded in 2019 by ex-Google Pay India execs, launched in 2021, and raised about $169M (Ribbit, B Capital, Alpha Wave, Peak XV). It had served more than 3.5M customers. Its pivot is to "deep tech", AI and enterprise systems — [TechCrunch](https://techcrunch.com/2026/03/11/india-neobank-fi-winds-down-banking-services-on-its-platform)
- The Fi-Federal co-branded credit cards were discontinued, reward accrual stopped, and holders were asked to clear dues and close the cards — [cardadvisor.in](https://cardadvisor.in/articles/fi-federal-credit-card-review-2026); [hellobanker](https://hellobanker.in/fi-neobank-to-shut-down-banking-services-on-its-platform-users-shifted-to-federal-bank-app/)
- Mint earlier reported Fi "shuts some features, lays off staff to stay afloat" — [Mint via Magzter](https://www.magzter.com/ar/stories/newspaper/Mint-Hyderabad/FI-MONEY-SHUTS-SOME-FEATURES-LAYS-OFF-STAFF-TO-STAY-AFLOAT). The exact date of that report is not confirmed. The assignment's framing of a 2025 shutdown is wrong: the banking wind-down was March 2026, and 2025 saw only cutbacks.

**Jupiter**
- Jupiter offers UPI and credit spend tracking with monthly "Insights" and spend categories — [Jupiter App Store](https://apps.apple.com/in/app/jupiter-money/id1507748747)
- It launched the Edge CSB Bank RuPay credit card (sold only via Jupiter) in June 2025 — [IBS Intelligence](https://ibsintelligence.com/?p=568601); [Tribune](https://www.tribuneindia.com/news/business/jupiter-bets-on-rupay-credit-card-partners-with-csb)
- "Pots" hold money aside for saving. Users have asked to spend directly from Pots (Aug–Nov 2025 threads) and for yearly or quarterly spend views. A credit card tracking "lab test" was flaky for some users — [Jupiter community features](https://community.jupiter.money/c/ideas/features/32); [Lab test #15 CC tracking](https://community.jupiter.money/t/new-lab-test-15-credit-card-tracking/53872?page=4)

**CRED / CRED Money / Kuvera**
- CRED Money launched in July 2024. Built on Account Aggregator, it gives a unified view of balances and transactions across banks, wallets and UPI IDs, spend analysis by merchant and category, and tracks dividends, reversals and tax refunds — [Inc42](https://inc42.com/buzz/kunal-shah-led-cred-unveils-new-offering-to-help-customers-manage-bank-accounts-track-expenses); [Entrackr](https://entrackr.com/2024/07/cred-launches-financial-management-platform-cred-money); [Business Standard](https://www.business-standard.com/amp/finance/news/cred-launches-product-to-track-expenses-view-total-bank-account-balances-124072400997_1.html)
- CRED Money sends recurring-payment reminders (SIPs, EMIs, rent, insurance premiums) that can be paid via CRED UPI — [goodreturns](https://www.goodreturns.in/classroom/cred-money-launched-an-all-in-one-pocket-to-manage-your-financial-transactions-how-to-use-1360011.html)
- CRED acquired Kuvera in February 2024. Kuvera offers goal planning, LTCG tax harvesting (realising up to ₹1.25 lakh of LTCG tax-free each year), family accounts and portfolio consolidation — [equitiesindia Kuvera review](https://equitiesindia.com/tools-directory/kuvera-review); [Kuvera App Store](https://apps.apple.com/app/id1329701793)
- One user values Kuvera as "pretty bare bones" with a useful tax-harvesting notification — [TechEnclave thread](https://techenclave.com/threads/alternatives-for-et-money-which-app-do-you-use-for-mutual-fund-or-stock-investing.223718)

**INDmoney**
- INDmoney offers a free credit card tracker: link multiple cards, see transactions, get due-date reminders and pay bills — [INDmoney CC bills](https://www.indmoney.com/features/track-credit-card-bills)
- It tracks bank accounts with categorised spend — [INDmoney bank tracking](https://www.indmoney.com/features/track-bank-account); [savings account tracking](https://www.indmoney.com/features/track-savings-account)
- It runs on Account Aggregator (not SMS). It also offers family wealth tracking, goal planning that maps investments to goals, and AI financial-health reports. Its strength is consolidation more than granular daily tracking — [equitiesindia INDmoney review](https://equitiesindia.com/tools-directory/indmoney-review); [getfinny SMS trackers (competitor)](https://getfinny.app/blog/sms-expense-tracking-app)

**ET Money**
- ET Money focuses on goal-based investing and tax-loss harvesting, and added automated NPS contributions and health-insurance comparison (2025) — [5paisa blog](https://www.5paisa.com/index.php/blog/best-mutual-fund-apps-in-india); [analyticsinsight 2025](https://www.analyticsinsight.net/apps/top-investing-apps-in-2025-for-safe-everyday-wealth-building)
- Users complained that a redesign pushed them toward the paid "Genius" plan, and some moved to Kuvera — [TechEnclave](https://techenclave.com/threads/alternatives-for-et-money-which-app-do-you-use-for-mutual-fund-or-stock-investing.223718)

**axio (formerly Walnut)**
- Walnut was the original Indian SMS-parsing expense tracker. Rebranded as axio, it has expanded into lending. The last Play Store update seen was 17 Feb 2025 (developer: Axio Digital). I found no shutdown report — [getfinny SMS trackers](https://getfinny.app/blog/sms-expense-tracking-app); [AppWisp listing](https://appwisp.com/app/android/com.daamitt.walnut.app)
- Older user reports say it works poorly on newer Android versions — [Jupiter community thread](https://community.jupiter.money/t/what-is-your-favorite-budgeting-app-and-why/1384)

**New 2025–2026 entrants (mostly self-reported claims)**
- MandateVault is an offline-first expense and SIP-mandate tracker that parses bank SMS on-device with no servers — [hunted.space](https://hunted.space/product/mandatevault)
- Ruply is on-device, with an EMI tracker and "festival envelopes" — [Product Hunt](https://www.producthunt.com/products/ruply)
- LABAM ("100% local") — [site](https://sites.google.com/view/labamapp/home)
- MyHishob (listed May 2026) — [peerpush](https://peerpush.com/p/myhishob)
- Spend AI does SMS auto-tracking with no sign-up — [Product Hunt](https://www.producthunt.com/products/spend-ai/launches)
- Fold is AA-based, with OTP to link banks — [lapaas Fold](https://voice.lapaas.com/startup/fold/business-model/)
- AA providers include Setu, Finvu, Onemoney, NeSL, CAMS Finserv and PhonePe Anumati — [richify AA brands](https://www.richify.ai/in/account-aggregator-brands)

### Inferences
- "Privacy-first, on-device, no login" is the most visible 2026 indie trend in India, which suits joy's self-hosted SMS-ingest approach well.
- Joy could add AA ingest later, but AA needs a regulated FIU relationship, which is likely out of reach for a solo self-hosted app. SMS plus CSV/statement PDF is the pragmatic route.
- Credit card management (due-date reminders, multi-card view, statement cycles) is table stakes in India: CRED, INDmoney and Jupiter all have it.
- LTCG/STCG tax harvesting is the signature Indian "planning" feature (Kuvera, ET Money). Joy's tax panel could add an LTCG ₹1.25L harvest nudge if MF holdings are imported, for example via a CAS statement.
- Indian festival and seasonal envelopes (Ruply) and EMI/SIP mandate tracking (MandateVault, CRED Money) are India-specific commitment types worth modelling in joy's bills/commitments.

### Gaps
- No primary-source info found on Fi's spend-analysis features (Ask Fi, Fi Pulse, etc.) before shutdown, or the exact date of the Mint layoffs article.
- Groww/Zerodha Coin features, and whether Groww or Zerodha added spend tracking, were not researched within the tool budget.
- Google Play SMS-permission policy constraints on SMS-reading apps (relevant to why SMS trackers declined) were not researched.
- No r/IndiaInvestments threads were captured with citations.

## Differentiators vs table stakes; most valued life-planning features

### Takeaway
Table stakes in 2026: bank/card sync or import, auto-categorisation with rules, recurring/subscription detection, budgets, net worth and bill reminders. Differentiators: recurring-aware "left to spend" (Copilot), shared partner views (Monarch, Origin), AI Q&A plus proactive insights (Origin, Monarch), forecasting and scenarios (Origin, Monarch Plus), tax optimisation (Origin; Kuvera/ET Money in India) and financial health scores (Origin, INDmoney).

### Cited Findings
- Every major app reviewed lists categorisation, recurring detection, budgets and net worth — [Copilot help](https://help.copilot.money/en/articles/11157550-quick-start-guide); [Rocket Money vs YNAB](https://rocketmoney.com/learn/personal-finance/rocket-money-vs-ynab); [Penny Hoarder best apps 2026](https://www.thepennyhoarder.com/budgeting/best-budgeting-apps/)
- AI assistants are moving into paid tiers ("$19.99 AI tiers set 2026's baseline") — [pulse.bot](https://www.pulse.bot/personal-finance/news/best-budgeting-apps-for-march-2026-8e3908eb-7888-442c-8875-08e5c04a4036)
- Retirement forecasting is gated to a higher Monarch tier (unverified single source) — [fincomparelab](https://www.fincomparelab.com/reviews/monarch-money-review/)
- Origin bundles scenario planning, a health score, and tax and estate planning as its premium value — [Origin](https://useorigin.com/)
- In India, goal-to-investment mapping (INDmoney, Kuvera) and tax harvesting (Kuvera, ET Money) are the planning features most often highlighted — [equitiesindia INDmoney](https://equitiesindia.com/tools-directory/indmoney-review); [equitiesindia Kuvera](https://equitiesindia.com/tools-directory/kuvera-review)

### Inferences
- Joy already covers most differentiators except an AI Q&A layer, a health score, and investment-to-goal mapping. The month-close review is close to the "proactive insights" pattern.
- The fact that vendors gate retirement and scenario tools to premium tiers suggests users value them. A self-hosted app can give them away.

### Gaps
- I found no usage data (analytics, surveys) showing which life-planning features are used most. "Most valued" is inferred from what vendors gate behind premium tiers and what reviewers emphasise, not from measured usage.

## Couples and shared-finance features

### Takeaway
Monarch (free partner seat, separate logins, shared budget) and Origin (free partner invite) are the main couples options among full-featured apps. Zeta, the couples-first app, shut down in May 2025 after Acorns bought its assets. Honeydue survives as a free, ad-supported, basic option with in-app chat. Rocket Money has no shared mode. No Indian app found has couples-specific features.

### Cited Findings
- Monarch includes a partner seat: two emails, one shared budget — [Penny Hoarder couples 2026](https://www.thepennyhoarder.com/budgeting/best-budgeting-apps-couples/?aff_id=401); [Gerald](https://joingerald.com/learn/financial-wellness/monarch-vs-ynab-comparison)
- Origin lets users invite a partner to build a budget together, free with membership — [Origin](https://useorigin.com/)
- Zeta reportedly shut down on 9 May 2025. Acorns acquired its assets in June 2025 but discontinued the couples product, and the app is gone from both stores — [pocketclear (competitor)](https://pocketclear.app/blog/zeta-app-alternative-couples.html); [debtpayofftools](https://debtpayofftools.pages.dev/apps/zeta)
- Honeydue is free and ad-supported. It offers bill tracking, budgets and in-app messaging/chat about transactions, but no savings-goal features and little future planning — [NerdWallet Honeydue review](https://www.nerdwallet.com/finance/learn/honeydue-app-review); [Experian Honeydue review](https://www.experian.com/blogs/ask-experian/honeydue-app-review/); [College Investor](https://thecollegeinvestor.com/35477/honeydue-review/)
- One App Store reviewer speculates that Honeydue shows "pre-shutdown signals" similar to Honeyfi/Firstly (anecdotal) — [Honeydue App Store](https://apps.apple.com/us/app/id1157633945)
- Rocket Money is one user per account with no shared budget — [switchwize](https://www.switchwize.com/learn/ynab-vs-monarch-vs-rocket-money)

### Inferences
- Couples-first finance apps struggle commercially (Zeta closed, Honeyfi/Firstly closed, Honeydue basic and ad-funded). Shared finance survives as a feature inside general apps (Monarch, Origin) rather than as a standalone product.
- Joy's shared date jar with a budget combines a sinking fund with shared experience planning. I found no equivalent in any app researched, so it is a genuine differentiator.
- Honeydue's per-transaction chat and Monarch's "yours/mine/ours" shared views are the proven mechanics to borrow.

### Gaps
- Splitwise was not researched within the tool budget (its 2024–2026 Pro features and pricing changes are unverified).
- I could not verify whether Monarch has explicit per-transaction ownership tags ("mine/yours/ours") from a primary source.
