//! Billing Contract
//!
//! A standalone scheduler/executor. It maintains a registry of (subscriber,
//! plan_id, subscription_contract) triples it's responsible for, and
//! exposes `run_billing_cycle`, which anyone (a cron-style keeper, or a
//! button in the frontend for demo purposes) can call to sweep due
//! subscriptions and trigger charges. This is the core cross-contract call
//! of the project: Billing reads due-ness from Subscription, then calls
//! back into Subscription to actually execute the charge.

#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, Vec,
};
mod subscription_client;
use subscription_client::client::{SubscriptionClient, SubscriptionError};
#[contracttype]
#[derive(Clone, Debug)]
pub struct Registration {
    pub subscriber: Address,
    pub plan_id: u64,
    pub subscription_contract: Address,
}

#[contracttype]
pub enum DataKey {
    Admin,
    Registrations,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum BillingError {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    Unauthorized = 3,
    AlreadyRegistered = 4,
}

#[contract]
pub struct BillingContract;

#[contractimpl]
impl BillingContract {
    pub fn initialize(env: Env, admin: Address) -> Result<(), BillingError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(BillingError::AlreadyInitialized);
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        let registrations: Vec<Registration> = Vec::new(&env);
        env.storage()
            .instance()
            .set(&DataKey::Registrations, &registrations);
        Ok(())
    }

    /// Registers a (subscriber, plan) pair with this Billing contract so it
    /// gets swept on future `run_billing_cycle` calls. In production this
    /// would be called automatically by the Subscription contract's
    /// `subscribe` function; kept as an explicit call here so the demo can
    /// show the registration step happening.
    pub fn register_subscription(
        env: Env,
        subscription_contract: Address,
        subscriber: Address,
        plan_id: u64,
    ) -> Result<(), BillingError> {
        subscriber.require_auth();

        let mut registrations: Vec<Registration> = env
            .storage()
            .instance()
            .get(&DataKey::Registrations)
            .ok_or(BillingError::NotInitialized)?;

        for r in registrations.iter() {
            if r.subscriber == subscriber
                && r.plan_id == plan_id
                && r.subscription_contract == subscription_contract
            {
                return Err(BillingError::AlreadyRegistered);
            }
        }

        registrations.push_back(Registration {
            subscriber: subscriber.clone(),
            plan_id,
            subscription_contract,
        });
        env.storage()
            .instance()
            .set(&DataKey::Registrations, &registrations);

        env.events().publish(
            (symbol_short!("billing"), symbol_short!("registrd"), plan_id),
            subscriber,
        );
        Ok(())
    }

    /// Sweeps all registered subscriptions, charging any that are due.
    /// Returns the number of successful charges triggered this cycle.
    /// Anyone can call this (like a public cron keeper) — the actual
    /// authorization for moving funds lives in the Subscription contract,
    /// which only accepts calls from this Billing contract's address.
    pub fn run_billing_cycle(env: Env) -> Result<u32, BillingError> {
        let registrations: Vec<Registration> = env
            .storage()
            .instance()
            .get(&DataKey::Registrations)
            .ok_or(BillingError::NotInitialized)?;

        let mut charged_count: u32 = 0;

        for reg in registrations.iter() {
            let sub_client =
                SubscriptionClient::new(&env, &reg.subscription_contract);

            let due = sub_client.is_due(&reg.subscriber, &reg.plan_id);
            if !due {
                continue;
            }

            // Cross-contract call: Billing invokes Subscription's charge
            // logic. Requires this contract's own address to authorize,
            // satisfied automatically since we're the caller.
            let amount = sub_client.charge_subscriber(&reg.subscriber, &reg.plan_id);
            if amount > 0 {
                charged_count += 1;
            }
        }

        env.events().publish(
            (symbol_short!("billing"), symbol_short!("cycle")),
            charged_count,
        );

        Ok(charged_count)
    }

    pub fn get_registrations(env: Env) -> Vec<Registration> {
        env.storage()
            .instance()
            .get(&DataKey::Registrations)
            .unwrap_or(Vec::new(&env))
    }
}

mod test;
