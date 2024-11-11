use codec::{Decode, Encode};
use core::convert::TryFrom;
use plonky2::{field::goldilocks_field::GoldilocksField, plonk::proof::ProofWithPublicInputs};

const BOC_HASH_SIZE: usize = 32;
const BRIDGE_PREFIX: &[u8] = b"ZKBRIDGE";

#[cfg(target_pointer_width = "64")]
fn ptr_to_u32(ptr: *const u8) -> u32 {
    ptr as u64 as u32
}

#[cfg(target_pointer_width = "32")]
fn ptr_to_u32(ptr: *const u8) -> u32 {
    ptr as u32
}

#[derive(Clone, Debug, Encode, Decode, PassByCodec)]
pub struct ZkProofBoc {
    proof_data: Vec<u8>,
    vk_hash: [u8; 32],
    public_inputs: Vec<u8>,
    auxiliary_data: Vec<u8>,
}

#[derive(Clone, Debug, Encode, Decode, PassByCodec)]
pub struct ZkProofSlice {
    boc_hash: [u8; BOC_HASH_SIZE],
    metadata: ProofMetadata,
}

#[derive(Clone, Copy, Debug, Encode, Decode)]
pub struct ProofMetadata {
    version: u8,
    proof_type: ProofType,
    height_bounds: HeightBounds,
}

#[derive(Clone, Copy, Debug, Encode, Decode, PartialEq)]
#[repr(u8)]
pub enum ProofType {
    Deposit = 1,
    Withdrawal = 2,
    Transfer = 3,
}

impl TryFrom<u8> for ProofType {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(ProofType::Deposit),
            2 => Ok(ProofType::Withdrawal),
            3 => Ok(ProofType::Transfer),
            _ => Err("Invalid proof type"),
        }
    }
}

#[derive(Clone, Copy, Debug, Encode, Decode, PartialEq)]
pub struct HeightBounds {
    min_height: u32,
    max_height: u32,
}

#[runtime_interface]
pub trait ZkBoc {
    fn serialize(&self, boc: &ZkProofBoc) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend(&boc.proof_data);
        data.extend(&boc.vk_hash);
        data.extend(&boc.public_inputs);
        data.extend(&boc.auxiliary_data);
        data
    }

    fn calculate_hash(&self, data: &[u8]) -> [u8; 32] {
        keccak_256(data)
    }

    fn create_slice(&self, boc: &ZkProofBoc, metadata: ProofMetadata) -> ZkProofSlice {
        let boc_data = self.serialize(boc);
        let boc_hash = self.calculate_hash(&boc_data);
        ZkProofSlice { boc_hash, metadata }
    }
}

#[cfg(not(feature = "std"))]
impl IntoFFIValue for ZkProofSlice {
    type Owned = Vec<u8>;

    fn into_ffi_value(&self) -> WrappedFFIValue<u64, Vec<u8>> {
        let mut data = Vec::with_capacity(40);
        data.extend_from_slice(&self.boc_hash);
        data.extend_from_slice(&[self.metadata.version, self.metadata.proof_type as u8]);
        data.extend_from_slice(&self.metadata.height_bounds.min_height.to_le_bytes());
        data.extend_from_slice(&self.metadata.height_bounds.max_height.to_le_bytes());

        let ffi_value = pack_ptr_and_len(ptr_to_u32(data.as_ptr()), data.len() as u32);
        (ffi_value, data).into()
    }
}

impl ZkProofSlice {
    pub fn to_op_return(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(80);
        data.extend_from_slice(BRIDGE_PREFIX);
        data.extend_from_slice(&self.boc_hash);
        data.push(self.metadata.version);
        data.push(self.metadata.proof_type as u8);
        data.extend_from_slice(&self.metadata.height_bounds.min_height.to_le_bytes());
        data.extend_from_slice(&self.metadata.height_bounds.max_height.to_le_bytes());
        data
    }

    pub fn from_op_return(data: &[u8]) -> Result<Self, &'static str> {
        if data.len() < BRIDGE_PREFIX.len() + BOC_HASH_SIZE + 10 {
            return Err("Invalid OP_RETURN data length");
        }

        if &data[..BRIDGE_PREFIX.len()] != BRIDGE_PREFIX {
            return Err("Invalid bridge prefix");
        }

        let mut offset = BRIDGE_PREFIX.len();

        let mut boc_hash = [0u8; BOC_HASH_SIZE];
        boc_hash.copy_from_slice(&data[offset..offset + BOC_HASH_SIZE]);
        offset += BOC_HASH_SIZE;

        let version = data[offset];
        offset += 1;

        let proof_type = ProofType::try_from(data[offset])?;
        offset += 1;

        let min_height = u32::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]);
        offset += 4;

        let max_height = u32::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]);

        Ok(Self {
            boc_hash,
            metadata: ProofMetadata {
                version,
                proof_type,
                height_bounds: HeightBounds {
                    min_height,
                    max_height,
                },
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_boc() -> ZkProofBoc {
        ZkProofBoc {
            proof_data: vec![1, 2, 3, 4],
            vk_hash: [0u8; 32],
            public_inputs: vec![5, 6, 7, 8],
            auxiliary_data: vec![9, 10, 11, 12],
        }
    }

    #[test]
    fn test_slice_roundtrip() {
        let boc = create_test_boc();
        let metadata = ProofMetadata {
            version: 1,
            proof_type: ProofType::Deposit,
            height_bounds: HeightBounds {
                min_height: 100,
                max_height: 200,
            },
        };

        let api = ZkBocApi;
        let slice = api.create_slice(&boc, metadata);
        let op_return = slice.to_op_return();
        let recovered = ZkProofSlice::from_op_return(&op_return).unwrap();

        assert_eq!(recovered.boc_hash, slice.boc_hash);
        assert_eq!(recovered.metadata.version, metadata.version);
        assert_eq!(recovered.metadata.proof_type, ProofType::Deposit);
        assert_eq!(
            recovered.metadata.height_bounds.min_height,
            metadata.height_bounds.min_height
        );
        assert_eq!(
            recovered.metadata.height_bounds.max_height,
            metadata.height_bounds.max_height
        );
    }
}
