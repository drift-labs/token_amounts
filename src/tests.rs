use anchor_lang::{Owner, ZeroCopy};
use solana_program::account_info::AccountInfo;
use solana_program::pubkey::Pubkey;
use crate::{get_token_amounts, UserTokenAmount};
use bytes::BytesMut;
use drift::math::constants::{SPOT_BALANCE_PRECISION_U64, SPOT_CUMULATIVE_INTEREST_PRECISION};
use drift::state::spot_market::SpotBalanceType;

#[cfg(test)]
use drift::state::spot_market::SpotMarket;
use drift::state::user::{SpotPosition, User};
use solana_program::example_mocks::solana_sdk::signature::{Keypair, Signer};

#[test]
fn test() {
    let mut jito_spot_market = SpotMarket {
        market_index: 6,
        cumulative_deposit_interest: SPOT_CUMULATIVE_INTEREST_PRECISION,
        cumulative_borrow_interest: SPOT_CUMULATIVE_INTEREST_PRECISION,
        decimals: 9,
        ..SpotMarket::default()
    };
    let jito_spot_market_key = Keypair::new().pubkey();
    crate::create_anchor_account_info!(jito_spot_market, &jito_spot_market_key, SpotMarket, jito_spot_market_account);

    let mut sol_spot_market = SpotMarket {
        market_index: 1,
        cumulative_deposit_interest: SPOT_CUMULATIVE_INTEREST_PRECISION,
        cumulative_borrow_interest: SPOT_CUMULATIVE_INTEREST_PRECISION,
        decimals: 9,
        ..SpotMarket::default()
    };
    let sol_spot_market_key = Keypair::new().pubkey();
    crate::create_anchor_account_info!(sol_spot_market, &sol_spot_market_key, SpotMarket, sol_spot_market_account);

    let first_user_pubkey = Keypair::new().pubkey();
    let first_authority = Keypair::new().pubkey();
    let mut user_spot_positions = [SpotPosition::default(); 8];
    user_spot_positions[1] = SpotPosition {
        market_index: 6,
        balance_type: SpotBalanceType::Deposit,
        scaled_balance: 100 * SPOT_BALANCE_PRECISION_U64,
        ..SpotPosition::default()
    };
    let mut first_user = User {
        authority: first_authority,
        spot_positions: user_spot_positions,
        ..User::default()
    };
    crate::create_anchor_account_info!(first_user, &first_user_pubkey, User, first_user_account_info);

    let second_user_pubkey = Keypair::new().pubkey();
    let second_authority = Keypair::new().pubkey();
    let mut user_spot_positions = [SpotPosition::default(); 8];
    user_spot_positions[1] = SpotPosition {
        market_index: 6,
        balance_type: SpotBalanceType::Borrow,
        scaled_balance: 100 * SPOT_BALANCE_PRECISION_U64,
        ..SpotPosition::default()
    };
    let mut second_user = User {
        authority: second_authority,
        spot_positions: user_spot_positions,
        ..User::default()
    };
    crate::create_anchor_account_info!(second_user, &second_user_pubkey, User, second_user_account_info);

    let third_user_pubkey = Keypair::new().pubkey();
    let third_authority = Keypair::new().pubkey();
    let mut user_spot_positions = [SpotPosition::default(); 8];
    user_spot_positions[1] = SpotPosition {
        market_index: 1,
        balance_type: SpotBalanceType::Deposit,
        scaled_balance: 100 * SPOT_BALANCE_PRECISION_U64,
        ..SpotPosition::default()
    };
    let mut third_user = User {
        authority: third_authority,
        spot_positions: user_spot_positions,
        ..User::default()
    };
    crate::create_anchor_account_info!(third_user, &third_user_pubkey, User, third_user_account_info);

    let token_amounts = get_token_amounts(vec![
        &sol_spot_market_account,
        &jito_spot_market_account,
        &first_user_account_info,
        &second_user_account_info,
        &third_user_account_info,
    ], 6);

    assert_eq!(token_amounts, vec![
        UserTokenAmount {
            user: first_user_pubkey,
            authority: first_authority,
            token_amount:  100 * 10_i128.pow(9),
        },
        UserTokenAmount {
            user: second_user_pubkey,
            authority: second_authority,
            token_amount: -100 * 10_i128.pow(9),
        },
    ]);
}

fn create_account_info<'a>(
    key: &'a Pubkey,
    is_writable: bool,
    lamports: &'a mut u64,
    bytes: &'a mut [u8],
    owner: &'a Pubkey,
) -> AccountInfo<'a> {
    AccountInfo::new(key, false, is_writable, lamports, bytes, owner, false, 0)
}

pub fn get_anchor_account_bytes<T: ZeroCopy + Owner>(account: &mut T) -> BytesMut {
    let mut bytes = BytesMut::new();
    bytes.extend_from_slice(&T::discriminator());
    let data = bytemuck::bytes_of_mut(account);
    bytes.extend_from_slice(data);
    bytes
}

#[macro_export]
macro_rules! create_anchor_account_info {
    ($account:expr, $type:ident, $name: ident) => {
        let key = Pubkey::default();
        let mut lamports = 0;
        let mut data = get_anchor_account_bytes(&mut $account);
        let owner = $type::owner();
        let $name = create_account_info(&key, true, &mut lamports, &mut data[..], &owner);
    };
    ($account:expr, $pubkey:expr, $type:ident, $name: ident) => {
        let mut lamports = 0;
        let mut data = get_anchor_account_bytes(&mut $account);
        let owner = $type::owner();
        let $name = create_account_info($pubkey, true, &mut lamports, &mut data[..], &owner);
    };
}