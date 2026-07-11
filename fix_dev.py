import re

with open('contracts/billing/Cargo.toml', 'r', encoding='utf-8') as f:
    content = f.read()

# Add subscription to dev-dependencies if not there
if 'subscription = { path = "../subscription", features = ["testutils"] }' not in content:
    content = content.replace('[dev-dependencies]\nsoroban-sdk', '[dev-dependencies]\nsubscription = { path = "../subscription", features = ["testutils"] }\nsoroban-sdk')

with open('contracts/billing/Cargo.toml', 'w', encoding='utf-8') as f:
    f.write(content)

# Update billing test to use the subscription crate again
with open('contracts/billing/src/test.rs', 'r', encoding='utf-8') as f:
    test_content = f.read()

# If I changed `use subscription::SubscriptionContractClient` to `use subscription_client::client::SubscriptionClient` I need to revert it for tests or keep it?
# Actually my script update_billing.py replaced it in lib.rs, but test.rs uses `use subscription::SubscriptionContractClient;` directly from the crate `subscription` which is perfectly fine since it's in dev-dependencies!

