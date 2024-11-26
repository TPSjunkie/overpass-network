// ./src/core/zkps/zkp_transaction.rs

//
// pub struct ZkpTransaction {
//     pub circuit: ZkpCircuit, // The circuit that will be used to generate the proof
//     pub zkp: Zkp,            // The proof that will be generated
//     pub token_oc_data: TokenOcData, // The token data that will be used to generate the proof
//     pub token_oc_data_hash: ByteArray32, // The hash of the token data that will be used to generate the proof
//     pub transaction_oc_data: TransactionOcData, // The transaction data that will be used to generate the proof
//     pub transaction_oc_data_hash: ByteArray32, // The hash of the transaction data that will be used to generate the proof
// }
//
// impl ZkpTransaction {
//     // This function generates a new zkp transaction.
//     pub fn new(
//         token_oc_data: TokenOcData,
//         token_oc_data_hash: ByteArray32,
//         transaction_oc_data: TransactionOcData,
//         transaction_oc_data_hash: ByteArray32,
//     ) -> Result<Self, SystemError> {
//         // Generate the circuit. This is the circuit that will be used to generate the proof.
//         let circuit = ZkpCircuit::new(token_oc_data, transaction_oc_data)?;
//         // Generate the proof.
//         let zkp = circuit.generate_proof()?;
//         Ok(Self {
//             circuit,
//             zkp,
//             token_oc_data,
//             token_oc_data_hash,
//             transaction_oc_data,
//             transaction_oc_data_hash,
//         })
//     }
//
//     // This function verifies the zkp transaction.
//     pub fn verify(&self) -> Result<bool, SystemError> {

// Verify the proof.
//         let result = self.zkp.verify()?;
//         Ok(result)
//     }
// }
