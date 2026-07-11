pub mod client {
    use soroban_sdk::{contractclient, contracterror, Address, Env};

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
        fn is_due(env: Env, subscriber: Address, plan_id: u64) -> bool;
        fn charge_subscriber(
            env: Env,
            subscriber: Address,
            plan_id: u64,
        ) -> Result<i128, SubscriptionError>;
    }
}
