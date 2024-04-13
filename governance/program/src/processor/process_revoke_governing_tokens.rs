//! Program state processor

use {
    crate::{
        error::GovernanceError,
        state::{
            realm::{get_realm_data, get_realm_address_seeds},
            realm_config::get_realm_config_data_for_realm,
            token_owner_record::{get_token_owner_record_data_for_realm_and_governing_mint, TokenExtension},
        },
        tools::{
            spl_token::{assert_spl_token_mint_authority_is_signer, burn_spl_tokens_signed},
            token2022::{assert_token2022_mint_authority_is_signer, burn_token2022_tokens_signed},
        },
    },
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint::ProgramResult,
        pubkey::Pubkey,
    },
};

enum TokenType {
    SPL,
    Token2022,
}

/// Processes RevokeGoverningTokens instruction
pub fn process_revoke_governing_tokens(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
    token_type: TokenType,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let realm_info = next_account_info(account_info_iter)?; // 0
    let governing_token_holding_info = next_account_info(account_info_iter)?; // 1
    let token_owner_record_info = next_account_info(account_info_iter)?; // 2
    let governing_token_mint_info = next_account_info(account_info_iter)?; // 3
    let revoke_authority_info = next_account_info(account_info_iter)?; // 4
    let realm_config_info = next_account_info(account_info_iter)?; // 5
    let token_program_info = next_account_info(account_info_iter)?; // 6

    let realm_data = get_realm_data(program_id, realm_info)?;
    let realm_config_data =
        get_realm_config_data_for_realm(program_id, realm_config_info, realm_info.key)?;

    realm_data.assert_is_valid_governing_token_mint_and_holding(
        program_id,
        realm_info.key,
        governing_token_mint_info.key,
        governing_token_holding_info.key,
    )?;
    realm_config_data.assert_can_revoke_governing_token(&realm_data, governing_token_mint_info.key)?;

    let mut token_owner_record_data = get_token_owner_record_data_for_realm_and_governing_mint(
        program_id,
        token_owner_record_info,
        realm_info.key,
        governing_token_mint_info.key,
    )?;

    let token_extension_data = &token_owner_record_info.data.borrow();
    let token_extensions = TokenExtension::deserialize_all(token_extension_data)?;
    let mint_authority = match token_type {
        TokenType::SPL => assert_spl_token_mint_authority_is_signer,
        TokenType::Token2022 => assert_token2022_mint_authority_is_signer,
    };

    if *revoke_authority_info.key == token_owner_record_data.governing_token_owner {
        if !revoke_authority_info.is_signer {
            return Err(GovernanceError::GoverningTokenOwnerMustSign.into());
        }
    } else {
        mint_authority(governing_token_mint_info, revoke_authority_info)?;
    }

    token_owner_record_data.governing_token_deposit_amount = token_owner_record_data
        .governing_token_deposit_amount
        .checked_sub(amount)
        .ok_or(GovernanceError::InvalidRevokeAmount)?;

    token_owner_record_data.serialize(&mut token_owner_record_info.data.borrow_mut()[..])?;

    match token_type {
        TokenType::SPL => burn_spl_tokens_signed(
            governing_token_holding_info,
            governing_token_mint_info,
            realm_info,
            &get_realm_address_seeds(&realm_data.name),
            program_id,
            amount,
            token_program_info,
        )?,
        TokenType::Token2022 => burn_token2022_tokens_signed(
            governing_token_holding_info,
            governing_token_mint_info,
            realm_info,
            &get_realm_address_seeds(&realm_data.name),
            program_id,
            amount,
            token_program_info,
        )?,
    }

    Ok(())
}
