// ./src/types/common/conversion_common.rs

use crate::core::types::ovp_types::*;

pub fn deserialize_common_msg_info(data: &[u8]) -> OMResult<CommonMsgInfoData> {
    Ok(CommonMsgInfoData::new(data.to_vec()))
}

pub fn serialize_common_msg_info(data: &CommonMsgInfoData) -> OMResult<Vec<u8>> {
    Ok(data.data.clone())
}
