// src/utils/ton.ts


interface Cell {
  toBoc(): unknown;
}

interface CellBuilder {
  storeAddress(address: string): CellBuilder;
  storeCoins(amount: bigint): CellBuilder;
  storeUint(value: number, bits: number): CellBuilder;
  storeString(str: string): CellBuilder;
  storeBuffer(buffer: Buffer): CellBuilder;
  storeRef(cell: Cell): CellBuilder;
  endCell(): Cell;
}
const beginCell = (): CellBuilder => {
  return beginCell() as unknown as CellBuilder;
};

export const createCell = (params: {
  address: string;
  amount: bigint;
  payload?: string;
}): Cell => {
  const builder = beginCell()
    .storeAddress(params.address)
    .storeCoins(params.amount)
    .storeUint(0, 1) // Simple transfer
    .storeUint(0, 1) // No state init
    .storeUint(params.payload ? 1 : 0, 1); // Has payload?

  if (params.payload) {
    builder.storeString(params.payload);
  }

  return builder.endCell();
};
export const formatAddress = (address: string): string => {
  if (!address || address.length <= 12) return address;
  return `${address.slice(0, 6)}...${address.slice(-6)}`;
};

export const parseAmount = (amount: string): bigint => {
  return BigInt(Math.floor(parseFloat(amount) * 1e9));
};