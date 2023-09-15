use drift::state::spot_market::{SpotMarket};
use solana_program::account_info::AccountInfo;
use anchor_lang::Discriminator;
use anchor_lang::prelude::AccountLoader;
use drift::state::user::User;
use solana_program::pubkey::Pubkey;
use arrayref::array_ref;

pub struct UserTokenAmount {
    /// drift user account key
    pub user: Pubkey,
    /// user wallet public key
    pub authority: Pubkey,
    /// positive for deposit, negative for borrow
    pub token_amount: i128,
}

pub fn get_token_amounts(account_infos: Vec<&AccountInfo>, spot_market_index: u16) -> Vec<UserTokenAmount> {
    let mut spot_market : Option<SpotMarket> = None;
    let mut users = vec![];

    // Find relevant accounts
    for account_info in account_infos {
        let data = account_info.try_borrow_data().unwrap();

        let account_discriminator = array_ref![data, 0, 8];

        if *account_discriminator == SpotMarket::discriminator() {
            let account_loader: AccountLoader<SpotMarket> =
                AccountLoader::try_from(account_info).unwrap();

            let selected_spot_market = account_loader.load().unwrap();
            if selected_spot_market.market_index == spot_market_index {
                spot_market = Some(*selected_spot_market);
            }
        }

        if *account_discriminator == User::discriminator() {
            let user_key = *account_info.key;
            let account_loader: AccountLoader<User> =
                AccountLoader::try_from(account_info).unwrap();

            let selected_user = account_loader.load().unwrap();
            users.push((user_key, *selected_user));
        }
    }

    let spot_market = spot_market.unwrap();

    let mut token_amounts = vec![];
    for (user_key, user) in users {
        let authority = user.authority;
        let spot_position = match user.get_spot_position(spot_market_index) {
            Ok(spot_position) => spot_position,
            Err(_) => continue,
        };

        let token_amount = spot_position.get_signed_token_amount(&spot_market).unwrap();

        token_amounts.push(UserTokenAmount {
            user: user_key,
            authority,
            token_amount,
        });
    }

    token_amounts
}