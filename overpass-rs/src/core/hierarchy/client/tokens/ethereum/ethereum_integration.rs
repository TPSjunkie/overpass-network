// ./src/core/tokens/ethereum/ethereum_integration.rs
use codec::Encode;
use core::marker::PhantomData;
use frame_support::{
    pallet_prelude::Member,
    traits::{Currency, ExistenceRequirement, Imbalance, WithdrawReasons},
    Parameter,
};
use sp_runtime::{
    traits::MaybeSerializeDeserialize, DispatchResult,
};

use super::token_types::{
    TokenNetwork, TokenError, TokenAccountData, TokenTransactionData, TokenBalance,
};

/// Configuration trait for Ethereum integration
pub trait EthereumConfig: Eq + Clone {
    /// The native currency type
    type NativeCurrency: Currency<Self::AccountId>;
    
    /// The type used to identify accounts
    type AccountId: Parameter + Member + Default;
    
    /// The type used to represent balances
    type Balance: TokenBalance + Parameter + Member + Default + Copy + MaybeSerializeDeserialize;
    
    /// The type used to represent transactions
    type Transaction: Parameter + Member + Default;
}

/// Core Ethereum integration struct
pub struct Ethereum<T: EthereumConfig> {
    phantom: PhantomData<T>,
}

impl<T: EthereumConfig> Ethereum<T>
where
    T::NativeCurrency: Currency<T::AccountId>,
{
    /// Create a new Ethereum instance
    pub fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }

    /// Deposit Ethereum to an account
    pub fn deposit(
        &self,
        network: TokenNetwork,
        who: &T::AccountId,
        amount: T::Balance,
    ) -> DispatchResult where <<T as EthereumConfig>::NativeCurrency as Currency<<T as EthereumConfig>::AccountId>>::Balance: From<<T as EthereumConfig>::Balance> {
        match network {
            TokenNetwork::Ethereum => {
                T::NativeCurrency::deposit_creating(who, amount.into());
                Ok(())
            },
            _ => Err(sp_runtime::DispatchError::Other("Unsupported network")),
        }
    }

    /// Withdraw Ethereum from an account
    pub fn withdraw(
        &self,
        network: TokenNetwork,
        who: &T::AccountId,
        amount: T::Balance,
        reasons: WithdrawReasons,
    ) -> DispatchResult where <<T as EthereumConfig>::NativeCurrency as Currency<<T as EthereumConfig>::AccountId>>::Balance: From<<T as EthereumConfig>::Balance> {
        match network {
            TokenNetwork::Ethereum => {
                T::NativeCurrency::withdraw(
                    who,
                    amount.into(),
                    reasons,
                    ExistenceRequirement::KeepAlive,
                )?;
                Ok(())
            },
            _ => Err(sp_runtime::DispatchError::Other("Unsupported network")),
        }
    }
    /// Transfer Ethereum between accounts
    pub fn transfer(
        &self,
        network: TokenNetwork,
        from: &T::AccountId,
        to: &T::AccountId,
        amount: T::Balance,
    ) -> DispatchResult where <<T as EthereumConfig>::NativeCurrency as Currency<<T as EthereumConfig>::AccountId>>::Balance: From<<T as EthereumConfig>::Balance> {
        match network {
            TokenNetwork::Ethereum => {
                T::NativeCurrency::transfer(
                    from,
                    to,
                    amount.into(),
                    ExistenceRequirement::KeepAlive,
                )
            },
            _ => Err(sp_runtime::DispatchError::Other("Unsupported network")),
        }
    }
    /// Slash an account's balance
    pub fn slash(
        &self,
        network: TokenNetwork,
        who: &T::AccountId,
        amount: T::Balance,
    ) -> T::Balance 
    where 
        <T as EthereumConfig>::Balance: From<<<T as EthereumConfig>::NativeCurrency as Currency<<T as EthereumConfig>::AccountId>>::Balance>,
        <<T as EthereumConfig>::NativeCurrency as Currency<<T as EthereumConfig>::AccountId>>::Balance: From<<T as EthereumConfig>::Balance>
    {
        match network {
            TokenNetwork::Ethereum => {
                let (imbalance, _) = T::NativeCurrency::slash(who, amount.into());
                T::Balance::from(imbalance.peek())
            },
            _ => T::Balance::zero(),
        }
    }

    /// Check if an account can be slashed
    pub fn can_slash(
        &self,
        network: TokenNetwork,
        who: &T::AccountId,
        amount: T::Balance,
    ) -> bool where <<T as EthereumConfig>::NativeCurrency as Currency<<T as EthereumConfig>::AccountId>>::Balance: From<<T as EthereumConfig>::Balance> {
        match network {
            TokenNetwork::Ethereum => {
                T::NativeCurrency::can_slash(who, amount.into())
            },
            _ => false,
        }
    }

    /// Get account balance
    pub fn get_balance(
        &self,
        network: TokenNetwork,
        who: &T::AccountId,
    ) -> T::Balance
    where
        T::Balance: From<<<T as EthereumConfig>::NativeCurrency as Currency<<T as EthereumConfig>::AccountId>>::Balance>
    {
        match network {
            TokenNetwork::Ethereum => {
                T::Balance::from(T::NativeCurrency::total_balance(who))
            },
            _ => T::Balance::zero(),
        }
    }

    /// Get account data
    pub fn get_account_data(
        &self,
        network: TokenNetwork,
        who: &T::AccountId,
    ) -> Option<TokenAccountData>
    where
        T::Balance: From<<<T as EthereumConfig>::NativeCurrency as Currency<<T as EthereumConfig>::AccountId>>::Balance>
        {
            match network {
                TokenNetwork::Ethereum => {
                    Some(TokenAccountData {
                        address: who.encode(),
                        balance: self.get_balance(network.clone(), who).try_into().ok()?,
                        nonce: T::NativeCurrency::total_issuance().try_into().ok()?,
                        network,
                    })
                },
                _ => None,
            }
        }

        /// Create a new transaction
        pub fn create_transaction(
            &self,
            network: TokenNetwork,
            from: &T::AccountId,
            to: &T::AccountId,
            amount: T::Balance,
        ) -> Result<TokenTransactionData, TokenError> 
        where 
            T::Balance: From<<<T as EthereumConfig>::NativeCurrency as Currency<<T as EthereumConfig>::AccountId>>::Balance>
        {
            let sender_data = self.get_account_data(network.clone(), from)
                .ok_or_else(|| TokenError::InvalidAccount("Sender account not found".into()))?;

            Ok(TokenTransactionData {
                sender: from.encode(),
                recipient: to.encode(),
                amount: amount.try_into().map_err(|_| TokenError::InvalidTransaction("Invalid amount".into()))?,
                nonce: sender_data.nonce + 1,
                network: network.clone(),
                timestamp: (std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as u64),
                metadata: Vec::new(),
            })
        }
    }