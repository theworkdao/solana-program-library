#![cfg(feature = "test-sbf")]
mod program_test;

use {
    crate::program_test::args::RealmSetupArgs,
    program_test::*,
    solana_program_test::*,
    solana_sdk::signature::Keypair,
    spl_governance::{error::GovernanceError, state::enums::VoteThreshold},
    spl_governance_tools::error::GovernanceToolsError,
};

#[tokio::test]
async fn test_create_governance_token_2022() {
    // Arrange
    let mut governance_test = GovernanceProgramTest::start_new().await;

    let realm_cookie = governance_test.with_realm_token_2022().await;

    let token_owner_record_cookie = governance_test
        .with_community_2022_token_deposit(&realm_cookie)
        .await
        .unwrap();

    // Act
    let governance_cookie = governance_test
        .with_governance(&realm_cookie, &token_owner_record_cookie)
        .await
        .unwrap();

    // Assert
    let governance_account = governance_test
        .get_governance_account(&governance_cookie.address)
        .await;

    assert_eq!(governance_cookie.account, governance_account);
}

#[tokio::test]
async fn test_create_governance_token_2022_with_transfer_fees() {
    // Arrange
    let mut governance_test = GovernanceProgramTest::start_new().await;

    let realm_cookie = governance_test.with_realm_token_2022_with_transfer_fees().await;

    let token_owner_record_cookie = governance_test
        .with_community_2022_token_deposit_with_transfer_fees(&realm_cookie)
        .await
        .unwrap();

    // Act
    let governance_cookie = governance_test
        .with_governance(&realm_cookie, &token_owner_record_cookie)
        .await
        .unwrap();

    // Assert
    let governance_account = governance_test
        .get_governance_account(&governance_cookie.address)
        .await;

    assert_eq!(governance_cookie.account, governance_account);
}