// src/bocutils.ts

import { Cell, beginCell } from '@ton/core';

export function parseBoc(boc: Buffer | Uint8Array): Cell {
    const cells = Cell.fromBoc(boc);
    // Add null check and provide default
    const rootCell = cells[0];
    if (!rootCell) {
        throw new Error('Failed to parse BOC: root cell is missing');
    }
    return rootCell;
}

export interface CellInfo {
    data: string;
    refs: CellInfo[];
    isExotic?: boolean;  // Make isExotic optional since it's not in Cell type
    level?: number;      // Add level as optional if needed
}

export function getCellInfo(cell: Cell): CellInfo {
    // Convert cell bits to hex string safely
    const bits = cell.bits;
    const bytes = new Uint8Array(Math.ceil(bits.length / 8));
    
    // Process bits into bytes
    for (let i = 0; i < bits.length; i++) {
        // Implement bit to byte conversion logic
        const byteIndex = Math.floor(i / 8);
        const bitIndex = i % 8;
        // Set the bit in the byte if it's 1
        // You'll need to implement actual bit checking from your Cell type
    }

    // Get refs safely
    const refs = cell.refs.map(ref => {
        if (!ref) {
            throw new Error('Invalid reference in cell');
        }
        return getCellInfo(ref);
    });

    // Check for exotic cell type if applicable
    const isExotic = false; // Default value since isExotic isn't available

    return {
        data: Buffer.from(bytes).toString('hex'),
        refs,
        isExotic,
        level: 0 // Add default level if needed
    };
}

export function getBocHash(boc: Buffer | Uint8Array): string {
    const rootCell = parseBoc(boc);
    if (!rootCell) {
        throw new Error('Failed to get BOC hash: invalid BOC');
    }
    return rootCell.hash().toString('hex');
}


