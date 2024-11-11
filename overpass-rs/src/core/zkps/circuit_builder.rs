use plonky2::{
    field::extension::Extendable,
    hash::{hash_types::RichField, poseidon::PoseidonHash},
    iop::target::Target,
    plonk::{circuit_builder::CircuitBuilder, circuit_data::CircuitConfig},
};
use wasm_bindgen::prelude::*;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct Column {
    index: usize,
}

impl Column {
    pub fn new(index: usize) -> Self {
        Self { index }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct VirtualCell {
    target: Target,
    #[allow(dead_code)]
    rotation: i32,
}

impl VirtualCell {
    pub fn new(target: Target) -> Self {
        Self {
            target,
            rotation: 0,
        }
    }

    pub fn target(&self) -> Target {
        self.target
    }
}

pub struct Circuit<F: RichField + Extendable<D>, const D: usize> {
    builder: CircuitBuilder<F, D>,
}

impl<F: RichField + Extendable<D>, const D: usize> Circuit<F, D> {
    pub fn new(builder: CircuitBuilder<F, D>) -> Self {
        Self { builder }
    }

    pub fn check_circuit(&self) -> Result<(), String> {
        if self.builder.num_gates() == 0 {
            return Err("Circuit must contain at least one gate".to_string());
        }
        Ok(())
    }
}

pub struct ZkCircuitBuilder<F: RichField + Extendable<D>, const D: usize> {
    builder: CircuitBuilder<F, D>,
}

impl<F: RichField + Extendable<D>, const D: usize> ZkCircuitBuilder<F, D> {
    pub fn new(config: CircuitConfig) -> Self {
        Self {
            builder: CircuitBuilder::new(config),
        }
    }

    pub fn add_public_input(&mut self) -> VirtualCell {
        let target = self.builder.add_virtual_public_input();
        VirtualCell::new(target)
    }

    pub fn add_witness(&mut self) -> VirtualCell {
        let target = self.builder.add_virtual_target();
        VirtualCell::new(target)
    }

    pub fn connect(&mut self, left: VirtualCell, right: VirtualCell) {
        self.builder.connect(left.target, right.target);
    }

    pub fn assert_zero(&mut self, x: VirtualCell) {
        let zero = self.builder.zero();
        self.builder.connect(x.target, zero);
    }

    pub fn assert_one(&mut self, x: VirtualCell) {
        let one = self.builder.one();
        self.builder.connect(x.target, one);
    }

    pub fn assert_equal(&mut self, left: VirtualCell, right: VirtualCell) {
        self.builder.connect(left.target, right.target);
    }

    pub fn add(&mut self, a: VirtualCell, b: VirtualCell) -> VirtualCell {
        let target = self.builder.add(a.target, b.target);
        VirtualCell::new(target)
    }

    pub fn sub(&mut self, a: VirtualCell, b: VirtualCell) -> VirtualCell {
        let target = self.builder.sub(a.target, b.target);
        VirtualCell::new(target)
    }

    pub fn mul(&mut self, a: VirtualCell, b: VirtualCell) -> VirtualCell {
        let target = self.builder.mul(a.target, b.target);
        VirtualCell::new(target)
    }

    pub fn constant(&mut self, value: u64) -> VirtualCell {
        let constant_value = F::from_canonical_u64(value);
        let target = self.builder.constant(constant_value);
        VirtualCell::new(target)
    }

    pub fn poseidon(&mut self, inputs: &[VirtualCell]) -> VirtualCell {
        let input_targets: Vec<Target> = inputs.iter().map(|x| x.target).collect();
        let hash = self
            .builder
            .hash_n_to_hash_no_pad::<PoseidonHash>(input_targets);
        let target = hash.elements[0];
        VirtualCell::new(target)
    }

    pub fn build_channel_state_circuit(
        &mut self,
        old_balance: VirtualCell,
        new_balance: VirtualCell,
        amount: VirtualCell,
    ) -> Result<(), JsValue> {
        let computed_new = self.sub(old_balance, amount);
        self.assert_equal(computed_new, new_balance);

        let two = self.constant(2);
        let half_balance = self.builder.div(old_balance.target, two.target);
        let limit_check = self.builder.sub(half_balance, amount.target);
        self.assert_zero(VirtualCell::new(limit_check));

        Ok(())
    }

    pub fn build_merkle_proof_circuit(
        &mut self,
        leaf: VirtualCell,
        path: &[VirtualCell],
        root: VirtualCell,
    ) -> Result<(), JsValue> {
        let mut current = leaf;

        for sibling in path {
            let inputs = [current, *sibling];
            current = self.poseidon(&inputs);
        }

        self.assert_equal(current, root);

        Ok(())
    }

    pub fn build_transaction_circuit(
        &mut self,
        sender_balance: VirtualCell,
        recipient_balance: VirtualCell,
        amount: VirtualCell,
        sender_nonce: VirtualCell,
    ) -> Result<(), JsValue> {
        let _new_sender_balance = self.sub(sender_balance, amount);
        let _new_recipient_balance = self.add(recipient_balance, amount);

        let two = self.constant(2);
        let half_balance = self.builder.div(sender_balance.target, two.target);
        let limit_check = self.builder.sub(half_balance, amount.target);
        self.assert_zero(VirtualCell::new(limit_check));

        let one = self.constant(1);
        let _new_nonce = self.add(sender_nonce, one);

        Ok(())
    }

    pub fn build(self) -> Result<Circuit<F, D>, JsValue> {
        Ok(Circuit::new(self.builder))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use plonky2::field::goldilocks_field::GoldilocksField;

    const D: usize = 2;
    type F = GoldilocksField;

    #[test]
    fn test_channel_state_circuit() {
        let config = CircuitConfig::standard_recursion_config();
        let mut builder = ZkCircuitBuilder::<F, D>::new(config);

        let old_balance = builder.add_public_input();
        let new_balance = builder.add_public_input();
        let amount = builder.add_public_input();

        builder
            .build_channel_state_circuit(old_balance, new_balance, amount)
            .unwrap();

        let circuit = builder.build().unwrap();
        assert!(circuit.check_circuit().is_ok());
    }

    #[test]
    fn test_merkle_proof_circuit() {
        let config = CircuitConfig::standard_recursion_config();
        let mut builder = ZkCircuitBuilder::<F, D>::new(config);

        let leaf = builder.add_public_input();
        let root = builder.add_public_input();

        let path_inputs: Vec<_> = (0..32).map(|_| builder.add_public_input()).collect();

        builder
            .build_merkle_proof_circuit(leaf, &path_inputs, root)
            .unwrap();

        let circuit = builder.build().unwrap();
        assert!(circuit.check_circuit().is_ok());
    }

    #[test]
    fn test_transaction_circuit() {
        let config = CircuitConfig::standard_recursion_config();
        let mut builder = ZkCircuitBuilder::<F, D>::new(config);

        let sender_balance = builder.add_public_input();
        let recipient_balance = builder.add_public_input();
        let amount = builder.add_public_input();
        let sender_nonce = builder.add_public_input();

        builder
            .build_transaction_circuit(sender_balance, recipient_balance, amount, sender_nonce)
            .unwrap();

        let circuit = builder.build().unwrap();
        assert!(circuit.check_circuit().is_ok());
    }
}
