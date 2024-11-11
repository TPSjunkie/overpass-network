// src/api/api_epoch.rs

use crate::core::types::ovp_types::*;
use crate::db as database;
use crate::db::EpochData;

pub fn get_epoch(epoch_id: u64) -> OMResult<EpochData> {
    let db = database::connect()?;
    let query = "SELECT * FROM epochs WHERE epoch_id = $1";
    let row = db
        .query_one(query, &[&epoch_id])
        .map_err(|e| OMError::DeserializationError(e.to_string()))?;

    let epoch_data = EpochData {
        id: row.get("epoch_id"),
        start_time: row.get("start_time"),
        end_time: row.get("end_time"),
        block_count: row.get("block_count"),
        transaction_count: row.get("transaction_count"),
        total_volume: row.get("total_volume"),
        rewards_distributed: row.get("rewards_distributed"),
        total_fees: row.get("total_fees"),
        validator_count: row.get("validator_count"),
        total_stake: row.get("total_stake"),
        participation_rate: row.get("participation_rate"),
        status: row.get("status"),
    };

    Ok(epoch_data)
}

pub fn get_recent_epochs(limit: u32) -> OMResult<Vec<EpochData>> {
    let db = database::connect()?;
    let query = "SELECT * FROM epochs ORDER BY epoch_id DESC LIMIT $1";
    let rows = db
        .query(query, &[&limit])
        .map_err(|e| OMError::DeserializationError(e.to_string()))?;
    let epochs = rows
        .iter()
        .map(|row| EpochData {
            id: row.get("epoch_id"),
            start_time: row.get("start_time"),
            end_time: row.get("end_time"),
            block_count: row.get("block_count"),
            transaction_count: row.get("transaction_count"),
            total_volume: row.get("total_volume"),
            rewards_distributed: row.get("rewards_distributed"),
            total_fees: row.get("total_fees"),
            validator_count: row.get("validator_count"),
            total_stake: row.get("total_stake"),
            participation_rate: row.get("participation_rate"),
            status: row.get("status"),
        })
        .collect::<Vec<EpochData>>();

    Ok(epochs)
}

pub fn get_epoch_transactions(epoch_id: u64) -> OMResult<Vec<Transaction>> {
    let db = database::connect()?;
    let query = "SELECT * FROM transactions WHERE epoch_id = $1";
    let rows = db
        .query(query, &[&epoch_id])
        .map_err(|e| OMError::DeserializationError(e.to_string()))?;

    let transactions = rows
        .iter()
        .map(|row| Transaction {
            id: row.get("id"),
            timestamp: row.get("timestamp"),
            sender: row.get("sender"),
            recipient: row.get("recipient"),
            amount: row.get("amount"),
            status: row.get("status"),
            nonce: row.get("nonce"),
            sequence_number: row.get("sequence_number"),
            signature: row.get("signature"),
            fee: row.get("fee"),
            channel_id: row.get("channel_id"),
            zk_proof: row.get("zk_proof"),
            merkle_proof: row.get("merkle_proof"),
            previous_state: row.get("previous_state"),
            new_state: row.get("new_state"),
        })
        .collect::<Vec<Transaction>>();

    Ok(transactions)
}