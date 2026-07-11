#![no_std]
use soroban_sdk::{contracterror, contracttype, contractclient, Address, Env, String};

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

#[contractclient(name = "SubscriptionClient")]
pub trait SubscriptionTrait {
    fn initialize(env: Env, admin: Address, billing_contract: Address) -> Result<(), SubscriptionError>;
    fn create_plan(
        env: Env,
        merchant: Address,
        token: Address,
        price: i128,
        period_seconds: u64,
        name: String,
    ) -> Result<u64, SubscriptionError>;
    fn deactivate_plan(env: Env, merchant: Address, plan_id: u64) -> Result<(), SubscriptionError>;
    fn subscribe(env: Env, subscriber: Address, plan_id: u64) -> Result<(), SubscriptionError>;
    fn cancel(env: Env, subscriber: Address, plan_id: u64) -> Result<(), SubscriptionError>;
    fn charge_subscriber(
        env: Env,
        subscriber: Address,
        plan_id: u64,
    ) -> Result<i128, SubscriptionError>;
    fn get_plan(env: Env, plan_id: u64) -> Result<Plan, SubscriptionError>;
    fn get_subscription(
        env: Env,
        subscriber: Address,
        plan_id: u64,
    ) -> Result<Subscription, SubscriptionError>;
    fn is_due(env: Env, subscriber: Address, plan_id: u64) -> bool;
}
