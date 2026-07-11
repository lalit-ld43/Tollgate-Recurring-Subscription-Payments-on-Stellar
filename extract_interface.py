import re

with open('contracts/subscription/src/lib.rs', 'r', encoding='utf-8') as f:
    content = f.read()

interface_code = """#![no_std]
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
"""

with open('contracts/subscription-interface/src/lib.rs', 'w', encoding='utf-8') as f:
    f.write(interface_code)

# Remove the extracted types from subscription/src/lib.rs
# and add the import
content = re.sub(r'#\[contracttype\]\n#\[derive\(Clone, Debug\)\]\npub struct Plan \{[\s\S]*?\}', '', content)
content = re.sub(r'#\[contracttype\]\n#\[derive\(Clone, Debug, Eq, PartialEq\)\]\npub enum SubStatus \{[\s\S]*?\}', '', content)
content = re.sub(r'#\[contracttype\]\n#\[derive\(Clone, Debug\)\]\npub struct Subscription \{[\s\S]*?\}', '', content)
content = re.sub(r'#\[contracterror\]\n#\[derive\(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord\)\]\n#\[repr\(u32\)\]\npub enum SubscriptionError \{[\s\S]*?\}', '', content)

content = content.replace("use soroban_sdk::{", "use subscription_interface::{Plan, Subscription, SubscriptionError, SubStatus, SubscriptionTrait};\nuse soroban_sdk::{")

# Also change the inherent impl to trait impl
content = content.replace("#[contractimpl]\nimpl SubscriptionContract {", "#[contractimpl]\nimpl SubscriptionTrait for SubscriptionContract {")
content = content.replace("pub fn initialize(", "fn initialize(")
content = content.replace("pub fn create_plan(", "fn create_plan(")
content = content.replace("pub fn deactivate_plan(", "fn deactivate_plan(")
content = content.replace("pub fn subscribe(", "fn subscribe(")
content = content.replace("pub fn cancel(", "fn cancel(")
content = content.replace("pub fn charge_subscriber(", "fn charge_subscriber(")
content = content.replace("pub fn get_plan(", "fn get_plan(")
content = content.replace("pub fn get_subscription(", "fn get_subscription(")
content = content.replace("pub fn is_due(", "fn is_due(")

with open('contracts/subscription/src/lib.rs', 'w', encoding='utf-8') as f:
    f.write(content)

with open('contracts/billing/src/lib.rs', 'r', encoding='utf-8') as f:
    billing_content = f.read()

billing_content = billing_content.replace("use subscription::{SubscriptionContractClient, SubscriptionError};", "use subscription_interface::{SubscriptionClient, SubscriptionError};")
billing_content = billing_content.replace("SubscriptionContractClient::new", "SubscriptionClient::new")

with open('contracts/billing/src/lib.rs', 'w', encoding='utf-8') as f:
    f.write(billing_content)

# Update Cargo.toml
with open('contracts/subscription/Cargo.toml', 'r', encoding='utf-8') as f:
    sub_cargo = f.read()

if 'subscription-interface = { path = "../subscription-interface" }' not in sub_cargo:
    sub_cargo = sub_cargo.replace('[dependencies]', '[dependencies]\nsubscription-interface = { path = "../subscription-interface" }')
with open('contracts/subscription/Cargo.toml', 'w', encoding='utf-8') as f:
    f.write(sub_cargo)

with open('contracts/billing/Cargo.toml', 'r', encoding='utf-8') as f:
    bill_cargo = f.read()

bill_cargo = re.sub(r'subscription = \{ path = "\.\./subscription" \}', 'subscription-interface = { path = "../subscription-interface" }', bill_cargo)
with open('contracts/billing/Cargo.toml', 'w', encoding='utf-8') as f:
    f.write(bill_cargo)

