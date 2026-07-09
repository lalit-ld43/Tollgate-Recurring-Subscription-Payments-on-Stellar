# Architecture

## Why two contracts

A single "subscription" contract could hold plans, subscriber state, *and*
scheduling logic all in one place — but that couples "who's due to be
charged" with "how do I move money," and gives no way for one scheduler to
service multiple subscription contracts across different merchants.

```
┌──────────────────┐   is_due() / charge_subscriber()   ┌─────────────────────┐
│ Billing Contract   │ ──────────────────────────────────▶│ Subscription Contract│
│ - registrations    │◀──────────────────────────────────│ - plans               │
│ - run_billing_cycle│         return amount charged      │ - subscriber records  │
└──────────────────┘                                     │ - charge logic         │
                                                            └──────────┬───────────┘
                                                                       │ cross-contract call
                                                                       ▼
                                                            ┌─────────────────────┐
                                                            │  Token Contract       │
                                                            │  (SEP-41 / SAC)        │
                                                            └─────────────────────┘
```

- **Subscription** owns all the money-moving logic and only accepts
  `charge_subscriber` calls from the one Billing contract address it was
  initialized with (`require_auth` on that stored address).
- **Billing** owns scheduling only — it never touches a token contract
  directly. It reads `is_due` (a view call), and if true, calls
  `charge_subscriber` (a state-changing cross-contract call) and reports
  back how many subscriptions were actually charged this sweep.
- This split means a single Billing contract deployment could, in
  principle, service many different Subscription contracts / merchants —
  much like a real payment processor's scheduler is decoupled from any one
  merchant's ledger.

## State machine

**Subscription status:** `Active` → (charge succeeds) → `Active`, or
(charge fails) → `PastDue` → (another failure) → `Cancelled` after 2
consecutive misses (`GRACE_PERIOD_MISSES`). A subscriber can also cancel
directly at any time.

**Timing:** every subscription tracks `next_due_at`, a ledger timestamp.
`is_due` and `charge_subscriber` both compare it against
`env.ledger().timestamp()`, so due-ness is derived purely from on-chain
time — no off-chain clock or oracle is needed.

## Events (the "real-time" layer)

| Event topics | Emitted when |
|---|---|
| `plan, created` | Merchant creates a plan |
| `sub, started` | Subscriber opts in and pays the first cycle |
| `sub, cancel` | Subscriber cancels, or auto-cancel after repeated misses |
| `charge, ok` | A charge succeeds |
| `charge, failed` | A charge fails (insufficient balance, etc.) |
| `billing, registrd` | A subscription is registered with the Billing contract |
| `billing, cycle` | A sweep completes, with the count of subscriptions charged |

The frontend polls `getEvents` on an interval (`useEventStream`), and
additionally drives a **live countdown** (`useCountdown`) per subscription
using the on-chain `next_due_at` timestamp — this is what makes "next
charge in 3d 4h" tick down in real time client-side without any polling.

## Frontend structure

```
frontend/src/
├── lib/
│   ├── config.js               # env-driven contract addresses & network config
│   ├── wallet.js                 # Freighter wallet integration
│   ├── sorobanClient.js          # low-level build/simulate/sign/submit
│   ├── subscriptionActions.js     # typed wrappers per contract method
│   ├── events.js                  # getEvents polling with ledger cursor
│   └── formatEvent.js             # pure event -> label mapping (unit tested)
├── hooks/
│   ├── useWallet.js
│   ├── useEventStream.js
│   └── useCountdown.js            # live "time until next charge" ticker
└── components/                    # presentational, mobile-first with Tailwind
```

## Security notes

- `subscribe`, `cancel`, and `create_plan` all call `require_auth()` on the
  relevant party.
- `charge_subscriber` requires the *Billing contract's own address* to
  authorize — an arbitrary caller cannot trigger a charge outside the
  sweep flow, since Soroban's auth framework verifies the calling
  contract's identity as the "invoker" for contract-to-contract auth.
- `run_billing_cycle` is intentionally open to any caller (like a public
  keeper bot) — it can't move funds itself; it can only ask Subscription
  to check due-ness and charge, and Subscription independently re-verifies
  due-ness before transferring anything ("defense in depth").
- Failed charges degrade gracefully (`PastDue` with a grace period) rather
  than immediately cancelling, so a single transient failure doesn't lose
  the subscriber.
