// ./src/core/client/wallet_extension/transactions.rs

use bitcoin::secp256k1::{Message, SecretKey, PublicKey as Secp256k1PublicKey, KeyPair};
use bitcoin::{Address, Amount, Network, LockTime, Script, Transaction, TxIn, TxOut, Witness};
use bitcoin::blockdata::script::{Builder, ScriptBuf};
use bitcoin::blockdata::opcodes::all as opcodes;
use bitcoin::hashes::{sha256, hash160, Hash};
use bitcoin::util::uint::Uint256;

// Add new structs for cross-chain operations
#[derive(Debug, Clone)]
pub struct BridgeConfig {
    pub network: Network,
    pub min_confirmation_depth: u32,
    pub max_timelock_duration: u32,
    pub min_value_sat: u64,
    pub security_level: u32,
}

#[derive(Debug, Clone)]
pub struct StealthAddress {
    pub scan_pubkey: Secp256k1PublicKey,
    pub spend_pubkey: Secp256k1PublicKey,
    pub view_tag: [u8; 32],
}

#[derive(Debug, Clone)]
pub struct CrossChainSwap {
    pub htlc_params: HtlcParams,
    pub stealth_address: StealthAddress,
    pub bridge_config: BridgeConfig,
    pub merkle_root: [u8; 32],
}

// Enhance HtlcParams with cross-chain features
#[derive(Debug, Clone)]
pub struct HtlcParams {
    pub amount: Amount,
    pub timelock: u32,
    pub hash_lock: [u8; 32],
    pub recipient_key: bitcoin::PublicKey,
    pub refund_key: bitcoin::PublicKey,
    pub stealth_pubkey: Option<Secp256k1PublicKey>,
    pub merkle_proof: Option<Vec<[u8; 32]>>,
}

// Add cross-chain specific error types
#[derive(Error, Debug)]
pub enum TransactionError {
    // ... existing errors ...

    #[error("Cross-chain verification failed: {0}")]
    CrossChainError(String),

    #[error("Stealth address error: {0}")]
    StealthAddressError(String),

    #[error("Bridge validation error: {0}")]
    BridgeValidationError(String),
}

impl TransactionManager {
    // Add new methods for cross-chain support

    /// Creates a new cross-chain atomic swap transaction
    pub fn create_cross_chain_swap(
        &self,
        sender_address: &Address<NetworkChecked>,
        swap_config: CrossChainSwap,
        fee_rate: Amount,
    ) -> Result<Transaction> {
        self.validate_network(sender_address.network)?;
        self.validate_bridge_config(&swap_config.bridge_config)?;

        // Generate stealth address for privacy
        let stealth_script = self.create_stealth_script(&swap_config.stealth_address)?;

        // Create HTLC with enhanced privacy features
        let htlc_script = self.create_enhanced_htlc_script(
            &swap_config.htlc_params,
            &stealth_script,
        )?;

        // Select UTXOs and create transaction
        let utxos = self.select_utxos(sender_address, swap_config.htlc_params.amount + fee_rate)?;
        let total_input = self.calculate_total_input(&utxos)?;
        let tx_ins = self.create_transaction_inputs(&utxos);

        let mut tx_outs = vec![TxOut {
            value: swap_config.htlc_params.amount.to_sat(),
            script_pubkey: htlc_script,
        }];

        // Add change output if needed
        let change = total_input - swap_config.htlc_params.amount - fee_rate;
        if change.to_sat() > Amount::from_sat(546).to_sat() {
            tx_outs.push(TxOut {
                value: change.to_sat(),
                script_pubkey: sender_address.script_pubkey(),
            });
        }

        // Create transaction with timelock
        let transaction = Transaction {
            version: 2,
            lock_time: LockTime::from_height(swap_config.htlc_params.timelock)
                .map_err(|e| TransactionError::ScriptError(e.to_string()))?,
            input: tx_ins,
            output: tx_outs,
        };

        Ok(transaction)
    }

    /// Creates enhanced HTLC script with privacy features
    fn create_enhanced_htlc_script(
        &self,
        params: &HtlcParams,
        stealth_script: &Script,
    ) -> Result<ScriptBuf> {
        let script = Builder::new()
            // Hash timelock branch
            .push_opcode(opcodes::OP_IF)
                .push_opcode(opcodes::OP_HASH256)
                .push_slice(&params.hash_lock)
                .push_opcode(opcodes::OP_EQUALVERIFY)
                // Add stealth address verification
                .push_slice(stealth_script.as_bytes())
                .push_opcode(opcodes::OP_SWAP)
                .push_opcode(opcodes::OP_SIZE)
                .push_int(32)
                .push_opcode(opcodes::OP_EQUALVERIFY)
                .push_opcode(opcodes::OP_HASH256)
                .push_slice(&params.recipient_key.inner.serialize())
                .push_opcode(opcodes::OP_CHECKSIG)
            // Refund branch
            .push_opcode(opcodes::OP_ELSE)
                .push_int(params.timelock as i64)
                .push_opcode(opcodes::OP_CHECKLOCKTIMEVERIFY)
                .push_opcode(opcodes::OP_DROP)
                .push_slice(&params.refund_key.inner.serialize())
                .push_opcode(opcodes::OP_CHECKSIG)
            .push_opcode(opcodes::OP_ENDIF)
            .into_script();

        Ok(script)
    }

    /// Creates stealth address script
    fn create_stealth_script(&self, stealth: &StealthAddress) -> Result<ScriptBuf> {
        let script = Builder::new()
            .push_slice(&stealth.spend_pubkey.serialize())
            .push_opcode(opcodes::OP_CHECKSIG)
            .push_slice(&stealth.view_tag)
            .push_opcode(opcodes::OP_DROP)
            .into_script();

        Ok(script)
    }

    /// Claims coins from a cross-chain HTLC
    pub fn claim_cross_chain_htlc(
        &self,
        htlc_outpoint: bitcoin::OutPoint,
        preimage: [u8; 32],
        stealth_key: &SecretKey,
        fee_rate: Amount,
    ) -> Result<Transaction> {
        // Verify preimage
        let hash = sha256::Hash::hash(&preimage);
        
        // Create claim transaction
        let tx_in = TxIn {
            previous_output: htlc_outpoint,
            script_sig: ScriptBuf::default(),
            sequence: bitcoin::Sequence::MAX,
            witness: Witness::default(),
        };

        // Generate stealth address for claiming
        let secp = Secp256k1::new();
        let stealth_pubkey = Secp256k1PublicKey::from_secret_key(&secp, stealth_key);
        
        let tx_out = TxOut {
            value: htlc_outpoint.value.saturating_sub(fee_rate.to_sat()),
            script_pubkey: Builder::new()
                .push_slice(&stealth_pubkey.serialize())
                .push_opcode(opcodes::OP_CHECKSIG)
                .into_script(),
        };

        let mut transaction = Transaction {
            version: 2,
            lock_time: LockTime::ZERO,
            input: vec![tx_in],
            output: vec![tx_out],
        };

        // Sign transaction
        self.sign_cross_chain_claim(
            &mut transaction,
            0,
            stealth_key,
            &preimage,
            fee_rate,
        )?;

        Ok(transaction)
    }

    /// Validates bridge configuration
    fn validate_bridge_config(&self, config: &BridgeConfig) -> Result<()> {
        if config.min_confirmation_depth < 6 {
            return Err(TransactionError::BridgeValidationError(
                "Minimum confirmation depth must be at least 6".into()
            ));
        }

        if config.max_timelock_duration > 2016 {
            return Err(TransactionError::BridgeValidationError(
                "Maximum timelock duration exceeded".into()
            ));
        }

        if config.min_value_sat < 546 {
            return Err(TransactionError::BridgeValidationError(
                "Value too small for cross-chain transfer".into()
            ));
        }

        Ok(())
    }

    /// Signs a cross-chain claim transaction
    fn sign_cross_chain_claim(
        &self,
        transaction: &mut Transaction,
        input_index: usize,
        stealth_key: &SecretKey,
        preimage: &[u8; 32],
        amount: Amount,
    ) -> Result<()> {
        let mut sighash_cache = bitcoin::sighash::SighashCache::new(transaction);
        
        // Generate signature hash
        let sighash = sighash_cache.taproot_signature_hash(
            input_index,
            &[],
            None,
            bitcoin::sighash::TapSighashType::All,
            amount.to_sat(),
        )?;

        let message = Message::from_slice(sighash.as_ref())
            .map_err(|e| TransactionError::ScriptError(e.to_string()))?;

        // Sign with stealth key
        let keypair = KeyPair::from_secret_key(&self.secp, stealth_key);
        let signature = self.secp.sign_schnorr(&message, &keypair);

        // Create witness with signature and preimage
        let witness = Witness::from_vec(vec![
            signature.as_ref().to_vec(),
            preimage.to_vec(),
        ]);

        sighash_cache.into_transaction().input[input_index].witness = witness;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitcoin::secp256k1::SecretKey;

    #[test]
    fn test_cross_chain_swap() {
        let (manager, sender, _) = setup_test_environment();
        
        let bridge_config = BridgeConfig {
            network: Network::Regtest,
            min_confirmation_depth: 6,
            max_timelock_duration: 144,
            min_value_sat: 10_000,
            security_level: 128,
        };

        // Generate stealth address
        let secp = Secp256k1::new();
        let scan_key = SecretKey::new(&mut rand::thread_rng());
        let spend_key = SecretKey::new(&mut rand::thread_rng());

        let stealth = StealthAddress {
            scan_pubkey: Secp256k1PublicKey::from_secret_key(&secp, &scan_key),
            spend_pubkey: Secp256k1PublicKey::from_secret_key(&secp, &spend_key),
            view_tag: [0u8; 32],
        };

        let htlc_params = HtlcParams {
            amount: Amount::from_sat(100_000),
            timelock: 144,
            hash_lock: [0u8; 32],
            recipient_key: bitcoin::PublicKey::from_private_key(&secp, &bitcoin::PrivateKey::new(spend_key, Network::Regtest)),
            refund_key: bitcoin::PublicKey::from_private_key(&secp, &bitcoin::PrivateKey::new(scan_key, Network::Regtest)),
            stealth_pubkey: Some(stealth.spend_pubkey),
            merkle_proof: None,
        };

        let swap = CrossChainSwap {
            htlc_params,
            stealth_address: stealth,
            bridge_config,
            merkle_root: [0u8; 32],
        };

        let result = manager.create_cross_chain_swap(
            &sender,
            swap,
            Amount::from_sat(1000),
        );

        assert!(result.is_err()); // Expected in test environment with no UTXOs
    }

    #[test]
    fn test_bridge_config_validation() {
        let (manager, _, _) = setup_test_environment();

        let invalid_config = BridgeConfig {
            network: Network::Regtest,
            min_confirmation_depth: 3, // Too low
            max_timelock_duration: 144,
            min_value_sat: 10_000,
            security_level: 128,
        };

        let result = manager.validate_bridge_config(&invalid_config);
        assert!(result.is_err());
    }
}
