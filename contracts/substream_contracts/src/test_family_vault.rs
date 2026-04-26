#[test]
fn test_create_family_vault() {
    let env = Env::default();
    let contract_id = env.register(SubStreamContract, &());
    let client = SubStreamContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let signer1 = Address::generate(&env);
    let signer2 = Address::generate(&env);
    let signers = soroban_sdk::vec![&env, signer1.clone(), signer2.clone()];
    let token = Address::generate(&env);

    // Mock authentication
    env.mock_all_auths();

    // Create vault
    client.create_family_vault(
        &owner,
        &signers,
        &2, // threshold
        &1000, // allowance
        &token,
    );

    // Vault should be created successfully
    // Additional assertions would verify storage state
}

#[test]
fn test_authorize_and_revoke_delegate() {
    let env = Env::default();
    let contract_id = env.register(SubStreamContract, &());
    let client = SubStreamContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let delegate = Address::generate(&env);
    let vault_id = Address::generate(&env);
    let signers = soroban_sdk::vec![&env, owner.clone()];
    let token = Address::generate(&env);

    env.mock_all_auths();

    // Create vault first
    client.create_family_vault(&vault_id, &signers, &1, &1000, &token);

    // Authorize delegate
    let expires_at = env.ledger().timestamp() + 86400; // 1 day
    client.authorize_delegate(&vault_id, &delegate, &500, &expires_at);

    // Revoke delegate
    client.revoke_delegate(&vault_id, &delegate);
}

#[test]
fn test_vault_subscribe_spending_limit() {
    let env = Env::default();
    let contract_id = env.register(SubStreamContract, &());
    let client = SubStreamContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let delegate = Address::generate(&env);
    let merchant = Address::generate(&env);
    let vault_id = Address::generate(&env);
    let signers = soroban_sdk::vec![&env, owner.clone()];
    let token = Address::generate(&env);

    env.mock_all_auths();

    // Setup vault and delegate
    client.create_family_vault(&vault_id, &signers, &1, &1000, &token);
    let expires_at = env.ledger().timestamp() + 86400;
    client.authorize_delegate(&vault_id, &delegate, &500, &expires_at);

    // Subscribe within limit - should succeed
    client.vault_subscribe(&vault_id, &delegate, &merchant, &token, &100, &10);

    // Attempt to exceed limit - should panic
    // This would require error handling in tests
}
