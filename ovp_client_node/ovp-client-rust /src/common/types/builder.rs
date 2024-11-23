// ./src/common/types/builder.rs
use crate::common::types::ops::OpCode;
use crate::common::types::state_boc::STATEBOC;
use crate::common::types::builder_data::BuilderData;
use crate::common::error::common_errors::CommonError;
use crate::common::error::common_errors::CommonErrorType;
use crate::common::types::cell::Cell;
use crate::common::types::cell::CellType;
use crate::common::types::cell::CellBuilder;
use crate::common::types::cell::CellSlice;
use crate::common::types::cell::SliceData;
use crate::common::types::cell::SliceData::*;

/// Builder for cells
pub struct CellBuilderImpl {
    cells: Vec<Cell>,
}

impl CellBuilderImpl {
    pub fn new() -> Self {
        CellBuilderImpl { cells: Vec::new() }
    }

    /// Appends a raw data to the builder
    pub fn append_raw(&mut self, data: &[u8], len: usize) -> Result<(), CommonError> {
        if data.len() != len {
            return Err(CommonError::new(
                CommonErrorType::InvalidData,
                "Invalid data length".to_string(),
            ));
        }

        self.cells.push(Cell::new_raw(data.to_vec(), Vec::new(), CellType::Ordinary));
        Ok(())
    }

    /// Appends a reference to the builder
    pub fn append_reference(&mut self, cell: Cell) -> Result<(), CommonError> {
        self.cells.push(cell);
        Ok(())
    }

    /// Appends a raw data to the builder
    pub fn append_raw_slice(&mut self, data: &[u8], len: usize) -> Result<(), CommonError> {
        if data.len() != len {
            return Err(CommonError::new(
                CommonErrorType::InvalidData,
                "Invalid data length".to_string(),
            ));
        }

        self.cells.push(Cell::new_raw(data.to_vec(), Vec::new(), CellType::Ordinary));
        Ok(())
    }

    /// Appends a reference to the builder
    pub fn append_reference_slice(&mut self, cell: Cell) -> Result<(), CommonError> {
        self.cells.push(cell);
        Ok(())
    }

    /// Appends a raw data to the builder
    pub fn append_raw_builder(&mut self, data: &[u8], len: usize) -> Result<(), CommonError> {
        if data.len() != len {
            return Err(CommonError::new(
                CommonErrorType::InvalidData,
                "Invalid data length".to_string(),
            ));
        }

        self.cells.push(Cell::new_raw(data.to_vec(), Vec::new(), CellType::Ordinary));
        Ok(())
    }

    /// Appends a reference to the builder
    pub fn append_reference_builder(&mut self, cell: Cell) -> Result<(), CommonError> {
        self.cells.push(cell);
        Ok(())
    }

    /// Appends a raw data to the builder
    pub fn append_raw_slice_builder(&mut self, data: &[u8], len: usize) -> Result<(), CommonError> {
        if data.len() != len {
            return Err(CommonError::new(
                CommonErrorType::InvalidData,
                "Invalid data length".to_string(),
            ));
        }

        self.cells.push(Cell::new_raw(data.to_vec(), Vec::new(), CellType::Ordinary));
        Ok(())
    }

    /// Appends a reference to the builder
    pub fn append_reference_slice_builder(&mut self, cell: Cell) -> Result<(), CommonError> {
        self.cells.push(cell);
        Ok(())
    }

    /// Appends a raw data to the builder
    pub fn append_raw_cell(&mut self, data: &[u8], len: usize) -> Result<(), CommonError> {
        if data.len() != len {
            return Err(CommonError::new(
                CommonErrorType::InvalidData,
                "Invalid data length".to_string(),
            ));
        }

        self.cells.push(Cell::new_raw(data.to_vec(), Vec::new(), CellType::Ordinary));
        Ok(())
    }

    /// Appends a reference to the builder
    pub fn append_reference_cell(&mut self, cell: Cell) -> Result<(), CommonError> {
        self.cells.push(cell);
        Ok(())
    }

    /// Appends a raw data to the builder
    pub fn append_raw_slice_cell(&mut self, data: &[u8], len: usize) -> Result<(), CommonError> {
        if data.len() != len {
            return Err(CommonError::new(
                CommonErrorType::InvalidData,
                "Invalid data length".to_string(),
            ));
        }

        self.cells.push(Cell::new_raw(data.to_vec(), Vec::new(), CellType::Ordinary));
        Ok(())
    }

    /// Appends a reference to the builder
    pub fn append_reference_slice_cell(&mut self, cell: Cell) -> Result<(), CommonError> {
        self.cells.push(cell);
        Ok(())
    }

    /// Appends a raw data to the builder
    pub fn append_raw_builder_cell(&mut self, data: &[u8], len: usize) -> Result<(), CommonError> {
        if data.len() != len {
            return Err(CommonError::new(
                CommonErrorType::InvalidData,
                "Invalid data length".to_string(),
            ));
        }

        self.cells.push(Cell::new_raw(data.to_vec(), Vec::new(), CellType::Ordinary));
        Ok(())
    }}