import { Cell } from '@ton/core';

export const parseBoc = (bocData: Buffer): Cell => {
  try {
    const cells = Cell.fromBoc(bocData);
    if (cells.length === 0) {
      throw new Error('No cells found in BOC data');
    }
    return cells[0];
  } catch (error) {
    console.error('Error parsing BOC data:', error);
    throw error;
  }
};

export const serializeBoc = (cell: Cell): Buffer => {
  try {
    const serializedBoc = cell.toBoc();
    return serializedBoc;
  } catch (error) {
    console.error('Error serializing BOC:', error);
    throw error;
  }
};

export const verifyBoc = (bocData: Buffer): boolean => {
  try {
    const cells = Cell.fromBoc(bocData);
    if (cells.length === 0) {
      return false;
    }
    return cells[0].isExotic;
  } catch (error) {
    console.error('Error verifying BOC data:', error);
    return false;
  }
};

export const extractCellsFromBoc = (bocData: Buffer): Cell[] => {
  try {
    const cells = Cell.fromBoc(bocData);
    if (cells.length === 0) {
      throw new Error('No cells found in BOC data');
    }
    const rootCell = cells[0];
    const extractedCells: Cell[] = [];

    const traverseCells = (cell: Cell) => {
      extractedCells.push(cell);
      for (let i = 0; i < cell.refs.length; i++) {
        traverseCells(cell.refs[i]);
      }
    };

    traverseCells(rootCell);
    return extractedCells;
  } catch (error) {
    console.error('Error extracting cells from BOC:', error);
    throw error;
  }
};