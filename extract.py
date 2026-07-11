import re

with open('contracts/subscription/src/lib.rs', 'r', encoding='utf-8') as f:
    content = f.read()

trait_def = """
#[cfg_attr(feature = "library", soroban_sdk::contractclient(name = "SubscriptionContractClient"))]
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

#[cfg(not(feature = "library"))]
"""

content = content.replace("#[contractimpl]\nimpl SubscriptionContract {", trait_def + "#[contractimpl]\nimpl SubscriptionTrait for SubscriptionContract {")
content = content.replace("pub fn initialize(", "fn initialize(")
content = content.replace("pub fn create_plan(", "fn create_plan(")
content = content.replace("pub fn deactivate_plan(", "fn deactivate_plan(")
content = content.replace("pub fn subscribe(", "fn subscribe(")
content = content.replace("pub fn cancel(", "fn cancel(")
content = content.replace("pub fn charge_subscriber(", "fn charge_subscriber(")
content = content.replace("pub fn get_plan(", "fn get_plan(")
content = content.replace("pub fn get_subscription(", "fn get_subscription(")
content = content.replace("pub fn is_due(", "fn is_due(")

# Add PartialEq to Plan and Subscription
content = content.replace("#[derive(Clone, Debug)]\npub struct Plan", "#[derive(Clone, Debug, PartialEq)]\npub struct Plan")
content = content.replace("#[derive(Clone, Debug)]\npub struct Subscription", "#[derive(Clone, Debug, PartialEq)]\npub struct Subscription")

with open('contracts/subscription/src/lib.rs', 'w', encoding='utf-8') as f:
    f.write(content)
