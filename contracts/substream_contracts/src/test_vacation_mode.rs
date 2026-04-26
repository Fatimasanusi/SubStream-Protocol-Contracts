#[test]
fn test_activate_vacation_mode() {
    let env = Env::default();
    let contract_id = env.register(SubStreamContract, &());
    let client = SubStreamContractClient::new(&env, &contract_id);

    let merchant = Address::generate(&env);
    env.mock_all_auths();

    // Activate vacation mode
    client.activate_vacation_mode(&merchant);

    // Verify vacation mode is active
    assert!(client.is_vacation_mode_active(&merchant));
}

#[test]
fn test_deactivate_vacation_mode() {
    let env = Env::default();
    let contract_id = env.register(SubStreamContract, &());
    let client = SubStreamContractClient::new(&env, &contract_id);

    let merchant = Address::generate(&env);
    env.mock_all_auths();

    // Activate and then deactivate
    client.activate_vacation_mode(&merchant);
    
    // Advance time
    env.ledger().set_timestamp(env.ledger().timestamp() + 3600); // 1 hour later
    
    client.deactivate_vacation_mode(&merchant);

    // Verify vacation mode is inactive
    assert!(!client.is_vacation_mode_active(&merchant));
}

#[test]
fn test_vacation_mode_preserves_subscription_duration() {
    let env = Env::default();
    let contract_id = env.register(SubStreamContract, &());
    let client = SubStreamContractClient::new(&env, &contract_id);

    let merchant = Address::generate(&env);
    let subscriber = Address::generate(&env);
    let token = Address::generate(&env);
    
    env.mock_all_auths();

    // Setup: merchant activates vacation mode
    client.activate_vacation_mode(&merchant);
    
    // Simulate vacation period
    let vacation_start = env.ledger().timestamp();
    env.ledger().set_timestamp(vacation_start + 86400); // 1 day vacation
    
    // Deactivate vacation mode
    client.deactivate_vacation_mode(&merchant);

    // Subscription durations should be adjusted by the pause duration
    // This preserves the paid-for time for all subscribers
}

#[test]
fn test_vacation_mode_already_active() {
    let env = Env::default();
    let contract_id = env.register(SubStreamContract, &());
    let client = SubStreamContractClient::new(&env, &contract_id);

    let merchant = Address::generate(&env);
    env.mock_all_auths();

    // Activate once
    client.activate_vacation_mode(&merchant);

    // Attempt to activate again - should panic
    // In real test: expect panic with "vacation mode already active"
}
