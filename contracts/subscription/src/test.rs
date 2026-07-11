#![cfg(test)]

use super::*;
use soroban_sdk::testutils::{Address as _, Ledger};
use soroban_sdk::token::StellarAssetClient;
use soroban_sdk::{Address, Env, String};

struct TestSetup {
    env: Env,
    contract_id: Address,
    client: SubscriptionContractClient<'static>,
    token: Address,
    token_admin: StellarAssetClient<'static>,
    admin: Address,
    billing: Address,
    merchant: Address,
    subscriber: Address,
}

fn setup() -> TestSetup {
    let env = Env::default();
    env.mock_all_auths();

    let token_admin_addr = Address::generate(&env);
    let sac = env.register_stellar_asset_contract_v2(token_admin_addr.clone());
    let token = sac.address();
    let token_admin = StellarAssetClient::new(&env, &token);

    let admin = Address::generate(&env);
    let billing = Address::generate(&env);
    let merchant = Address::generate(&env);
    let subscriber = Address::generate(&env);

    token_admin.mint(&subscriber, &100_000);

    let contract_id = env.register(SubscriptionContract, ());
    let client = SubscriptionContractClient::new(&env, &contract_id);
    client.initialize(&admin, &billing);

    TestSetup {
        env,
        contract_id,
        client,
        token,
        token_admin,
        admin,
        billing,
        merchant,
        subscriber,
    }
}

const DAY: u64 = 86_400;

fn advance_time(env: &Env, seconds: u64) {
    env.ledger().with_mut(|l| l.timestamp += seconds);
}

#[test]
fn test_create_plan() {
    let t = setup();
    let plan_id = t.client.create_plan(
        &t.merchant,
        &t.token,
        &1000,
        &(30 * DAY),
        &String::from_str(&t.env, "Pro Monthly"),
    );
    assert_eq!(plan_id, 0);

    let plan = t.client.get_plan(&plan_id);
    assert_eq!(plan.price, 1000);
    assert!(plan.active);
}

#[test]
fn test_create_plan_rejects_zero_price() {
    let t = setup();
    let result = t.client.try_create_plan(
        &t.merchant,
        &t.token,
        &0,
        &(30 * DAY),
        &String::from_str(&t.env, "Free?"),
    );
    assert_eq!(result, Err(Ok(SubscriptionError::InvalidPrice)));
}

#[test]
fn test_create_plan_rejects_zero_period() {
    let t = setup();
    let result = t.client.try_create_plan(
        &t.merchant,
        &t.token,
        &500,
        &0,
        &String::from_str(&t.env, "Instant?"),
    );
    assert_eq!(result, Err(Ok(SubscriptionError::InvalidPeriod)));
}

#[test]
fn test_subscribe_charges_first_payment_immediately() {
    let t = setup();
    let plan_id = t.client.create_plan(
        &t.merchant,
        &t.token,
        &1000,
        &(30 * DAY),
        &String::from_str(&t.env, "Pro Monthly"),
    );

    t.client.subscribe(&t.subscriber, &plan_id, &1_000_000);

    let token_client = token::Client::new(&t.env, &t.token);
    assert_eq!(token_client.balance(&t.merchant), 1000);
    assert_eq!(token_client.balance(&t.subscriber), 99_000);

    let sub = t.client.get_subscription(&t.subscriber, &plan_id);
    assert_eq!(sub.status, SubStatus::Active);
    assert_eq!(sub.missed_charges, 0);
}

#[test]
fn test_cannot_subscribe_twice_while_active() {
    let t = setup();
    let plan_id = t.client.create_plan(
        &t.merchant,
        &t.token,
        &1000,
        &(30 * DAY),
        &String::from_str(&t.env, "Pro Monthly"),
    );
    t.client.subscribe(&t.subscriber, &plan_id, &1_000_000);
    let result = t.client.try_subscribe(&t.subscriber, &plan_id, &1_000_000);
    assert_eq!(result, Err(Ok(SubscriptionError::AlreadySubscribed)));
}

#[test]
fn test_cannot_subscribe_to_inactive_plan() {
    let t = setup();
    let plan_id = t.client.create_plan(
        &t.merchant,
        &t.token,
        &1000,
        &(30 * DAY),
        &String::from_str(&t.env, "Pro Monthly"),
    );
    t.client.deactivate_plan(&t.merchant, &plan_id);
    let result = t.client.try_subscribe(&t.subscriber, &plan_id, &1_000_000);
    assert_eq!(result, Err(Ok(SubscriptionError::PlanInactive)));
}

#[test]
fn test_is_due_false_before_period_elapses() {
    let t = setup();
    let plan_id = t.client.create_plan(
        &t.merchant,
        &t.token,
        &1000,
        &(30 * DAY),
        &String::from_str(&t.env, "Pro Monthly"),
    );
    t.client.subscribe(&t.subscriber, &plan_id, &1_000_000);
    assert!(!t.client.is_due(&t.subscriber, &plan_id));
}

#[test]
fn test_is_due_true_after_period_elapses() {
    let t = setup();
    let plan_id = t.client.create_plan(
        &t.merchant,
        &t.token,
        &1000,
        &(30 * DAY),
        &String::from_str(&t.env, "Pro Monthly"),
    );
    t.client.subscribe(&t.subscriber, &plan_id, &1_000_000);
    advance_time(&t.env, 30 * DAY);
    assert!(t.client.is_due(&t.subscriber, &plan_id));
}

#[test]
fn test_charge_subscriber_requires_billing_contract_auth() {
    let t = setup();
    let plan_id = t.client.create_plan(
        &t.merchant,
        &t.token,
        &1000,
        &(30 * DAY),
        &String::from_str(&t.env, "Pro Monthly"),
    );
    t.client.subscribe(&t.subscriber, &plan_id, &1_000_000);
    advance_time(&t.env, 30 * DAY);

    use soroban_sdk::testutils::MockAuth;
    use soroban_sdk::testutils::MockAuthInvoke;
    use soroban_sdk::IntoVal;

    t.env.mock_auths(&[
        MockAuth {
            address: &t.billing,
            invoke: &MockAuthInvoke {
                contract: &t.contract_id,
                fn_name: "charge_subscriber",
                args: (&t.subscriber, &plan_id).into_val(&t.env),
                sub_invokes: &[],
            },
        },
        MockAuth {
            address: &t.subscriber,
            invoke: &MockAuthInvoke {
                contract: &t.token,
                fn_name: "transfer",
                args: (&t.subscriber, &t.merchant, &1000i128).into_val(&t.env),
                sub_invokes: &[],
            },
        },
    ]);

    let amount = t.client.charge_subscriber(&t.subscriber, &plan_id);
    assert_eq!(amount, 1000);

    let token_client = token::Client::new(&t.env, &t.token);
    assert_eq!(token_client.balance(&t.merchant), 2000);
}

#[test]
fn test_charge_subscriber_rejects_when_not_due() {
    let t = setup();
    let plan_id = t.client.create_plan(
        &t.merchant,
        &t.token,
        &1000,
        &(30 * DAY),
        &String::from_str(&t.env, "Pro Monthly"),
    );
    t.client.subscribe(&t.subscriber, &plan_id, &1_000_000);

    let result = t.client.try_charge_subscriber(&t.subscriber, &plan_id);
    assert_eq!(result, Err(Ok(SubscriptionError::NotDueYet)));
}

#[test]
fn test_cancel_prevents_future_charges() {
    let t = setup();
    let plan_id = t.client.create_plan(
        &t.merchant,
        &t.token,
        &1000,
        &(30 * DAY),
        &String::from_str(&t.env, "Pro Monthly"),
    );
    t.client.subscribe(&t.subscriber, &plan_id, &1_000_000);
    t.client.cancel(&t.subscriber, &plan_id);

    advance_time(&t.env, 30 * DAY);
    let result = t.client.try_charge_subscriber(&t.subscriber, &plan_id);
    assert_eq!(result, Err(Ok(SubscriptionError::SubscriptionCancelled)));
}

#[test]
fn test_failed_charge_marks_past_due_and_increments_misses() {
    let t = setup();
    let plan_id = t.client.create_plan(
        &t.merchant,
        &t.token,
        &1000,
        &(30 * DAY),
        &String::from_str(&t.env, "Pro Monthly"),
    );
    t.client.subscribe(&t.subscriber, &plan_id, &1_000_000);

    // Drain the subscriber's balance so the next charge fails.
    let token_client = token::Client::new(&t.env, &t.token);
    let remaining = token_client.balance(&t.subscriber);
    token_client.transfer(&t.subscriber, &t.admin, &remaining);

    advance_time(&t.env, 30 * DAY);
    let amount = t.client.charge_subscriber(&t.subscriber, &plan_id);
    assert_eq!(amount, 0);

    let sub = t.client.get_subscription(&t.subscriber, &plan_id);
    assert_eq!(sub.status, SubStatus::PastDue);
    assert_eq!(sub.missed_charges, 1);
}

#[test]
fn test_auto_cancel_after_repeated_missed_charges() {
    let t = setup();
    let plan_id = t.client.create_plan(
        &t.merchant,
        &t.token,
        &1000,
        &(30 * DAY),
        &String::from_str(&t.env, "Pro Monthly"),
    );
    t.client.subscribe(&t.subscriber, &plan_id, &1_000_000);

    let token_client = token::Client::new(&t.env, &t.token);
    let remaining = token_client.balance(&t.subscriber);
    token_client.transfer(&t.subscriber, &t.admin, &remaining);

    for _ in 0..99 {
        advance_time(&t.env, 30 * DAY);
        t.client.charge_subscriber(&t.subscriber, &plan_id);
    }
    
    advance_time(&t.env, 30 * DAY);
    t.client.charge_subscriber(&t.subscriber, &plan_id); // miss 100 -> Cancelled

    let sub = t.client.get_subscription(&t.subscriber, &plan_id);
    assert_eq!(sub.status, SubStatus::Cancelled);
    assert_eq!(sub.missed_charges, 100);
}

#[test]
fn test_get_plan_not_found() {
    let t = setup();
    let result = t.client.try_get_plan(&999);
    assert_eq!(result, Err(Ok(SubscriptionError::PlanNotFound)));
}

#[test]
fn test_get_subscription_not_found() {
    let t = setup();
    let result = t.client.try_get_subscription(&t.subscriber, &999);
    assert_eq!(result, Err(Ok(SubscriptionError::SubscriptionNotFound)));
}

#[test]
fn test_only_merchant_can_deactivate_plan() {
    let t = setup();
    let plan_id = t.client.create_plan(
        &t.merchant,
        &t.token,
        &1000,
        &(30 * DAY),
        &String::from_str(&t.env, "Pro Monthly"),
    );
    let impostor = Address::generate(&t.env);
    let result = t.client.try_deactivate_plan(&impostor, &plan_id);
    assert_eq!(result, Err(Ok(SubscriptionError::Unauthorized)));
}

#[test]
fn test_cannot_initialize_twice() {
    let t = setup();
    let result = t.client.try_initialize(&t.admin, &t.billing);
    assert_eq!(result, Err(Ok(SubscriptionError::AlreadyInitialized)));
}

#[test]
fn test_resubscribe_after_cancel_is_allowed() {
    let t = setup();
    let plan_id = t.client.create_plan(
        &t.merchant,
        &t.token,
        &1000,
        &(30 * DAY),
        &String::from_str(&t.env, "Pro Monthly"),
    );
    t.client.subscribe(&t.subscriber, &plan_id, &1_000_000);
    t.client.cancel(&t.subscriber, &plan_id);

    // Should succeed: previous subscription record is Cancelled, not Active.
    t.client.subscribe(&t.subscriber, &plan_id, &1_000_000);
    let sub = t.client.get_subscription(&t.subscriber, &plan_id);
    assert_eq!(sub.status, SubStatus::Active);
}
