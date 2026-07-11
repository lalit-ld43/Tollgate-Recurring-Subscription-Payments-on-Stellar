import re

with open('contracts/subscription/src/lib.rs', 'r', encoding='utf-8') as f:
    content = f.read()

# Replace DataKey and Plan, SubscriptionError, Subscription with importing them
import_str = """use subscription_interface::{Plan, Subscription, SubscriptionError, SubscriptionTrait};
"""

# Remove SubscriptionError enum
content = re.sub(r'#\[contracterror\][\s\S]*?pub enum SubscriptionError \{[\s\S]*?\}', '', content)

# Remove Plan struct
content = re.sub(r'#\[contracttype\][\s\S]*?pub struct Plan \{[\s\S]*?\}', '', content)

# Remove Subscription struct
content = re.sub(r'#\[contracttype\][\s\S]*?pub struct Subscription \{[\s\S]*?\}', '', content)

# Remove the previously injected trait if it exists
content = re.sub(r'#\[cfg_attr\(feature = "library"[\s\S]*?pub trait SubscriptionTrait \{[\s\S]*?\}[\s\n]*#\[cfg\(not\(feature = "library"\)\)\]', '', content)

# change impl SubscriptionTrait for SubscriptionContract back to impl SubscriptionContract if it was changed
content = content.replace("impl SubscriptionTrait for SubscriptionContract {", "impl SubscriptionContract {")

# insert import right after first imports
content = content.replace("use soroban_sdk::{", "use subscription_interface::{Plan, Subscription, SubscriptionError, SubscriptionTrait};\nuse soroban_sdk::{")

# Now change #[contractimpl] to implement the trait!
# Wait, if we implement the trait, #[contractimpl] generates WASM exports.
# In soroban, when you implement a trait:
# #[contractimpl]
# impl SubscriptionTrait for SubscriptionContract { ... }
content = content.replace("impl SubscriptionContract {", "impl SubscriptionTrait for SubscriptionContract {")

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

sub_cargo = sub_cargo.replace('library = []', '')
if 'subscription-interface = { path = "../subscription-interface" }' not in sub_cargo:
    sub_cargo = sub_cargo.replace('[dependencies]', '[dependencies]\nsubscription-interface = { path = "../subscription-interface" }')
with open('contracts/subscription/Cargo.toml', 'w', encoding='utf-8') as f:
    f.write(sub_cargo)

with open('contracts/billing/Cargo.toml', 'r', encoding='utf-8') as f:
    bill_cargo = f.read()

bill_cargo = re.sub(r'subscription = \{ path = "\.\./subscription"[\s\S]*?\}', 'subscription-interface = { path = "../subscription-interface" }', bill_cargo)
with open('contracts/billing/Cargo.toml', 'w', encoding='utf-8') as f:
    f.write(bill_cargo)

