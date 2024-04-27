//! Realm Account

use {
    crate::{
        error::GovernanceError,
        state::{
            enums::{GovernanceAccountType, MintMaxVoterWeightSource},
            legacy::RealmV1,
            realm_config::{get_realm_config_data_for_realm, GoverningTokenType},
            token_owner_record::get_token_owner_record_data_for_realm,
            vote_record::VoteKind,
        },
        tools::structs::SetConfigItemActionType,
        PROGRAM_AUTHORITY_SEED,
    },
    borsh::{io::Write, BorshDeserialize, BorshSchema, BorshSerialize},
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        program_error::ProgramError,
        program_pack::IsInitialized,
        pubkey::Pubkey,
    },
    spl_governance_addin_api::voter_weight::VoterWeightAction,
    spl_governance_tools::account::{
        assert_is_valid_account_of_types, get_account_data, get_account_type, AccountMaxSize,
    },
    std::slice::Iter,
};

/// SetRealmConfigItem instruction arguments to set a single Realm config item
/// Note: In the current version only TokenOwnerRecordLockAuthority is supported
/// Eventually all Realm config items should be supported for single config item
/// change
#[derive(Clone, Debug, PartialEq, Eq, BorshDeserialize, BorshSerialize, BorshSchema)]
pub enum SetRealmConfigItemArgs {
    /// Set TokenOwnerRecord lock authority
    TokenOwnerRecordLockAuthority {
        /// Action indicating whether to add or remove the lock authority
        #[allow(dead_code)]
        action: SetConfigItemActionType,
        /// Mint of the governing token the lock authority is for
        #[allow(dead_code)]
        governing_token_mint: Pubkey,
        /// Authority to change
        #[allow(dead_code)]
        authority: Pubkey,
    },
}

/// Realm Config instruction args
#[derive(Clone, Debug, PartialEq, Eq, BorshDeserialize, BorshSerialize, BorshSchema)]
pub struct RealmConfigArgs {
    /// Indicates whether council_mint should be used
    /// If yes then council_mint account must also be passed to the instruction
    pub use_council_mint: bool,

    /// Min number of community tokens required to create a governance
    pub min_community_weight_to_create_governance: u64,

    /// The source used for community mint max vote weight source
    pub community_mint_max_voter_weight_source: MintMaxVoterWeightSource,

    /// Community token config args
    pub community_token_config_args: GoverningTokenConfigArgs,

    /// Council token config args
    pub council_token_config_args: GoverningTokenConfigArgs,
}

/// Realm Config instruction args
#[derive(Clone, Debug, PartialEq, Eq, BorshDeserialize, BorshSerialize, BorshSchema, Default)]
pub struct GoverningTokenConfigArgs {
    /// Indicates whether an external addin program should be used to provide
    /// voters weights If yes then the voters weight program account must be
    /// passed to the instruction
    pub use_voter_weight_addin: bool,

    /// Indicates whether an external addin program should be used to provide
    /// max voters weight for the token If yes then the max voter weight
    /// program account must be passed to the instruction
    pub use_max_voter_weight_addin: bool,

    /// Governing token type defines how the token is used for governance
    pub token_type: GoverningTokenType,
}

/// Realm Config instruction args with account parameters
#[derive(Clone, Debug, PartialEq, Eq, BorshDeserialize, BorshSerialize, BorshSchema, Default)]
pub struct GoverningTokenConfigAccountArgs {
    /// Specifies an external plugin program which should be used to provide
    /// voters weights for the given governing token
    pub voter_weight_addin: Option<Pubkey>,

    /// Specifies an external an external plugin program should be used to
    /// provide max voters weight for the given governing token
    pub max_voter_weight_addin: Option<Pubkey>,

    /// Governing token type defines how the token is used for governance power
    pub token_type: GoverningTokenType,
}

/// SetRealmAuthority instruction action
#[derive(Clone, Debug, PartialEq, Eq, BorshDeserialize, BorshSerialize, BorshSchema)]
pub enum SetRealmAuthorityAction {
    /// Sets realm authority without any checks
    /// Uncheck option allows to set the realm authority to non governance
    /// accounts
    SetUnchecked,

    /// Sets realm authority and checks the new new authority is one of the
    /// realm's governances
    // Note: This is not a security feature because governance creation is only
    // gated with min_community_weight_to_create_governance.
    // The check is done to prevent scenarios where the authority could be
    // accidentally set to a wrong or none existing account.
    SetChecked,

    /// Removes realm authority
    Remove,
}

/// Realm Config defining Realm parameters.
#[derive(Clone, Debug, PartialEq, Eq, BorshDeserialize, BorshSerialize, BorshSchema)]
pub struct RealmConfig {
    /// Legacy field introduced and used in V2 as
    /// use_community_voter_weight_addin: bool If the field is going to be
    /// reused in future version it must be taken under consideration
    /// that for some Realms it might be already set to 1
    pub legacy1: u8,

    /// Legacy field introduced and used in V2 as
    /// use_max_community_voter_weight_addin: bool If the field is going to
    /// be reused in future version it must be taken under consideration
    /// that for some Realms it might be already set to 1
    pub legacy2: u8,

    /// Reserved space for future versions
    pub reserved: [u8; 6],

    /// Min number of voter's community weight required to create a governance
    pub min_community_weight_to_create_governance: u64,

    /// The source used for community mint max vote weight source
    pub community_mint_max_voter_weight_source: MintMaxVoterWeightSource,

    /// Optional council mint
    pub council_mint: Option<Pubkey>,
}

/// Governance Realm Account
/// Account PDA seeds" ['governance', name]
#[derive(Clone, Debug, PartialEq, Eq, BorshDeserialize, BorshSerialize, BorshSchema)]
pub struct RealmV2 {
    /// Governance account type
    pub account_type: GovernanceAccountType,

    /// Community mint
    pub community_mint: Pubkey,

    /// Configuration of the Realm
    pub config: RealmConfig,

    /// Check if Governing Token is spl_token_22
    pub is_token_2022: bool,
    
    /// Reserved space for future versions
    pub reserved: [u8; 5],

    /// Legacy field not used since program V3 any longer
    /// Note: If the field is going to be reused in future version it must be
    /// taken under consideration that for some Realms it might be already
    /// set to none zero because it was used as voting_proposal_count before
    pub legacy1: u16,

    /// Realm authority. The authority must sign transactions which update the
    /// realm config The authority should be transferred to Realm Governance
    /// to make the Realm self governed through proposals
    pub authority: Option<Pubkey>,

    /// Governance Realm name
    pub name: String,

    /// Reserved space for versions v2 and onwards
    /// Note: V1 accounts must be resized before using this space
    pub reserved_v2: [u8; 128],
}

impl AccountMaxSize for RealmV2 {
    fn get_max_size(&self) -> Option<usize> {
        Some(self.name.len() + 264)
    }
}

impl IsInitialized for RealmV2 {
    fn is_initialized(&self) -> bool {
        self.account_type == GovernanceAccountType::RealmV2
    }
}

/// Checks if the given account type is on of the Realm account types of any
/// version
pub fn is_realm_account_type(account_type: &GovernanceAccountType) -> bool {
    match account_type {
        GovernanceAccountType::RealmV1 | GovernanceAccountType::RealmV2 => true,
        GovernanceAccountType::GovernanceV2
        | GovernanceAccountType::ProgramGovernanceV2
        | GovernanceAccountType::MintGovernanceV2
        | GovernanceAccountType::TokenGovernanceV2
        | GovernanceAccountType::Uninitialized
        | GovernanceAccountType::RealmConfig
        | GovernanceAccountType::TokenOwnerRecordV1
        | GovernanceAccountType::TokenOwnerRecordV2
        | GovernanceAccountType::GovernanceV1
        | GovernanceAccountType::ProgramGovernanceV1
        | GovernanceAccountType::MintGovernanceV1
        | GovernanceAccountType::TokenGovernanceV1
        | GovernanceAccountType::ProposalV1
        | GovernanceAccountType::ProposalV2
        | GovernanceAccountType::SignatoryRecordV1
        | GovernanceAccountType::SignatoryRecordV2
        | GovernanceAccountType::ProposalInstructionV1
        | GovernanceAccountType::ProposalTransactionV2
        | GovernanceAccountType::VoteRecordV1
        | GovernanceAccountType::VoteRecordV2
        | GovernanceAccountType::ProgramMetadata
        | GovernanceAccountType::ProposalDeposit
        | GovernanceAccountType::RequiredSignatory => false,
    }
}

impl RealmV2 {
    /// Asserts the given mint is either Community or Council mint of the Realm
    pub fn assert_is_valid_governing_token_mint(
        &self,
        governing_token_mint: &Pubkey,
    ) -> Result<(), ProgramError> {
        if self.community_mint == *governing_token_mint {
            return Ok(());
        }

        if self.config.council_mint == Some(*governing_token_mint) {
            return Ok(());
        }

        Err(GovernanceError::InvalidGoverningTokenMint.into())
    }

    /// Returns the governing token mint which is used to vote on a proposal
    /// given the provided Vote kind and vote_governing_token_mint
    ///
    /// Veto vote is cast on a proposal configured for the opposite voting
    /// population defined using governing_token_mint Council can veto
    /// Community vote and Community can veto Council assuming the veto for the
    /// voting population is enabled
    ///
    /// For all votes other than Veto (Electorate votes) the
    /// vote_governing_token_mint is the same as Proposal governing_token_mint
    pub fn get_proposal_governing_token_mint_for_vote(
        &self,
        vote_governing_token_mint: &Pubkey,
        vote_kind: &VoteKind,
    ) -> Result<Pubkey, ProgramError> {
        match vote_kind {
            VoteKind::Electorate => Ok(*vote_governing_token_mint),
            VoteKind::Veto => {
                // When Community veto Council proposal then return council_token_mint as the
                // Proposal governing_token_mint
                if self.community_mint == *vote_governing_token_mint {
                    return Ok(self.config.council_mint.unwrap());
                }

                // When Council veto Community proposal then return community_token_mint as the
                // Proposal governing_token_mint
                if self.config.council_mint == Some(*vote_governing_token_mint) {
                    return Ok(self.community_mint);
                }

                Err(GovernanceError::InvalidGoverningTokenMint.into())
            }
        }
    }

    /// Asserts the given governing token mint and holding accounts are valid
    /// for the realm
    pub fn assert_is_valid_governing_token_mint_and_holding(
        &self,
        program_id: &Pubkey,
        realm: &Pubkey,
        governing_token_mint: &Pubkey,
        governing_token_holding: &Pubkey,
    ) -> Result<(), ProgramError> {
        self.assert_is_valid_governing_token_mint(governing_token_mint)?;

        let governing_token_holding_address =
            get_governing_token_holding_address(program_id, realm, governing_token_mint);

        if governing_token_holding_address != *governing_token_holding {
            return Err(GovernanceError::InvalidGoverningTokenHoldingAccount.into());
        }

        Ok(())
    }

    /// Assert the given create authority can create governance
    pub fn assert_create_authority_can_create_governance(
        &self,
        program_id: &Pubkey,
        realm: &Pubkey,
        token_owner_record_info: &AccountInfo,
        create_authority_info: &AccountInfo,
        account_info_iter: &mut Iter<AccountInfo>,
    ) -> Result<(), ProgramError> {
        // Check if create_authority_info is realm_authority and if yes then it must
        // signed the transaction
        if self.authority == Some(*create_authority_info.key) {
            return if !create_authority_info.is_signer {
                Err(GovernanceError::RealmAuthorityMustSign.into())
            } else {
                Ok(())
            };
        }

        // If realm_authority hasn't signed then check if TokenOwner or Delegate signed
        // and can crate governance
        let token_owner_record_data =
            get_token_owner_record_data_for_realm(program_id, token_owner_record_info, realm)?;

        token_owner_record_data.assert_token_owner_or_delegate_is_signer(create_authority_info)?;

        let realm_config_info = next_account_info(account_info_iter)?;
        let realm_config_data =
            get_realm_config_data_for_realm(program_id, realm_config_info, realm)?;

        let voter_weight = token_owner_record_data.resolve_voter_weight(
            account_info_iter,
            self,
            &realm_config_data,
            VoterWeightAction::CreateGovernance,
            realm,
        )?;

        token_owner_record_data.assert_can_create_governance(self, voter_weight)?;

        Ok(())
    }

    /// Serializes account into the target buffer
    pub fn serialize<W: Write>(self, writer: W) -> Result<(), ProgramError> {
        if self.account_type == GovernanceAccountType::RealmV2 {
            borsh::to_writer(writer, &self)?
        } else if self.account_type == GovernanceAccountType::RealmV1 {
            // V1 account can't be resized and we have to translate it back to the original
            // format

            // If reserved_v2 is used it must be individually asses for v1 backward
            // compatibility impact
            if self.reserved_v2 != [0; 128] {
                panic!("Extended data not supported by RealmV1")
            }

            let realm_data_v1 = RealmV1 {
                account_type: self.account_type,
                community_mint: self.community_mint,
                config: self.config,
                is_token_2022: false,
                reserved: self.reserved,
                voting_proposal_count: 0,
                authority: self.authority,
                name: self.name,
            };

            borsh::to_writer(writer, &realm_data_v1)?
        }

        Ok(())
    }
}

/// Checks whether the Realm account exists, is initialized and  owned by
/// Governance program
pub fn assert_is_valid_realm(
    program_id: &Pubkey,
    realm_info: &AccountInfo,
) -> Result<(), ProgramError> {
    assert_is_valid_account_of_types(program_id, realm_info, is_realm_account_type)
}

/// Deserializes account and checks owner program
pub fn get_realm_data(
    program_id: &Pubkey,
    realm_info: &AccountInfo,
) -> Result<RealmV2, ProgramError> {
    let account_type: GovernanceAccountType = get_account_type(program_id, realm_info)?;

    // If the account is V1 version then translate to V2
    if account_type == GovernanceAccountType::RealmV1 {
        let realm_data_v1 = get_account_data::<RealmV1>(program_id, realm_info)?;

        return Ok(RealmV2 {
            account_type,
            community_mint: realm_data_v1.community_mint,
            config: realm_data_v1.config,
            // realm_v1 is always false
            is_token_2022: false,
            reserved: realm_data_v1.reserved,
            legacy1: 0,
            authority: realm_data_v1.authority,
            name: realm_data_v1.name,
            // Add the extra reserved_v2 padding
            reserved_v2: [0; 128],
        });
    }

    get_account_data::<RealmV2>(program_id, realm_info)
}

/// Deserializes account and checks the given authority is Realm's authority
pub fn get_realm_data_for_authority(
    program_id: &Pubkey,
    realm_info: &AccountInfo,
    realm_authority: &Pubkey,
) -> Result<RealmV2, ProgramError> {
    let realm_data = get_realm_data(program_id, realm_info)?;

    if realm_data.authority.is_none() {
        return Err(GovernanceError::RealmHasNoAuthority.into());
    }

    if realm_data.authority.unwrap() != *realm_authority {
        return Err(GovernanceError::InvalidAuthorityForRealm.into());
    }

    Ok(realm_data)
}

/// Deserializes Ream account and asserts the given governing_token_mint is
/// either Community or Council mint of the Realm
pub fn get_realm_data_for_governing_token_mint(
    program_id: &Pubkey,
    realm_info: &AccountInfo,
    governing_token_mint: &Pubkey,
) -> Result<RealmV2, ProgramError> {
    let realm_data = get_realm_data(program_id, realm_info)?;

    realm_data.assert_is_valid_governing_token_mint(governing_token_mint)?;

    Ok(realm_data)
}

/// Returns Realm PDA seeds
pub fn get_realm_address_seeds(name: &str) -> [&[u8]; 2] {
    [PROGRAM_AUTHORITY_SEED, name.as_bytes()]
}

/// Returns Realm PDA address
pub fn get_realm_address(program_id: &Pubkey, name: &str) -> Pubkey {
    Pubkey::find_program_address(&get_realm_address_seeds(name), program_id).0
}

/// Returns Realm Token Holding PDA seeds
pub fn get_governing_token_holding_address_seeds<'a>(
    realm: &'a Pubkey,
    governing_token_mint: &'a Pubkey,
) -> [&'a [u8]; 3] {
    [
        PROGRAM_AUTHORITY_SEED,
        realm.as_ref(),
        governing_token_mint.as_ref(),
    ]
}

/// Returns Realm Token Holding PDA address
pub fn get_governing_token_holding_address(
    program_id: &Pubkey,
    realm: &Pubkey,
    governing_token_mint: &Pubkey,
) -> Pubkey {
    Pubkey::find_program_address(
        &get_governing_token_holding_address_seeds(realm, governing_token_mint),
        program_id,
    )
    .0
}

/// Asserts given realm config args are correct
pub fn assert_valid_realm_config_args(
    realm_config_args: &RealmConfigArgs,
) -> Result<(), ProgramError> {
    match realm_config_args.community_mint_max_voter_weight_source {
        MintMaxVoterWeightSource::SupplyFraction(fraction) => {
            if !(1..=MintMaxVoterWeightSource::SUPPLY_FRACTION_BASE).contains(&fraction) {
                return Err(GovernanceError::InvalidMaxVoterWeightSupplyFraction.into());
            }
        }
        MintMaxVoterWeightSource::Absolute(value) => {
            if value == 0 {
                return Err(GovernanceError::InvalidMaxVoterWeightAbsoluteValue.into());
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod test {

    use {
        super::*, crate::instruction::GovernanceInstruction,
        solana_program::borsh1::try_from_slice_unchecked,
    };

    #[test]
    fn test_max_size() {
        let realm = RealmV2 {
            account_type: GovernanceAccountType::RealmV2,
            community_mint: Pubkey::new_unique(),
            is_token_2022: false,
            reserved: [0; 5],

            authority: Some(Pubkey::new_unique()),
            name: "test-realm".to_string(),
            config: RealmConfig {
                council_mint: Some(Pubkey::new_unique()),
                legacy1: 0,
                legacy2: 0,
                reserved: [0; 6],
                community_mint_max_voter_weight_source: MintMaxVoterWeightSource::Absolute(100),
                min_community_weight_to_create_governance: 10,
            },

            legacy1: 0,
            reserved_v2: [0; 128],
        };

        let size = borsh::to_vec(&realm).unwrap().len();

        assert_eq!(realm.get_max_size(), Some(size));
    }

    /// Realm Config instruction args
    #[derive(Clone, Debug, PartialEq, Eq, BorshDeserialize, BorshSerialize, BorshSchema)]
    pub struct RealmConfigArgsV1 {
        /// Indicates whether council_mint should be used
        /// If yes then council_mint account must also be passed to the
        /// instruction
        pub use_council_mint: bool,

        /// Min number of community tokens required to create a governance
        pub min_community_weight_to_create_governance: u64,

        /// The source used for community mint max vote weight source
        pub community_mint_max_voter_weight_source: MintMaxVoterWeightSource,
    }

    /// Instructions supported by the Governance program
    #[derive(Clone, Debug, PartialEq, Eq, BorshDeserialize, BorshSerialize, BorshSchema)]
    pub enum GovernanceInstructionV1 {
        /// Creates Governance Realm account which aggregates governances for
        /// given Community Mint and optional Council Mint
        CreateRealm {
            #[allow(dead_code)]
            /// UTF-8 encoded Governance Realm name
            name: String,

            #[allow(dead_code)]
            /// Realm config args
            config_args: RealmConfigArgsV1,
        },

        /// Deposits governing tokens (Community or Council) to Governance Realm
        /// and establishes your voter weight to be used for voting within the
        /// Realm
        DepositGoverningTokens {
            /// The amount to deposit into the realm
            #[allow(dead_code)]
            amount: u64,
        },
    }

    #[test]
    fn test_deserialize_v1_create_realm_instruction_from_v2() {
        // Arrange
        let create_realm_ix_v2 = GovernanceInstruction::CreateRealm {
            name: "test-realm".to_string(),
            config_args: RealmConfigArgs {
                use_council_mint: true,
                min_community_weight_to_create_governance: 100,
                community_mint_max_voter_weight_source:
                    MintMaxVoterWeightSource::FULL_SUPPLY_FRACTION,
                community_token_config_args: GoverningTokenConfigArgs::default(),
                council_token_config_args: GoverningTokenConfigArgs::default(),
            },
        };

        let mut create_realm_ix_data = vec![];
        create_realm_ix_v2
            .serialize(&mut create_realm_ix_data)
            .unwrap();

        // Act
        let create_realm_ix_v1: GovernanceInstructionV1 =
            try_from_slice_unchecked(&create_realm_ix_data).unwrap();

        // Assert
        if let GovernanceInstructionV1::CreateRealm { name, config_args } = create_realm_ix_v1 {
            assert_eq!("test-realm", name);
            assert_eq!(
                MintMaxVoterWeightSource::FULL_SUPPLY_FRACTION,
                config_args.community_mint_max_voter_weight_source
            );
        } else {
            panic!("Can't deserialize v1 CreateRealm instruction from v2");
        }
    }
}
