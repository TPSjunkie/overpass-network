// src/wasm/runtime_wasm.rs

// This is a placeholder for the actual implementation of the runtime environment
// in the WASM environment.

//example: 
use wasm_bindgen::prelude::wasm_bindgen;


#[wasm_bindgen]
pub struct Runtime {
    memory: Vec<u8>,
    stack: Vec<i32>,
    program_counter: usize,
    registers: Vec<i32>
}

#[wasm_bindgen]
impl Runtime {
    pub fn new() -> Self {
        Self {
            memory: vec![0; 65536],  // 64KB of memory
            stack: Vec::with_capacity(1024),
            program_counter: 0,
            registers: vec![0; 16]  // 16 general purpose registers
        }
    }

    pub fn execute(&mut self, instructions: &[u8]) -> Result<i32, String> {
        self.program_counter = 0;
        self.memory[..instructions.len()].copy_from_slice(instructions);
        
        while self.program_counter < instructions.len() {
            let opcode = self.memory[self.program_counter];
            match opcode {
                0x00 => break, // halt
                0x01 => { // push
                    let value = self.memory[self.program_counter + 1] as i32;
                    self.stack.push(value);
                    self.program_counter += 2;
                },
                0x02 => { // pop
                    self.stack.pop().ok_or("Stack underflow")?;
                    self.program_counter += 1;
                },
                0x03 => { // add
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    self.stack.push(a + b);
                    self.program_counter += 1;
                },
                _ => return Err(format!("Unknown opcode: {}", opcode))
            }
        }
        
        self.stack.pop().ok_or("Stack empty at end of execution".to_string())
    }

    pub fn get_memory(&self) -> Vec<u8> {
        self.memory.clone()
    }

    pub fn get_register(&self, index: usize) -> Option<i32> {
        self.registers.get(index).copied()
    }

    pub fn set_register(&mut self, index: usize, value: i32) {
        if index < self.registers.len() {
            self.registers[index] = value;
        }
    }

    pub fn reset(&mut self) {
        self.memory.fill(0);
        self.stack.clear();
        self.program_counter = 0;
        self.registers.fill(0);
    }
}