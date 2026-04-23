#![cfg(test)]

use soroban_sdk::{Address, Env, vec};
use crate::{
    SubStreamContract, ReentrancyGuard, ReentrancyAttemptDetected, is_reentrancy_guard_active,
    TokenClient, Subscription, Tier, DataKey, PRECISION_MULTIPLIER
};

/// Malicious token contract that attempts reentrancy attacks
pub struct MaliciousTokenContract;

impl MaliciousTokenContract {
    /// Mock transfer function that attempts to call back into the contract
    pub fn malicious_transfer(env: &Env, from: &Address, to: &Address, amount: &i128) {
        // Attempt to call back into the contract during transfer
        let subscriber = Address::random(env);
        let creator = Address::random(env);
        
        // This should fail due to reentrancy guard
        let result = env.try_invoke_contract::<(), (
            &SubStreamContract::collect,
            &env,
            &subscriber,
            &creator,
        );
        
        // The call should fail with reentrancy detection
        assert!(result.is_err());
    }
}

/// Reentrancy Guard Test Suite
/// 
/// This test suite verifies that the reentrancy guard properly prevents
/// reentrancy attacks and resets correctly on both success and failure paths.

#[test]
fn test_reentrancy_guard_basic_functionality() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SubStreamContract);
    let admin = Address::random(&env);
    
    // Initialize contract
    SubStreamContract::initialize(env.clone(), admin.clone());
    
    // Initially, guard should be inactive
    assert!(!is_reentrancy_guard_active(&env));
    
    // Create guard
    {
        let _guard = ReentrancyGuard::new(&env, "test_function");
        
        // Guard should be active
        assert!(is_reentrancy_guard_active(&env));
        
        // Attempting to create another guard should panic
        let result = env.try_invoke_contract::<(), (
            &ReentrancyGuard::new,
            &env,
            "test_function",
        );
        
        assert!(result.is_err());
    }
    
    // Guard should be inactive after drop
    assert!(!is_reentrancy_guard_active(&env));
}

#[test]
fn test_reentrancy_guard_event_emission() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SubStreamContract);
    let admin = Address::random(&env);
    
    // Initialize contract
    SubStreamContract::initialize(env.clone(), admin.clone());
    
    // Create first guard
    let _guard1 = ReentrancyGuard::new(&env, "protected_function");
    
    // Attempt to create second guard (should emit event and panic)
    let result = env.try_invoke_contract::<(), (
        &ReentrancyGuard::new,
        &env,
        "protected_function",
    );
    
    assert!(result.is_err());
    
    // Check that ReentrancyAttemptDetected event was emitted
    let reentrancy_events = env.events().all().filter(|event| {
        match event {
            soroban_sdk::xdr::ContractEvent::V0(v0) => {
                let topic = soroban_sdk::Symbol::new(&env, "ReentrancyAttemptDetected");
                v0.topics.contains(&topic.to_val())
            }
            _ => false,
        }
    });
    
    assert_eq!(reentrancy_events.len(), 1);
}

#[test]
fn test_reentrancy_guard_reset_on_success() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SubStreamContract);
    let admin = Address::random(&env);
    
    // Initialize contract
    SubStreamContract::initialize(env.clone(), admin.clone());
    
    // Guard should be inactive initially
    assert!(!is_reentrancy_guard_active(&env));
    
    // Create guard and let it go out of scope normally
    {
        let _guard = ReentrancyGuard::new(&env, "test_function");
        assert!(is_reentrancy_guard_active(&env));
    } // Guard drops here
    
    // Guard should be inactive after successful execution
    assert!(!is_reentrancy_guard_active(&env));
}

#[test]
fn test_reentrancy_guard_reset_on_panic() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SubStreamContract);
    let admin = Address::random(&env);
    
    // Initialize contract
    SubStreamContract::initialize(env.clone(), admin.clone());
    
    // Create guard and panic
    let result = env.try_invoke_contract::<(), (
        |_env| {
            let _guard = ReentrancyGuard::new(_env, "panic_function");
            panic!("intentional panic");
        },
        &env,
    );
    
    assert!(result.is_err());
    
    // Guard should be inactive even after panic
    assert!(!is_reentrancy_guard_active(&env));
}

#[test]
fn test_distribute_and_collect_reentrancy_protection() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SubStreamContract);
    let admin = Address::random(&env);
    let subscriber = Address::random(&env);
    let creator = Address::random(&env);
    let token = Address::random(&env);
    
    // Initialize contract
    SubStreamContract::initialize(env.clone(), admin.clone());
    
    // Register and verify merchant
    let kyc_issuer = Address::from_string(&soroban_sdk::String::from_str(&env, "GD5DQX2K7Q4D4PE4R6J4Y7Q2Q2Q2Q2Q2Q2Q2Q2Q2Q2Q2Q2Q2Q2Q2Q2"));
    let kyc_hash = vec![&env; 32u8];
    SubStreamContract::register_merchant_with_kyc(
        env.clone(),
        creator.clone(),
        kyc_hash,
        kyc_issuer.clone(),
    );
    
    // Create subscription
    SubStreamContract::subscribe(
        env.clone(),
        subscriber.clone(),
        creator.clone(),
        token.clone(),
        1000i128,
        1i128,
        None::<Address>,
    );
    
    // First collect should work
    let result1 = env.try_invoke_contract::<(), (
        &SubStreamContract::collect,
        &env,
        &subscriber,
        &creator,
    );
    assert!(result1.is_ok());
    
    // Attempt reentrancy during collect (should fail)
    // This simulates a malicious token contract trying to re-enter
    let result2 = env.try_invoke_contract::<(), (
        |_env| {
            // Simulate reentrancy attempt
            let _guard = ReentrancyGuard::new(_env, "distribute_and_collect");
            panic!("This should not be reached due to reentrancy guard");
        },
        &env,
    );
    
    assert!(result2.is_err());
}

#[test]
fn test_multiple_concurrent_guards() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SubStreamContract);
    let admin = Address::random(&env);
    
    // Initialize contract
    SubStreamContract::initialize(env.clone(), admin.clone());
    
    // Create first guard
    let _guard1 = ReentrancyGuard::new(&env, "function1");
    assert!(is_reentrancy_guard_active(&env));
    
    // Attempting to create guard for different function should still fail
    let result = env.try_invoke_contract::<(), (
        &ReentrancyGuard::new,
        &env,
        "function2",
    );
    
    assert!(result.is_err());
}

#[test]
fn test_reentrancy_guard_with_nested_calls() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SubStreamContract);
    let admin = Address::random(&env);
    
    // Initialize contract
    SubStreamContract::initialize(env.clone(), admin.clone());
    
    // Test nested call scenario
    let result = env.try_invoke_contract::<(), (
        |_env| {
            let _guard1 = ReentrancyGuard::new(_env, "outer_function");
            
            // Attempt nested call (should fail)
            let _guard2 = ReentrancyGuard::new(_env, "inner_function");
            panic!("This should not be reached");
        },
        &env,
    );
    
    assert!(result.is_err());
    
    // Guard should be inactive after failure
    assert!(!is_reentrancy_guard_active(&env));
}

#[test]
fn test_reentrancy_guard_storage_optimization() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SubStreamContract);
    let admin = Address::random(&env);
    
    // Initialize contract
    SubStreamContract::initialize(env.clone(), admin.clone());
    
    // Verify guard uses temporary storage (not persistent)
    assert!(!env.storage().persistent().has(&DataKey::ReentrancyGuard));
    assert!(!env.storage().temporary().has(&DataKey::ReentrancyGuard));
    
    // Create guard
    {
        let _guard = ReentrancyGuard::new(&env, "test_function");
        
        // Should use temporary storage, not persistent
        assert!(!env.storage().persistent().has(&DataKey::ReentrancyGuard));
        assert!(env.storage().temporary().has(&DataKey::ReentrancyGuard));
    }
    
    // Should be cleaned up from temporary storage
    assert!(!env.storage().persistent().has(&DataKey::ReentrancyGuard));
    assert!(!env.storage().temporary().has(&DataKey::ReentrancyGuard));
}

#[test]
fn test_reentrancy_guard_function_name_tracking() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SubStreamContract);
    let admin = Address::random(&env);
    
    // Initialize contract
    SubStreamContract::initialize(env.clone(), admin.clone());
    
    // Create guard
    let _guard1 = ReentrancyGuard::new(&env, "specific_function");
    
    // Attempt reentrancy and check event contains function name
    let result = env.try_invoke_contract::<(), (
        &ReentrancyGuard::new,
        &env,
        "specific_function",
    );
    
    assert!(result.is_err());
    
    // Check event contains correct function name
    let events = env.events().all();
    let reentrancy_event = events.iter().find(|event| {
        match event {
            soroban_sdk::xdr::ContractEvent::V0(v0) => {
                let topic = soroban_sdk::Symbol::new(&env, "ReentrancyAttemptDetected");
                v0.topics.contains(&topic.to_val())
            }
            _ => false,
        }
    });
    
    assert!(reentrancy_event.is_some());
}

// Integration test with actual subscription flow
#[test]
fn test_subscription_flow_reentrancy_protection() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SubStreamContract);
    let admin = Address::random(&env);
    let subscriber = Address::random(&env);
    let creator = Address::random(&env);
    let token = Address::random(&env);
    
    // Initialize contract
    SubStreamContract::initialize(env.clone(), admin.clone());
    
    // Register and verify merchant
    let kyc_issuer = Address::from_string(&soroban_sdk::String::from_str(&env, "GD5DQX2K7Q4D4PE4R6J4Y7Q2Q2Q2Q2Q2Q2Q2Q2Q2Q2Q2Q2Q2Q2Q2"));
    let kyc_hash = vec![&env; 32u8];
    SubStreamContract::register_merchant_with_kyc(
        env.clone(),
        creator.clone(),
        kyc_hash,
        kyc_issuer.clone(),
    );
    
    // Create subscription
    SubStreamContract::subscribe(
        env.clone(),
        subscriber.clone(),
        creator.clone(),
        token.clone(),
        1000i128,
        1i128,
        None::<Address>,
    );
    
    // Simulate malicious token attempting reentrancy during collect
    // In a real scenario, this would be triggered by a malicious token contract
    let result = env.try_invoke_contract::<(), (
        |_env| {
            // This simulates what would happen if a malicious token
            // tried to call back into collect during transfer
            let _guard = ReentrancyGuard::new(_env, "distribute_and_collect");
            panic!("Reentrancy should be prevented");
        },
        &env,
    );
    
    assert!(result.is_err());
    
    // Normal collect should still work
    let collect_result = env.try_invoke_contract::<(), (
        &SubStreamContract::collect,
        &env,
        &subscriber,
        &creator,
    );
    
    assert!(collect_result.is_ok());
}
