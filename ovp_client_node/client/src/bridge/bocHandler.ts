import { Cell, Builder, Slice, beginCell, address, Address } from '@ton/core';

class BOCHandler {
  static serializeBOC(cell: Cell): Uint8Array {
    return cell.toBoc();
  }

  static deserializeBOC(boc: Uint8Array): Cell {
    try {
      return Cell.fromBoc(Buffer.from(boc))[0];
    } catch (error) {
      throw new Error(
        `Failed to deserialize BOC: ${(error as Error).message}`
      );
    }
  }

  static createCell(data: string | number | BigInt | boolean): Cell {
    try {
      const builder = beginCell();
      if (typeof data === 'string') {
        builder.storeBuffer(Buffer.from(data));
      } else if (typeof data === 'number') {
        builder.storeUint(data, 32);
      } else if (typeof data === 'bigint') {
        builder.storeUint(data, 256);
      } else if (typeof data === 'boolean') {
        builder.storeBit(data);
      }
      return builder.endCell();
    } catch (error) {
      throw new Error(`Failed to create cell: ${(error as Error).message}`);
    }
  }

  static readCell(cell: Cell): string {
    try {
      const slice = cell.beginParse();
      return slice.loadBuffer(slice.remainingBits / 8).toString();
    } catch (error) {
      throw new Error(`Failed to read cell: ${(error as Error).message}`);
    }
  }

  static createComplexCell(data: any): Cell {
    try {
      const builder = beginCell();
      this.serializeData(builder, data);
      return builder.endCell();
    } catch (error) {
      throw new Error(`Failed to create complex cell: ${(error as Error).message}`);
    }
  }

  static readComplexCell(cell: Cell): any {
    try {
      const slice = cell.beginParse();
      return this.deserializeData(slice);
    } catch (error) {
      throw new Error(`Failed to read complex cell: ${(error as Error).message}`);
    }
  }
    private static deserializeData(slice: Slice): any {
        if (slice.remainingBits === 0) {
            return null;
        }

        if (slice.remainingBits >= 32) {
            const length = slice.loadUint(32);
            if (length > 0) {
                if (slice.remainingBits >= 8) {
                    // Assume it's an array
                    const result: any[] = [];
                    for (let i = 0; i < length; i++) {
                        result.push(this.deserializeData(slice));
                    }
                    return result;
                } else {
                    // Assume it's an object
                    const result: Record<string, any> = {};
                    for (let i = 0; i < length; i++) {
                        const key = slice.loadBuffer(slice.loadUint(8)).toString();
                        result[key] = this.deserializeData(slice);
                    }
                    return result;
                }
            }
        }

        // Handle other data types
        if (slice.remainingBits >= 8) {
            return slice.loadBuffer(slice.remainingBits / 8).toString();
        } else if (slice.remainingBits === 1) {
            return slice.loadBit();
        } else {
            return slice.loadUint(slice.remainingBits);
        }
    }    private static serializeData(builder: Builder, data: any): void {
    if (Array.isArray(data)) {
      builder.storeUint(data.length, 32);
      for (const item of data) {
        this.serializeData(builder, item);
      }
    } else if (typeof data === 'object' && data !== null) {
      const keys = Object.keys(data);
      builder.storeUint(keys.length, 32);
      for (const key of keys) {
        builder.storeBuffer(Buffer.from(key));
        this.serializeData(builder, data[key]);
      }
    } else if (typeof data === 'string') {
      builder.storeBuffer(Buffer.from(data));
    } else if (typeof data === 'number') {
      builder.storeUint(data, 32);
    } else if (typeof data === 'bigint') {
      builder.storeUint(data, 256);
    } else if (typeof data === 'boolean') {
      builder.storeBit(data);
    } else if (data === null) {
      builder.storeUint(0, 1);
    } else {
      throw new Error(`Unsupported data type: ${typeof data}`);
    }
  }

  static convertToBOC(data: any): Uint8Array {
    const cell = this.createComplexCell(data);
    return this.serializeBOC(cell);
  }

  static convertFromBOC(boc: Uint8Array): any {
    const cell = this.deserializeBOC(boc);
    return this.readComplexCell(cell);
  }

  static validateBOC(boc: Uint8Array): boolean {
    try {
      this.deserializeBOC(boc);
      return true;
    } catch (error) {
      console.error(`BOC validation failed: ${(error as Error).message}`);
      return false;
    }
  }

  static serializeAddress(address: string): Cell {
    try {
      return beginCell().storeAddress(Address.parse(address)).endCell();
    } catch (error) {
      throw new Error(`Failed to serialize address: ${(error as Error).message}`);
    }
  }
  static deserializeAddress(slice: Slice): string {
    try {
      return slice.loadAddress().toString();
    } catch (error) {
      throw new Error(`Failed to deserialize address: ${(error as Error).message}`);
    }
  }
}

export default BOCHandler;











