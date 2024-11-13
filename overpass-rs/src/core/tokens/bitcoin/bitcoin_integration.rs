// ./src/core/tokens/bitcoin/bitcoin_integration.rs
use codec::Encode;
use core::marker::PhantomData;
use frame_support::{
    pallet_prelude::Member,
    traits::{Currency, ExistenceRequirement, Imbalance, WithdrawReasons},
    Parameter,
};
use sp_runtime::{
    traits::MaybeSerializeDeserialize,
    DispatchError, DispatchResult,
};

use super::bitcoin_types::{
    BitcoinNetwork, BitcoinError, BitcoinAccountData, BitcoinTransactionData, BitcoinBalance,
};

/// Configuration trait for Bitcoin integration
pub trait BitcoinConfig: Eq + Clone {
    /// The native currency type
    type NativeCurrency: Currency<Self::AccountId>;
    
    /// The type used to identify accounts
    type AccountId: Parameter + Member + Default;
    
    /// The type used to represent balances
    type Balance: BitcoinBalance + Parameter + Member + Default + Copy + MaybeSerializeDeserialize;
    
    /// The type used to represent transactions
    type Transaction: Parameter + Member + Default;
}

/// Core Bitcoin integration struct
pub struct Bitcoin<T: BitcoinConfig> {
    phantom: PhantomData<T>,
}

impl<T: BitcoinConfig> Bitcoin<T>
where
    T::NativeCurrency: Currency<T::AccountId>,
{
    /// Create a new Bitcoin instance
    pub fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }

    /// Deposit Bitcoin to an account
    pub fn deposit(
        &self,
        network: BitcoinNetwork,
        who: &T::AccountId,
        amount: T::Balance,
    ) -> DispatchResult where <<T as BitcoinConfig>::NativeCurrency as Currency<<T as BitcoinConfig>::AccountId>>::Balance: From<<T as BitcoinConfig>::Balance> {
        match network {
            BitcoinNetwork::Bitcoin => {
                T::NativeCurrency::deposit_creating(who, amount.into());
                Ok(())
            },
            _ => Err(BitcoinError::InvalidNetwork("Unsupported network".into()).into()),
        }
    }

    /// Withdraw Bitcoin from an account
    pub fn withdraw(
        &self,
        network: BitcoinNetwork,
        who: &T::AccountId,
        amount: T::Balance,
        reasons: WithdrawReasons,
    ) -> DispatchResult where <<T as BitcoinConfig>::NativeCurrency as Currency<<T as BitcoinConfig>::AccountId>>::Balance: From<<T as BitcoinConfig>::Balance> {
        match network {
            BitcoinNetwork::Bitcoin => {
                T::NativeCurrency::withdraw(
                    who,
                    amount.into(),
                    reasons,
                    ExistenceRequirement::KeepAlive,
                )?;
                Ok(())
            },
            _ => Err(BitcoinError::InvalidNetwork("Unsupported network".into()).into()),
        }
    }
    /// Transfer Bitcoin between accounts
    pub fn transfer(
        &self,
        network: BitcoinNetwork,
        from: &T::AccountId,
        to: &T::AccountId,
        amount: T::Balance,
    ) -> DispatchResult where <<T as BitcoinConfig>::NativeCurrency as Currency<<T as BitcoinConfig>::AccountId>>::Balance: From<<T as BitcoinConfig>::Balance> {
        match network {
            BitcoinNetwork::Bitcoin => {
                T::NativeCurrency::transfer(
                    from,
                    to,
                    amount.into(),
                    ExistenceRequirement::KeepAlive,
                )
            },
            _ => Err(BitcoinError::InvalidNetwork("Unsupported network".into()).into()),
        }
    }

    /// Slash an account's balance
    pub fn slash(
        &self,
        network: BitcoinNetwork,
        who: &T::AccountId,
        amount: T::Balance,
    ) -> T::Balance 
    where 
        <T as BitcoinConfig>::Balance: From<<<T as BitcoinConfig>::NativeCurrency as Currency<<T as BitcoinConfig>::AccountId>>::Balance>,
        <<T as BitcoinConfig>::NativeCurrency as Currency<<T as BitcoinConfig>::AccountId>>::Balance: From<<T as BitcoinConfig>::Balance>
    {
        match network {
            BitcoinNetwork::Bitcoin => {
                let (imbalance, _) = T::NativeCurrency::slash(who, amount.into());
                T::Balance::from(imbalance.peek())
            },
            _ => T::Balance::zero(),
        }
    }

    /// Check if an account can be slashed
    pub fn can_slash(
        &self,
        network: BitcoinNetwork,
        who: &T::AccountId,
        amount: T::Balance,
    ) -> bool where <<T as BitcoinConfig>::NativeCurrency as Currency<<T as BitcoinConfig>::AccountId>>::Balance: From<<T as BitcoinConfig>::Balance> {
        match network {
            BitcoinNetwork::Bitcoin => {
                T::NativeCurrency::can_slash(who, amount.into())
            },
            _ => false,
        }
    }

    /// Get account balance
    pub fn get_balance(
        &self,
        network: BitcoinNetwork,
        who: &T::AccountId,
    ) -> T::Balance
    where
        T::Balance: From<<<T as BitcoinConfig>::NativeCurrency as Currency<<T as BitcoinConfig>::AccountId>>::Balance>
    {
        match network {
            BitcoinNetwork::Bitcoin => {
                T::Balance::from(T::NativeCurrency::total_balance(who))
            },
            _ => T::Balance::zero(),
        }
    }

    /// Get account data
    pub fn get_account_data(
        &self,
        network: BitcoinNetwork,
        who: &T::AccountId,
    ) -> Option<BitcoinAccountData>
    where
        T::Balance: From<<<T as BitcoinConfig>::NativeCurrency as Currency<<T as BitcoinConfig>::AccountId>>::Balance>
    {
        match network {
            BitcoinNetwork::Bitcoin => {
                Some(BitcoinAccountData {
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
        network: BitcoinNetwork,
        from: &T::AccountId,
        to: &T::AccountId,
        amount: T::Balance,
    ) -> Result<BitcoinTransactionData, BitcoinError> 
    where 
        T::Balance: From<<<T as BitcoinConfig>::NativeCurrency as Currency<<T as BitcoinConfig>::AccountId>>::Balance>
    {
        let sender_data = self.get_account_data(network.clone(), from)
            .ok_or_else(|| BitcoinError::InvalidAccount("Sender account not found".into()))?;

        Ok(BitcoinTransactionData {
            sender: from.encode(),
            recipient: to.encode(),
            amount: amount.try_into().map_err(|_| BitcoinError::InvalidTransaction("Invalid amount".into()))?,
            nonce: sender_data.nonce + 1,
            network: network.clone(),
            timestamp: (std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as u64),
            metadata: Vec::new(),
        })
    }
}

#[cfg(test)]mod tests {
    use super::*;
    use frame_support::traits::{Currency, SignedImbalance};

    #[derive(Clone, Debug, Eq, PartialEq)]
    pub struct TestConfig;

    impl BitcoinConfig for TestConfig {
        type NativeCurrency = TestCurrency;
        type AccountId = u64;
        type Balance = u128;
        type Transaction = Vec<u8>;
    }

    pub struct TestCurrency;

    impl Currency<u64> for TestCurrency {
        type Balance = u128;
        type PositiveImbalance = ();
        type NegativeImbalance = ();

        fn total_balance(_who: &u64) -> Self::Balance { 1000 }
        fn free_balance(_who: &u64) -> Self::Balance { 1000 }
        fn total_issuance() -> Self::Balance { 0 }
        fn minimum_balance() -> Self::Balance { 0 }
        fn burn(_amount: Self::Balance) -> Self::PositiveImbalance { () }
        fn issue(_amount: Self::Balance) -> Self::NegativeImbalance { () }
        
        fn transfer(
            _source: &u64,
            _dest: &u64,
            _value: Self::Balance,
            _existence_requirement: ExistenceRequirement,
        ) -> DispatchResult { Ok(()) }

        fn ensure_can_withdraw(
            _who: &u64,
            _amount: Self::Balance,
            _reasons: WithdrawReasons,
            _new_balance: Self::Balance,
        ) -> DispatchResult { Ok(()) }

        fn deposit_into_existing(
            _who: &u64,
            _amount: Self::Balance
        ) -> Result<Self::PositiveImbalance, DispatchError> { Ok(()) }

        fn withdraw(
            _who: &u64,
            _amount: Self::Balance,
            _reasons: WithdrawReasons,
            _liveness: ExistenceRequirement,
        ) -> Result<Self::NegativeImbalance, DispatchError> { Ok(()) }

        fn deposit_creating(_who: &u64, _amount: Self::Balance) -> Self::PositiveImbalance { () }
        
        fn can_slash(_who: &u64, _value: Self::Balance) -> bool { true }
        
        fn slash(_who: &u64, _amount: Self::Balance) -> (Self::NegativeImbalance, Self::Balance) {
            ((), 0)
        }

        fn make_free_balance_be(_who: &u64, _amount: Self::Balance) -> SignedImbalance<Self::Balance, Self::PositiveImbalance> {
            SignedImbalance::Positive(())
        }   
    }
    #[test]
    fn test_bitcoin_operations() {
        let bitcoin = Bitcoin::<TestConfig>::new();
        let account = 1_u64;
        let amount = 100_u128;

        assert!(bitcoin.deposit(BitcoinNetwork::Bitcoin, &account, amount).is_ok());
        assert!(bitcoin.withdraw(BitcoinNetwork::Bitcoin, &account, amount, WithdrawReasons::all()).is_ok());
        assert_eq!(bitcoin.can_slash(BitcoinNetwork::Bitcoin, &account, amount), true);
    }

    #[test]
    fn test_transaction_creation() {
        let bitcoin = Bitcoin::<TestConfig>::new();
        let from = 1_u64;
        let to = 2_u64;
        let amount = 100_u128;

        let tx_data = bitcoin.create_transaction(
            BitcoinNetwork::Bitcoin,
            &from,
            &to,
            amount,
        );
        assert!(tx_data.is_ok());

        let tx_data = tx_data.unwrap();
        assert_eq!(tx_data.network, BitcoinNetwork::Bitcoin);
        assert_eq!(tx_data.amount, amount as u64);
    }
}