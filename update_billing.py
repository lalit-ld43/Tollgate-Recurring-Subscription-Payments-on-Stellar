import re

with open('contracts/billing/src/lib.rs', 'r', encoding='utf-8') as f:
    content = f.read()

content = content.replace("use subscription::{SubscriptionContractClient, SubscriptionError};", "mod subscription_client;\nuse subscription_client::client::{SubscriptionClient, SubscriptionError};")
content = content.replace("SubscriptionContractClient::new", "SubscriptionClient::new")

with open('contracts/billing/src/lib.rs', 'w', encoding='utf-8') as f:
    f.write(content)

with open('contracts/billing/Cargo.toml', 'r', encoding='utf-8') as f:
    cargo = f.read()

cargo = re.sub(r'subscription = \{ path = "\.\./subscription" \}', '', cargo)
with open('contracts/billing/Cargo.toml', 'w', encoding='utf-8') as f:
    f.write(cargo)
