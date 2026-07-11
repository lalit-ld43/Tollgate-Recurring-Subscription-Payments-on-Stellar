import {
  invokeContract,
  readContract,
  addressToScVal,
  stringToScVal,
  i128ToScVal,
  u64ToScVal,
} from './sorobanClient'
import { SUBSCRIPTION_CONTRACT_ID, BILLING_CONTRACT_ID } from './config'

export async function createPlan({ merchant, token, price, periodSeconds, name }) {
  const args = [
    addressToScVal(merchant),
    addressToScVal(token),
    i128ToScVal(price),
    u64ToScVal(periodSeconds),
    stringToScVal(name),
  ]
  return invokeContract({
    contractId: SUBSCRIPTION_CONTRACT_ID,
    method: 'create_plan',
    args,
    sourcePublicKey: merchant,
  })
}

export async function subscribe({ subscriber, planId }) {
  const args = [addressToScVal(subscriber), u64ToScVal(planId)]
  return invokeContract({
    contractId: SUBSCRIPTION_CONTRACT_ID,
    method: 'subscribe',
    args,
    sourcePublicKey: subscriber,
  })
}

export async function cancelSubscription({ subscriber, planId }) {
  const args = [addressToScVal(subscriber), u64ToScVal(planId)]
  return invokeContract({
    contractId: SUBSCRIPTION_CONTRACT_ID,
    method: 'cancel',
    args,
    sourcePublicKey: subscriber,
  })
}

export async function registerWithBilling({ caller, subscriber, planId }) {
  const args = [addressToScVal(SUBSCRIPTION_CONTRACT_ID), addressToScVal(subscriber), u64ToScVal(planId)]
  return invokeContract({
    contractId: BILLING_CONTRACT_ID,
    method: 'register_subscription',
    args,
    sourcePublicKey: caller,
  })
}

export async function runBillingCycle({ caller }) {
  return invokeContract({
    contractId: BILLING_CONTRACT_ID,
    method: 'run_billing_cycle',
    args: [],
    sourcePublicKey: caller,
  })
}

export async function getPlan({ planId, sourcePublicKey }) {
  return readContract({
    contractId: SUBSCRIPTION_CONTRACT_ID,
    method: 'get_plan',
    args: [u64ToScVal(planId)],
    sourcePublicKey,
  })
}

export async function getSubscription({ subscriber, planId, sourcePublicKey }) {
  return readContract({
    contractId: SUBSCRIPTION_CONTRACT_ID,
    method: 'get_subscription',
    args: [addressToScVal(subscriber), u64ToScVal(planId)],
    sourcePublicKey,
  })
}

export function normalizePlan(planId, raw) {
  if (!raw) return null
  return {
    id: planId,
    merchant: raw.merchant,
    token: raw.token,
    price: (Number(raw.price?.toString?.() ?? String(raw.price)) / 10000000).toString(),
    periodSeconds: Number(raw.period_seconds),
    name: raw.name,
    active: raw.active,
  }
}

export function normalizeSubscription(raw) {
  if (!raw) return null
  return {
    subscriber: raw.subscriber,
    planId: Number(raw.plan_id),
    status: typeof raw.status === 'string' 
      ? raw.status 
      : Array.isArray(raw.status) 
        ? raw.status[0] 
        : Object.keys(raw.status)[0],
    lastChargedAt: Number(raw.last_charged_at),
    nextDueAt: Number(raw.next_due_at),
    missedCharges: Number(raw.missed_charges),
  }
}
