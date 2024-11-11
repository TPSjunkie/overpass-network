// ./src/types/client/conversion_cli.rs

use crate::core::types::ovp_ops::WalletOpCode;
use crate::core::types::ovp_types::OMResult;

pub fn deserialize_wallet_op(wallet_op: &WalletOpCode) -> OMResult<WalletOpCode> {
    Ok(WalletOpCode::new(wallet_op.op_code(), wallet_op.data()))
}
pub fn serialize_wallet_op(wallet_op: &WalletOpCode) -> OMResult<WalletOpCode> {
    Ok(WalletOpCode::new(wallet_op.op_code(), wallet_op.data()))
}
