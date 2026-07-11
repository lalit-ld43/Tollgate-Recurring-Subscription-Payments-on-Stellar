#![cfg(test)]

use super::*;
use soroban_sdk::testutils::{Address as _, Ledger};
use soroban_sdk::token::StellarAssetClient;
use soroban_sdk::{Address, Env, String};
use soroban_sdk::token;
use subscription::{SubStatus, SubscriptionContract, SubscriptionContractClient};

const DAY: u64 = 86_400;

struct TestSetup {
    env: Env,
    billing: BillingContractClient<'static>,
    billing_id: Address,
    sub_client: SubscriptionContractClient<'static>,
    sub_id: Address,
    token: Address,
    token_admin: StellarAssetClient<'static>,
    merchant: Address,
    subscriber: Address,
}

fn setup() -> TestSetup {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let merchant = Address::generate(&env);
    let subscriber = Address::generate(&env);

    let token_admin_addr = Address::generate(&env);
    let sac = env.register_stellar_asset_contract_v2(token_admin_addr.clone());
    let token = sac.address();
    let token_admin = StellarAssetClient::new(&env, &token);
    token_admin.mint(&subscriber, &100_000);

    let billing_id = env.register(BillingContract, ());
    let billing = BillingContractClient::new(&env, &billing_id);
    billing.initialize(&admin);

    let sub_id = env.register(SubscriptionContract, ());
    let sub_client = SubscriptionContractClient::new(&env, &sub_id);
    sub_client.initialize(&admin, &billing_id);

    TestSetup {
        env,
        billing,
        billing_id,
        sub_client,
        sub_id,
        token,
        token_admin,
        merchant,
        subscriber,
    }
}

fn advance_time(env: &Env, seconds: u64) {
    env.ledger().with_mut(|l| l.timestamp += seconds);
}

#[test]
fn test_register_subscription() {
    let t = setup();
    let plan_id = t.sub_client.create_plan(
        &t.merchant,
        &t.token,
        &1000,
        &(30 * DAY),
        &String::from_str(&t.env, "Pro Monthly"),
    );
    t.sub_client.subscribe(&t.subscriber, &plan_id);

    t.billing
        .register_subscription(&t.sub_id, &t.subscriber, &plan_id);

    let regs = t.billing.get_registrations();
    assert_eq!(regs.len(), 1);
    assert_eq!(regs.get(0).unwrap().plan_id, plan_id);
}

#[test]
fn test_cannot_register_same_subscription_twice() {
    let t = setup();
    let plan_id = t.sub_client.create_plan(
        &t.merchant,
        &t.token,
        &1000,
        &(30 * DAY),
        &String::from_str(&t.env, "Pro Monthly"),
    );
    t.sub_client.subscribe(&t.subscriber, &plan_id);
    t.billing
        .register_subscription(&t.sub_id, &t.subscriber, &plan_id);

    let result =
        t.billing
            .try_register_subscription(&t.sub_id, &t.subscriber, &plan_id);
    assert_eq!(result, Err(Ok(BillingError::AlreadyRegistered)));
}

#[test]
fn test_billing_cycle_skips_not_due_subscriptions() {
    let t = setup();
    let plan_id = t.sub_client.create_plan(
        &t.merchant,
        &t.token,
        &1000,
        &(30 * DAY),
        &String::from_str(&t.env, "Pro Monthly"),
    );
    t.sub_client.subscribe(&t.subscriber, &plan_id);
    t.billing
        .register_subscription(&t.sub_id, &t.subscriber, &plan_id);

    let charged = t.billing.run_billing_cycle();
    assert_eq!(charged, 0);
}

#[test]
fn test_billing_cycle_charges_due_subscription_via_cross_contract_call() {
    let t = setup();
    let plan_id = t.sub_client.create_plan(
        &t.merchant,
        &t.token,
        &1000,
        &(30 * DAY),
        &String::from_str(&t.env, "Pro Monthly"),
    );
    t.sub_client.subscribe(&t.subscriber, &plan_id);
    t.billing
        .register_subscription(&t.sub_id, &t.subscriber, &plan_id);

    advance_time(&t.env, 30 * DAY);

    use soroban_sdk::testutils::MockAuth;
    use soroban_sdk::testutils::MockAuthInvoke;
    use soroban_sdk::IntoVal;

    t.env.mock_auths(&[
        MockAuth {
            address: &t.subscriber,
            invoke: &MockAuthInvoke {
                contract: &t.token,
                fn_name: "transfer",
                args: (&t.subscriber, &t.merchant, &1000i128).into_val(&t.env),
                sub_invokes: &[],
            },
        }
    ]);

    let charged = t.billing.run_billing_cycle();
    assert_eq!(charged, 1);

    let token_client = token::Client::new(&t.env, &t.token);
    assert_eq!(token_client.balance(&t.merchant), 2000);

    let sub = t.sub_client.get_subscription(&t.subscriber, &plan_id);
    assert_eq!(sub.status, SubStatus::Active);
}

#[test]
fn test_billing_cycle_handles_multiple_registrations() {
    let t = setup();
    let plan_id = t.sub_client.create_plan(
        &t.merchant,
        &t.token,
        &500,
        &(7 * DAY),
        &String::from_str(&t.env, "Weekly"),
    );

    let subscriber2 = Address::generate(&t.env);
    t.token_admin.mint(&subscriber2, &50_000);

    t.sub_client.subscribe(&t.subscriber, &plan_id);
    t.sub_client.subscribe(&subscriber2, &plan_id);
    t.billing
        .register_subscription(&t.sub_id, &t.subscriber, &plan_id);
    t.billing
        .register_subscription(&t.sub_id, &subscriber2, &plan_id);

    advance_time(&t.env, 7 * DAY);

    use soroban_sdk::testutils::MockAuth;
    use soroban_sdk::testutils::MockAuthInvoke;
    use soroban_sdk::IntoVal;

    t.env.mock_auths(&[
        MockAuth {
            address: &t.subscriber,
            invoke: &MockAuthInvoke {
                contract: &t.token,
                fn_name: "transfer",
                args: (&t.subscriber, &t.merchant, &500i128).into_val(&t.env),
                sub_invokes: &[],
            },
        },
        MockAuth {
            address: &subscriber2,
            invoke: &MockAuthInvoke {
                contract: &t.token,
                fn_name: "transfer",
                args: (&subscriber2, &t.merchant, &500i128).into_val(&t.env),
                sub_invokes: &[],
            },
        }
    ]);

    let charged = t.billing.run_billing_cycle();
    assert_eq!(charged, 2);
}

#[test]
fn test_cannot_initialize_twice() {
    let t = setup();
    let admin2 = Address::generate(&t.env);
    let result = t.billing.try_initialize(&admin2);
    assert_eq!(result, Err(Ok(BillingError::AlreadyInitialized)));
}

#[test]
fn test_get_registrations_empty_initially() {
    let t = setup();
    let regs = t.billing.get_registrations();
    assert_eq!(regs.len(), 0);
}
