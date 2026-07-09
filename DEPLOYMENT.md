# Deployment Guide

Deploying both contracts to Stellar testnet and wiring up the frontend takes
about 15–20 minutes. Do this yourself — the addresses and transaction hash
the competition checklist asks for need to come from a real deployment.

## 0. Prerequisites

```bash
rustup target add wasm32-unknown-unknown
cargo install --locked soroban-cli --features opt
node --version   # 20+
```

Install the [Freighter wallet extension](https://freighter.app) and switch
its network to **Testnet**.

## 1. Create and fund a deployer identity

```bash
soroban keys generate deployer --network testnet
soroban keys fund deployer --network testnet
soroban keys address deployer
```

## 2. Build the contracts

```bash
cargo build --target wasm32-unknown-unknown --release
```

Produces:
- `target/wasm32-unknown-unknown/release/subscription.wasm`
- `target/wasm32-unknown-unknown/release/billing.wasm`

## 3. Deploy the Billing contract

```bash
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/billing.wasm \
  --source deployer \
  --network testnet
```

Save the printed ID as `BILLING_ID`, then initialize:

```bash
soroban contract invoke \
  --id $BILLING_ID \
  --source deployer \
  --network testnet \
  -- initialize --admin $(soroban keys address deployer)
```

## 4. Deploy the Subscription contract

```bash
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/subscription.wasm \
  --source deployer \
  --network testnet
```

Save the printed ID as `SUBSCRIPTION_ID`, then initialize it pointing at
the Billing contract:

```bash
soroban contract invoke \
  --id $SUBSCRIPTION_ID \
  --source deployer \
  --network testnet \
  -- initialize \
  --admin $(soroban keys address deployer) \
  --billing_contract $BILLING_ID
```

## 5. Get a test token

```bash
soroban contract asset deploy \
  --asset native \
  --source deployer \
  --network testnet
```

Save the printed address as `TOKEN_ID`.

## 6. Create a plan and subscribe (for your transaction hash)

```bash
soroban keys generate subscriber --network testnet
soroban keys fund subscriber --network testnet

soroban contract invoke \
  --id $SUBSCRIPTION_ID \
  --source deployer \
  --network testnet \
  -- create_plan \
  --merchant $(soroban keys address deployer) \
  --token $TOKEN_ID \
  --price 1000000000 \
  --period_seconds 2592000 \
  --name "Pro Monthly"
```

This returns a `plan_id` (starts at 0). Then subscribe:

```bash
soroban contract invoke \
  --id $SUBSCRIPTION_ID \
  --source subscriber \
  --network testnet \
  -- subscribe \
  --subscriber $(soroban keys address subscriber) \
  --plan_id 0
```

The CLI output includes the transaction hash — this is your submission's
required transaction hash.

Register the subscription with Billing so a sweep can find it:

```bash
soroban contract invoke \
  --id $BILLING_ID \
  --source subscriber \
  --network testnet \
  -- register_subscription \
  --subscription_contract $SUBSCRIPTION_ID \
  --subscriber $(soroban keys address subscriber) \
  --plan_id 0
```

## 7. Configure the frontend

```bash
cd frontend
cp .env.example .env
```

Fill in:

```
VITE_SUBSCRIPTION_CONTRACT_ID=<SUBSCRIPTION_ID>
VITE_BILLING_CONTRACT_ID=<BILLING_ID>
VITE_TOKEN_CONTRACT_ID=<TOKEN_ID>
```

```bash
npm install
npm run dev
```

## 8. Deploy to Vercel

```bash
npm install -g vercel
cd frontend
vercel
```

Set the same `VITE_*` variables in **Settings → Environment Variables**,
then redeploy.

## Checklist mapping

| Item | Where to get it |
|---|---|
| Contract deployment address | `$SUBSCRIPTION_ID` and `$BILLING_ID` from steps 3–4 |
| Transaction hash | Output of the `subscribe` call in step 6 |
| Live demo link | Your Vercel deployment URL |
| CI/CD screenshot | Green checks on the GitHub Actions tab |
| Test output screenshot | `cargo test --workspace` and `npm test` in your terminal |
| Mobile UI screenshot | Vercel URL on your phone, or dev tools device toolbar |
