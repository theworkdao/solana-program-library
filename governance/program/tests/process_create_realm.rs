#![cfg(feature = "test-sbf")]

use solana_program_test::*;
use spl_token_2022;

mod program_test;

use {
    crate::program_test::args::RealmSetupArgs,
    program_test::*,
    spl_governance::state::{enums::MintMaxVoterWeightSource, realm::get_realm_address},
    spl_token_2022::state::Mint,
};

#[tokio::test]
async fn test_create_realm() {
    let mut governance_test = GovernanceProgramTest::start_new().await;
    let realm_cookie = governance_test.with_realm().await;
    let realm_account = governance_test.get_realm_account(&realm_cookie.address).await;
    assert_eq!(realm_cookie.account, realm_account);
}

#[tokio::test]
async fn test_create_realm_with_non_default_config() {
    let mut governance_test = GovernanceProgramTest::start_new().await;

    let realm_setup_args = RealmSetupArgs {
        use_council_mint: false,
        community_mint_max_voter_weight_source: MintMaxVoterWeightSource::SupplyFraction(1),
        min_community_weight_to_create_governance: 1,
        ..Default::default()
    };

    let realm_cookie = governance_test.with_realm_using_args(&realm_setup_args).await;
    let realm_account = governance_test.get_realm_account(&realm_cookie.address).await;
    assert_eq!(realm_cookie.account, realm_account);
}

#[tokio::test]
async fn test_create_realm_with_max_voter_weight_absolute_value() {
    let mut governance_test = GovernanceProgramTest::start_new().await;

    let realm_setup_args = RealmSetupArgs {
        community_mint_max_voter_weight_source: MintMaxVoterWeightSource::Absolute(1),
        ..Default::default()
    };

    let realm_cookie = governance_test.with_realm_using_args(&realm_setup_args).await;
    let realm_account = governance_test.get_realm_account(&realm_cookie.address).await;

    assert_eq!(realm_cookie.account, realm_account);
    assert_eq!(
        realm_cookie.account.config.community_mint_max_voter_weight_source,
        MintMaxVoterWeightSource::Absolute(1)
    );
}

#[tokio::test]
async fn test_create_realm_for_existing_pda() {
    let mut governance_test = GovernanceProgramTest::start_new().await;

    let realm_name = format!("Realm #{}", governance_test.next_realm_id).to_string();
    let realm_address = get_realm_address(&governance_test.program_id, &realm_name);
    let rent_exempt = governance_test.bench.rent.minimum_balance(0);
    governance_test.bench.transfer_sol(&realm_address, rent_exempt).await;

    let realm_cookie = governance_test.with_realm().await;
    let realm_account = governance_test.get_realm_account(&realm_cookie.address).await;
    assert_eq!(realm_cookie.account, realm_account);
}

#[tokio::test]
async fn test_create_realm_with_token2022_mint() {
    let mut governance_test = GovernanceProgramTest::start_new().await;

    let realm_setup_args = RealmSetupArgs {
        use_token2022_mint: true,
        community_mint_max_voter_weight_source: MintMaxVoterWeightSource::SupplyFraction(1),
        min_community_weight_to_create_governance: 1,
        ..Default::default()
    };

    let realm_cookie = governance_test.with_realm_using_args(&realm_setup_args).await;
    let realm_account = governance_test.get_realm_account(&realm_cookie.address).await;
    assert_eq!(realm_cookie.account, realm_account);
    // Additional assertions specific to token2022 properties can be added here
}
