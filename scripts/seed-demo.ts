// Creates a demo account full of round numbers, through the API, so a demo needs no typing.
//
//   bun scripts/seed-demo.ts                     # http://localhost:3000, user "demo"
//   bun scripts/seed-demo.ts --url https://your-app.fly.dev --user demo2 --pass secret123
//
// Needs a running server. Refuses to touch an account that already has data.

const arg = (name: string, fallback: string) => {
  const i = process.argv.indexOf(`--${name}`);
  return i >= 0 && process.argv[i + 1] ? process.argv[i + 1] : fallback;
};
const BASE = arg("url", "http://localhost:3000").replace(/\/$/, "");
const USER = arg("user", "demo");
const PASS = arg("pass", "demo-pass-123");

let token = "";
async function call<T = any>(path: string, body?: unknown): Promise<T> {
  const res = await fetch(BASE + path, {
    method: body === undefined ? "GET" : "POST",
    headers: { "Content-Type": "application/json", ...(token ? { Authorization: `Bearer ${token}` } : {}) },
    body: body === undefined ? undefined : JSON.stringify(body),
  });
  const text = await res.text();
  if (!res.ok) throw new Error(`${path} -> ${res.status}: ${text}`);
  return (text ? JSON.parse(text) : {}) as T;
}
const rupees = (n: number) => n * 100; // the API speaks paise
const inr = (paise: number) => "₹" + (paise / 100).toLocaleString("en-IN");

// ---- account ----
try {
  // Sign-ups need the server's code: pass the same value in SIGNUP_CODE.
  await call("/api/common/register", { user_name: USER, user_password: PASS, user_role: "user", signup_code: process.env.SIGNUP_CODE ?? "" });
} catch (e) {
  if (!String(e).includes("500")) throw e; // 500 = user name already taken
}
token = (await call<{ access_token: string }>("/api/common/login", { user_name: USER, user_password: PASS })).access_token;
if ((await call<unknown[]>("/api/account/list")).length > 0) {
  console.error(`"${USER}" already has data. Pick another name with --user, or delete this one first.`);
  process.exit(1);
}

// ---- dates relative to today, so the demo works whenever you run it ----
const iso = (d: Date) => d.toISOString().slice(0, 10);
const inMonths = (n: number, day: number) => iso(new Date(Date.UTC(new Date().getUTCFullYear(), new Date().getUTCMonth() + n, day)));

// ---- accounts ----
const hdfc = (await call("/api/account/create", { name: "HDFC Savings", kind: "bank", balance_paise: rupees(120000), role: "spending" })).account_id;
const kotak = (await call("/api/account/create", { name: "Kotak Savings", kind: "bank", balance_paise: rupees(90000), role: "reserve" })).account_id;
await call("/api/account/create", {
  name: "Emergency Fund", kind: "fd", balance_paise: rupees(200000), interest_bps: 700, maturity_date: inMonths(9, 15),
  institution: "SBI", renewal: "renew_all",
});
await call("/api/account/create", {
  name: "Travel FD", kind: "fd", balance_paise: rupees(50000), interest_bps: 650, maturity_date: inMonths(2, 15),
  institution: "Post Office", renewal: "payout", payout_account_id: kotak,
});

// ---- income, fixed payments, budget ----
await call("/api/plan/income/create", { label: "Salary", amount_paise: rupees(100000), day_of_month: 30, account_id: hdfc });
const pay = (label: string, amount: number, day: number, account: string, category: string, extra: object = {}) =>
  call("/api/plan/commitment/create", { label, amount_paise: rupees(amount), day_of_month: day, account_id: account, category, ...extra });
await pay("Rent", 15000, 1, kotak, "house");
await pay("Internet", 1000, 5, kotak, "bills");
await pay("Phone bill (yearly)", 12000, 12, kotak, "bills", { frequency: "yearly", due_month: 12 });
await pay("Gym (yearly)", 6000, 1, kotak, "gym", { frequency: "yearly", due_month: 4 });
await pay("Donation", 1000, 31, kotak, "donation", { bucket: "needs" });
await pay("SIP", 20000, 10, hdfc, "sip");
await pay("Netflix", 500, 1, hdfc, "subscription");
await call("/api/plan/settings/update", {
  spending_paise: rupees(25000), allocation_day: 10, savings_floor_paise: rupees(40000), recovery_months: 3, week_start: 1,
});

// ---- cards ----
const card = (name: string, bank: string, statement: number, due: number, limit: number, color: [number, number, number]) =>
  call("/api/card/create", {
    card_name: name, card_bank: bank, card_primary_color: color, card_secondary_color: color.map((c) => Math.min(255, c + 40)),
    statement_day: statement, due_day: due, credit_limit_paise: rupees(limit), pay_from_account_id: kotak,
  }).then((r) => r.card_id as string);
const travel = await card("Travel Card", "HDFC", 20, 8, 300000, [30, 41, 59]);
const online = await card("Online Card", "ICICI", 1, 20, 100000, [88, 28, 135]);
await call("/api/card/emi/create", { card_id: travel, label: "Phone EMI", monthly_paise: rupees(5000), remaining_principal_paise: rupees(30000), months_left: 6 });
// the four numbers from the bank app (first snapshot = a starting balance, not new spending)
await call("/api/card/snapshot", {
  card_id: travel, total_due_paise: 0, outstanding_paise: rupees(65000),
  unbilled_spent_paise: rupees(40000), unbilled_credit_paise: rupees(5000), starting_balance: true,
});
await call("/api/card/snapshot", {
  card_id: online, total_due_paise: rupees(8000), outstanding_paise: rupees(13000),
  unbilled_spent_paise: rupees(6000), unbilled_credit_paise: rupees(1000), starting_balance: true,
});

// the card bills are already sitting in Kotak, so mark them covered (the "Already in the reserve" button)
const summary = await call("/api/dashboard/summary");
for (const a of summary.actions) {
  const l = a.ledger;
  if (l?.kind === "card_bill") {
    await call("/api/moves/record", { kind: l.kind, key: l.key, amount_paise: l.amount_paise, mode: "already_covered" });
  }
}
const after = await call("/api/dashboard/summary");

console.log(`
Demo account ready on ${BASE}
  username: ${USER}
  password: ${PASS}

The dashboard should now show:  safe to spend ${inr(after.safe_to_spend_paise)}  (${after.status}, ${inr(after.per_week_paise)} a week, until ${after.until})
  Do this:  ${after.actions.map((a: any) => a.text).join("\n            ") || "(nothing)"}

Demo script
  1. Dashboard: the one number, and what to do. Open "See the working" to show where it comes from.
  2. Click "I moved" on the savings top-up. The balances change and the action disappears; the number stays.
  3. Travel Card -> "Update from bank app": raise Unbilled spent 40,000 -> 48,000 and Outstanding 65,000 -> 73,000, save.
     The number drops by exactly 8,000 and a new "move ₹8,000" button appears. Click it.
  4. What-if -> "Recurring deposit": ₹10,000 a month. Savings take the cut vs spending takes the cut, and the chart.
  5. Accounts -> Details on an FD: renewal choices and payout account. Plan: yearly bills are set aside monthly.
`);
