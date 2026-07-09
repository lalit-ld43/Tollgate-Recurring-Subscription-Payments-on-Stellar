# Tollgate — Recurring Subscription Payments on Stellar

A subscription billing protocol on Soroban. A merchant sets a plan and
price; a subscriber opts in once and pays the first cycle immediately. From
then on, a **separate Billing contract** sweeps due subscriptions and
charges them automatically — no off-chain backend, no cron server, just a
public `run_billing_cycle` call that anyone (a keeper bot, or a button in
this demo) can trigger.

> Built for Stellar Level 3 (Orange Belt) — advanced smart contracts,
> production dApp architecture, CI/CD, and real-time event streaming.

---

## Why this shape of project

Recurring billing is a genuinely different problem from a one-off escrow:
it needs a *scheduler* that's decoupled from the *ledger of who owes what*,
so scheduling logic doesn't have to be reinvented per merchant. This
project models that split for real: the Billing contract calls into the
Subscription contract to check due-ness and trigger charges; the
Subscription contract independently re-verifies due-ness (defense in depth)
before calling out to the token contract.

## How it works

1. **Merchant creates a plan** — price, billing period, name.
2. **Subscriber subscribes** — first payment is pulled immediately, and the
   subscription is registered with the Billing contract.
3. **Time passes.** The subscription's `next_due_at` timestamp (an on-chain
   ledger timestamp) eventually falls in the past.
4. **Anyone runs a billing sweep** (`run_billing_cycle`) — the Billing
   contract checks every registered subscription's due-ness and charges the
   ones that are due, all via cross-contract calls into Subscription.
5. **A failed charge** (e.g. insufficient balance) marks the subscription
   `PastDue` rather than cancelling immediately — after 2 consecutive
   misses, it auto-cancels.

Every step emits an event, streamed live into the frontend's activity feed;
a per-subscription countdown ("next charge in 3d 4h") ticks down client-side
against the on-chain due timestamp. See [ARCHITECTURE.md](./ARCHITECTURE.md)
for the full diagram and event table.

## Tech stack

| Layer | Choice |
|---|---|
| Smart contracts | Rust + Soroban SDK 21 |
| Token standard | SEP-41 (Stellar Asset Contract compatible) |
| Frontend | React 18 + Vite + Tailwind CSS |
| Wallet | Freighter |
| Testing | `cargo test` (contracts), Vitest + Testing Library (frontend) |
| CI/CD | GitHub Actions |
| Hosting | Vercel |

## Project structure

```
tollgate/
├── contracts/
│   ├── subscription/     # plans, subscriber state, charge logic
│   └── billing/           # scheduler that sweeps due subscriptions
├── frontend/               # React app
├── .github/workflows/      # CI/CD pipeline
├── ARCHITECTURE.md          # contract design, state machine, event table
└── DEPLOYMENT.md            # step-by-step testnet deployment guide
```

## Running locally

### Contracts

```bash
rustup target add wasm32-unknown-unknown
cargo test --workspace
cargo build --release --target wasm32-unknown-unknown
```

### Frontend

```bash
cd frontend
npm install
npm test
npm run lint
npm run dev
```

By default the frontend runs with no contract addresses configured and
shows a clear banner saying so — see [DEPLOYMENT.md](./DEPLOYMENT.md) for
deploying your own instance to testnet.

## Testing

- **Contracts:** 25 tests across both contracts — subscription lifecycle
  (subscribe, charge, cancel, resubscribe), missed-charge grace period and
  auto-cancel, billing sweep with multiple registrations, and authorization
  checks (only the Billing contract can trigger a charge, only a merchant
  can deactivate their own plan).
- **Frontend:** 15 tests covering event-label formatting, the live
  countdown hook, and the gate-arm status component.

Run `cargo test --workspace` and `cd frontend && npm test` locally, or
check the **Actions** tab on GitHub for CI runs.

## Deployment

See [DEPLOYMENT.md](./DEPLOYMENT.md) for deploying both contracts to
Stellar testnet, wiring up a test token, and deploying the frontend to
Vercel.

**Live demo:** _add your Vercel URL here after deploying_
**Subscription contract:** _add your deployed contract ID here_
**Billing contract:** _add your deployed contract ID here_
**Example transaction:** _add a transaction hash from a real interaction here_

## Screenshots

_Add screenshots here after deploying:_
- Mobile responsive UI
- CI/CD pipeline passing (GitHub Actions tab)
- Test output showing passing tests (`cargo test --workspace` and `npm test`)

## Demo video

_Add your 1–2 minute demo video link here._

## License

MIT
