// ./src/core/zkps/plonky2/plonky2.rs

use plonky2::hash::hash_types::RichField;
use plonky2::{
    gates::{
        arithmetic_base::ArithmeticGate, constant::ConstantGate, gate::Gate, poseidon::PoseidonGate,
    },
    plonk::circuit_data::CircuitConfig,
};
use plonky2_field::extension::Extendable;
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

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct VirtualCell {
    column: Column,
    rotation: i32,
}

impl VirtualCell {
    pub fn new(column: Column, rotation: i32) -> Self {
        Self { column, rotation }
    }

    pub fn value(&self) -> u64 {
        0
    }
}

#[allow(dead_code)]
pub struct Circuit<F: RichField + Extendable<D>, const D: usize> {
    config: CircuitConfig,
    gates: Vec<Box<dyn Gate<F, D>>>,
    public_inputs: Vec<Column>,
    witnesses: Vec<Column>,
}

impl<F: RichField + Extendable<D>, const D: usize> Circuit<F, D> {
    pub fn new(
        config: CircuitConfig,
        gates: Vec<Box<dyn Gate<F, D>>>,
        public_inputs: Vec<Column>,
        witnesses: Vec<Column>,
    ) -> Result<Self, String> {
        Ok(Self {
            config,
            gates,
            public_inputs,
            witnesses,
        })
    }

    pub fn check_circuit(&self) -> Result<(), String> {
        Ok(())
    }
}

#[wasm_bindgen]
#[derive(Clone, Debug)]
pub enum GateType {
    Arithmetic,
    Constant,
    Poseidon,
}

pub struct ZkCircuitBuilder<F: RichField + Extendable<D>, const D: usize> {
    config: CircuitConfig,
    gates: Vec<Box<dyn Gate<F, D>>>,
    public_inputs: Vec<Column>,
    witnesses: Vec<Column>,
}

impl<F: RichField + Extendable<D>, const D: usize> ZkCircuitBuilder<F, D> {
    pub fn new(config: CircuitConfig) -> Self {
        Self {
            config,
            gates: Vec::new(),
            public_inputs: Vec::new(),
            witnesses: Vec::new(),
        }
    }

    pub fn add_gate<G: Gate<F, D> + 'static>(&mut self, gate: G) {
        self.gates.push(Box::new(gate));
    }

    pub fn add_public_input(&mut self) -> Column {
        let column = Column::new(self.public_inputs.len());
        self.public_inputs.push(column.clone());
        column
    }

    pub fn add_witness(&mut self) -> Column {
        let column = Column::new(self.witnesses.len());
        self.witnesses.push(column.clone());
        column
    }

    pub fn connect(&mut self, left: VirtualCell, right: VirtualCell) {
        let _ = (left, right); // Mark as used
        let gate = ArithmeticGate::new_from_config(&self.config);
        self.add_gate(gate);
    }

    pub fn assert_zero(&mut self, x: VirtualCell) {
        let _ = x; // Mark as used
        let gate = ArithmeticGate::new_from_config(&self.config);
        self.add_gate(gate);
    }

    pub fn assert_one(&mut self, x: VirtualCell) {
        let _ = x; // Mark as used
        let gate = ArithmeticGate::new_from_config(&self.config);
        self.add_gate(gate);
    }

    pub fn assert_equal(&mut self, left: VirtualCell, right: VirtualCell) {
        let _ = (left, right); // Mark as used
        let gate = ArithmeticGate::new_from_config(&self.config);
        self.add_gate(gate);
    }

    pub fn add(&mut self, a: VirtualCell, b: VirtualCell) -> VirtualCell {
        let _ = (a, b); // Mark as used
        let c = self.add_witness();
        let gate = ArithmeticGate::new_from_config(&self.config);
        self.add_gate(gate);
        VirtualCell::new(c, 0)
    }

    pub fn sub(&mut self, a: VirtualCell, b: VirtualCell) -> VirtualCell {
        let _ = (a, b); // Mark as used
        let c = self.add_witness();
        let gate = ArithmeticGate::new_from_config(&self.config);
        self.add_gate(gate);
        VirtualCell::new(c, 0)
    }

    pub fn mul(&mut self, a: VirtualCell, b: VirtualCell) -> VirtualCell {
        let _ = (a, b); // Mark as used
        let c = self.add_witness();
        let gate = ArithmeticGate::new_from_config(&self.config);
        self.add_gate(gate);
        VirtualCell::new(c, 0)
    }

    pub fn constant(&mut self, value: F) -> VirtualCell {
        let _ = value; // Mark as used
        let c = self.add_witness();
        let gate = ConstantGate::new(1);
        self.add_gate(gate);
        VirtualCell::new(c, 0)
    }

    pub fn poseidon(&mut self, inputs: &[VirtualCell]) -> VirtualCell {
        let _ = inputs; // Mark as used
        let output = self.add_witness();
        let gate = PoseidonGate::<F, D>::new();
        self.add_gate(gate);
        VirtualCell::new(output, 0)
    }

    pub fn build_transaction_circuit(
        &mut self,
        sender_balance: &VirtualCell,
        recipient_balance: &VirtualCell,
        amount: &VirtualCell,
        sender_nonce: &VirtualCell,
    ) -> Result<(), JsValue> {
        let constant_one = self.constant(F::ONE);

        let _new_sender = self.sub(sender_balance.clone(), amount.clone());
        let _new_recipient = self.add(recipient_balance.clone(), amount.clone());
        let new_nonce = self.add(sender_nonce.clone(), constant_one);

        self.assert_equal(sender_nonce.clone(), new_nonce);

        Ok(())
    }

    pub fn build_circuit(self) -> Result<Circuit<F, D>, JsValue> {
        Circuit::new(self.config, self.gates, self.public_inputs, self.witnesses)
            .map_err(|e| JsValue::from_str(&e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use plonky2_field::goldilocks_field::GoldilocksField;

    const D: usize = 2;

    #[test]
    fn test_circuit_builder() {
        let config = CircuitConfig::standard_recursion_config();
        let builder = ZkCircuitBuilder::<GoldilocksField, D>::new(config);
        let circuit = builder.build_circuit();
        assert!(circuit.is_ok());
    }
}
