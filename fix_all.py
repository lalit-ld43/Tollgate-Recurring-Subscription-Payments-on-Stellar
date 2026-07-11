import re

# 1. Update subscription/src/lib.rs to add PartialEq to Plan and Subscription
with open('contracts/subscription/src/lib.rs', 'r', encoding='utf-8') as f:
    sub_content = f.read()
sub_content = sub_content.replace("#[derive(Clone, Debug)]\npub struct Plan", "#[derive(Clone, Debug, PartialEq)]\npub struct Plan")
sub_content = sub_content.replace("#[derive(Clone, Debug)]\npub struct Subscription", "#[derive(Clone, Debug, PartialEq)]\npub struct Subscription")
with open('contracts/subscription/src/lib.rs', 'w', encoding='utf-8') as f:
    f.write(sub_content)

# 2. Update billing/Cargo.toml to remove subscription dependency
with open('contracts/billing/Cargo.toml', 'r', encoding='utf-8') as f:
    bill_cargo = f.read()
bill_cargo = re.sub(r'subscription = \{ path = "\.\./subscription"[\s\S]*?\}', '', bill_cargo)
with open('contracts/billing/Cargo.toml', 'w', encoding='utf-8') as f:
    f.write(bill_cargo)

# 3. Update billing/src/lib.rs to use local client
with open('contracts/billing/src/lib.rs', 'r', encoding='utf-8') as f:
    bill_content = f.read()
bill_content = bill_content.replace("use subscription::{SubscriptionContractClient, SubscriptionError};", "mod subscription_client;\nuse subscription_client::client::{SubscriptionClient, SubscriptionError};")
bill_content = bill_content.replace("SubscriptionContractClient::new", "SubscriptionClient::new")
with open('contracts/billing/src/lib.rs', 'w', encoding='utf-8') as f:
    f.write(bill_content)

