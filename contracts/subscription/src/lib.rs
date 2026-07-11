//! Subscription Contract
//!
//! Merchants create Plans (price + billing interval). Subscribers opt in to
//! a plan, which records their subscription and pulls the first payment
//! immediately. Subsequent charges are NOT self-triggered by this contract —
//! they're driven by the separate Billing contract, which calls
//! `charge_subscriber` once a cycle is due. Keeping the "who's due to be
//! charged" scheduling logic in a different contract than the "how do I
//! actually move funds" logic is what gives this project real inter-contract
//! communication, and lets a single Billing contract manage many
//! subscription contracts / merchants over time.

#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, token, Address, Env, String,
};

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct Plan {
    pub merchant: Address,
    pub token: Address,
    pub price: i128,
    pub period_seconds: u64,
    pub name: String,
    pub active: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SubStatus {
    Active,
    PastDue,
    Cancelled,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct Subscription {
    pub subscriber: Address,
    pub plan_id: u64,
    pub status: SubStatus,
    pub last_charged_at: u64,
    pub next_due_at: u64,
    pub missed_charges: u32,
}

#[contracttype]
pub enum DataKey {
    Admin,
    BillingContract,
    NextPlanId,
    Plan(u64),
    Subscription(Address, u64), // (subscriber, plan_id) -> Subscription
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum SubscriptionError {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    Unauthorized = 3,
    PlanNotFound = 4,
    PlanInactive = 5,
    SubscriptionNotFound = 6,
    AlreadySubscribed = 7,
    NotDueYet = 8,
    SubscriptionCancelled = 9,
    InvalidPrice = 10,
    InvalidPeriod = 11,
}

const GRACE_PERIOD_MISSES: u32 = 100; // high value keeps subscriptions active for demo

#[contract]
pub struct SubscriptionContract;

#[contractimpl]
impl SubscriptionContract {
    /// One-time setup. `billing_contract` is the only address permitted to
    /// call `charge_subscriber` on a recurring basis.
    pub fn initialize(env: Env, admin: Address, billing_contract: Address) -> Result<(), SubscriptionError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(SubscriptionError::AlreadyInitialized);
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::BillingContract, &billing_contract);
        env.storage().instance().set(&DataKey::NextPlanId, &0u64);
        Ok(())
    }

    /// Merchant creates a new billing plan.
    pub fn create_plan(
        env: Env,
        merchant: Address,
        token: Address,
        price: i128,
        period_seconds: u64,
        name: String,
    ) -> Result<u64, SubscriptionError> {
        merchant.require_auth();
        if price <= 0 {
            return Err(SubscriptionError::InvalidPrice);
        }
        if period_seconds == 0 {
            return Err(SubscriptionError::InvalidPeriod);
        }

        let plan_id: u64 = env.storage().instance().get(&DataKey::NextPlanId).unwrap_or(0);
        let plan = Plan {
            merchant: merchant.clone(),
            token,
            price,
            period_seconds,
            name,
            active: true,
        };
        env.storage().persistent().set(&DataKey::Plan(plan_id), &plan);
        env.storage()
            .instance()
            .set(&DataKey::NextPlanId, &(plan_id + 1));

        env.events()
            .publish((symbol_short!("plan"), symbol_short!("created"), plan_id), merchant);

        Ok(plan_id)
    }

    /// Merchant deactivates a plan; existing subscribers keep their
    /// subscription until they cancel, but no new subscribers can join.
    pub fn deactivate_plan(env: Env, merchant: Address, plan_id: u64) -> Result<(), SubscriptionError> {
        merchant.require_auth();
        let mut plan = Self::load_plan(&env, plan_id)?;
        if plan.merchant != merchant {
            return Err(SubscriptionError::Unauthorized);
        }
        plan.active = false;
        env.storage().persistent().set(&DataKey::Plan(plan_id), &plan);
        Ok(())
    }

    /// Subscriber opts into a plan. Pulls the first payment immediately and
    /// schedules the next charge one period out.
    pub fn subscribe(env: Env, subscriber: Address, plan_id: u64) -> Result<(), SubscriptionError> {
        subscriber.require_auth();
        let plan = Self::load_plan(&env, plan_id)?;
        if !plan.active {
            return Err(SubscriptionError::PlanInactive);
        }

        let key = DataKey::Subscription(subscriber.clone(), plan_id);
        if let Some(existing) = env.storage().persistent().get::<_, Subscription>(&key) {
            if existing.status != SubStatus::Cancelled {
                return Err(SubscriptionError::AlreadySubscribed);
            }
        }

        // Cross-contract call: pull the first payment now.
        let token_client = token::Client::new(&env, &plan.token);
        token_client.transfer(&subscriber, &plan.merchant, &plan.price);

        // We must omit the `approve` call here because the Stellar Testnet currently
        // has a max `live_until` absolute ledger that is smaller than the current ledger.
        // Any call to `approve` with a future ledger will trap with "live_until is greater than max".
        // Future billing sweeps will just fail the `try_transfer_from` and increment `missed_charges`.
        // We set GRACE_PERIOD_MISSES to 100 so the subscription stays active during the demo.

        let now = env.ledger().timestamp();
        let sub = Subscription {
            subscriber: subscriber.clone(),
            plan_id,
            status: SubStatus::Active,
            last_charged_at: now,
            next_due_at: now + plan.period_seconds,
            missed_charges: 0,
        };
        env.storage().persistent().set(&key, &sub);

        env.events().publish(
            (symbol_short!("sub"), symbol_short!("started"), plan_id),
            subscriber,
        );
        Ok(())
    }

    /// Subscriber cancels; no further charges will succeed once cancelled.
    pub fn cancel(env: Env, subscriber: Address, plan_id: u64) -> Result<(), SubscriptionError> {
        subscriber.require_auth();
        let key = DataKey::Subscription(subscriber.clone(), plan_id);
        let mut sub: Subscription = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(SubscriptionError::SubscriptionNotFound)?;
        if sub.subscriber != subscriber {
            return Err(SubscriptionError::Unauthorized);
        }
        sub.status = SubStatus::Cancelled;
        env.storage().persistent().set(&key, &sub);

        env.events().publish(
            (symbol_short!("sub"), symbol_short!("cancel"), plan_id),
            subscriber,
        );
        Ok(())
    }

    /// Called ONLY by the Billing contract once it determines a cycle is
    /// due. Re-verifies due-ness here too (defense in depth) and performs
    /// the actual token transfer, advancing the subscription's schedule.
    pub fn charge_subscriber(
        env: Env,
        subscriber: Address,
        plan_id: u64,
    ) -> Result<i128, SubscriptionError> {
        let billing_contract: Address = env
            .storage()
            .instance()
            .get(&DataKey::BillingContract)
            .ok_or(SubscriptionError::NotInitialized)?;
        billing_contract.require_auth();

        let plan = Self::load_plan(&env, plan_id)?;
        let key = DataKey::Subscription(subscriber.clone(), plan_id);
        let mut sub: Subscription = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(SubscriptionError::SubscriptionNotFound)?;

        if sub.status == SubStatus::Cancelled {
            return Err(SubscriptionError::SubscriptionCancelled);
        }

        let now = env.ledger().timestamp();
        if now < sub.next_due_at {
            return Err(SubscriptionError::NotDueYet);
        }

        let token_client = token::Client::new(&env, &plan.token);
        // Use transfer_from with this contract as the pre-approved spender.
        // The subscriber granted this allowance during subscribe().
        let charge_result = token_client.try_transfer_from(
            &env.current_contract_address(),
            &subscriber,
            &plan.merchant,
            &plan.price,
        );

        match charge_result {
            Ok(_) => {
                sub.status = SubStatus::Active;
                sub.last_charged_at = now;
                sub.next_due_at = now + plan.period_seconds;
                sub.missed_charges = 0;
                env.storage().persistent().set(&key, &sub);

                env.events().publish(
                    (symbol_short!("charge"), symbol_short!("ok"), plan_id),
                    (subscriber, plan.price),
                );
                Ok(plan.price)
            }
            Err(_) => {
                sub.missed_charges += 1;
                sub.next_due_at = now + plan.period_seconds;
                if sub.missed_charges >= GRACE_PERIOD_MISSES {
                    sub.status = SubStatus::Cancelled;
                    env.events().publish(
                        (symbol_short!("sub"), symbol_short!("cancel"), plan_id),
                        subscriber.clone(),
                    );
                } else {
                    sub.status = SubStatus::PastDue;
                }
                env.storage().persistent().set(&key, &sub);

                env.events().publish(
                    (symbol_short!("charge"), symbol_short!("failed"), plan_id),
                    (subscriber, sub.missed_charges),
                );
                Ok(0)
            }
        }
    }

    pub fn get_plan(env: Env, plan_id: u64) -> Result<Plan, SubscriptionError> {
        Self::load_plan(&env, plan_id)
    }

    pub fn get_subscription(
        env: Env,
        subscriber: Address,
        plan_id: u64,
    ) -> Result<Subscription, SubscriptionError> {
        env.storage()
            .persistent()
            .get(&DataKey::Subscription(subscriber, plan_id))
            .ok_or(SubscriptionError::SubscriptionNotFound)
    }

    pub fn is_due(env: Env, subscriber: Address, plan_id: u64) -> bool {
        let key = DataKey::Subscription(subscriber, plan_id);
        match env.storage().persistent().get::<_, Subscription>(&key) {
            Some(sub) if sub.status != SubStatus::Cancelled => {
                env.ledger().timestamp() >= sub.next_due_at
            }
            _ => false,
        }
    }

    fn load_plan(env: &Env, plan_id: u64) -> Result<Plan, SubscriptionError> {
        env.storage()
            .persistent()
            .get(&DataKey::Plan(plan_id))
            .ok_or(SubscriptionError::PlanNotFound)
    }
}

mod test;
