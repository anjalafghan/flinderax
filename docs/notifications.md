# Notifications

joy sends two nudges a day to browsers that turned them on (Settings → Notifications):

- **morning, 07:30 IST:** what is due today (bills, dates, birthdays, follow-ups, overdue follow-ups);
- **evening, 19:00 IST:** what is on tomorrow;
- **weekly (`?slot=weekly`, schedule it for Sunday morning):** how many things need a decision ("This week: 3 things need a decision."), a count only. Not scheduled yet: add a weekly cron to `.github/workflows/notify.yml`.

A nudge only counts things by kind ("Today: 2 bills, 1 date."). It never shows names or amounts, so a
lock screen reveals nothing. Nothing is sent when there is nothing to say.

## How it runs

The Fly machine sleeps when idle, so something outside has to wake it. A GitHub Actions schedule
(`.github/workflows/notify.yml`) calls `POST /api/internal/push/run?slot=morning|evening|weekly` with a secret.
That request wakes the machine, which sends the nudges and sleeps again.

## One-time setup

```sh
# 1. make the key pair (the private half stays secret; the app serves the public half itself)
cargo run --release --bin joy -- vapid-keys

# 2. give the server the private key and a token for the scheduled call
fly secrets set VAPID_PRIVATE_KEY=<private> PUSH_RUN_TOKEN=<a long random string> -a joy
#    optional: VAPID_CONTACT=mailto:you@example.com

# 3. give GitHub the same token (repo → Settings → Secrets and variables → Actions)
#    name: JOY_PUSH_TOKEN, value: the PUSH_RUN_TOKEN you chose
```

Then open Settings → Notifications on each device and press _Turn on notifications_. On an iPhone the app
must be added to the Home Screen first and opened from there.

Try it by hand: Actions → Notify → _Run workflow_.

Without `VAPID_PRIVATE_KEY` the Settings card says notifications are not set up, and the scheduled call is
refused. Without `PUSH_RUN_TOKEN` the round endpoint answers 404.
