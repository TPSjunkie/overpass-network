
export enum ChannelStatus {
  Active = 0,
  TransactionPending = 1,
  DisputeOpen = 2,
  Closing = 3,
  Closed = 4,
}

export enum ContractOpCode {
  CreatePayment = 160,
  UpdateState = 161,
  FinalizeState = 162,
  DisputeState = 163,
  InitChannel = 164,
}

export enum GateType {
  Arithmetic = 0,
  Constant = 1,
  Poseidon = 2,
}

export interface IByteArray32 {
  free(): void;
  toWasmAbi(): any;
  to_array(): Uint8Array;
  to_string(): string;
}

export interface ByteArray32Constructor {
  new(arg0: Uint8Array): IByteArray32;
  new(arg0: Uint8Array): IByteArray32;
  new(arg0: Uint8Array): IByteArray32;
  new (array: Uint8Array): IByteArray32;
  fromWasmAbi(val: any): IByteArray32;
  from_string(val: string): IByteArray32;
}
export const ByteArray32: ByteArray32Constructor = class ByteArray32
  implements IByteArray32
{
  constructor(array: Uint8Array) {
    return {
      free() {},
      toWasmAbi() { return array; },
      to_array() { return array; },
      to_string() { return Array.from(array).map(b => b.toString(16).padStart(2, '0')).join(''); }
    };
  }
  free(): void {
    throw new Error("Method not implemented.");
  }
  toWasmAbi() {
    throw new Error("Method not implemented.");
  }
  to_array(): Uint8Array {
    throw new Error("Method not implemented.");
  }
  to_string(): string {
    throw new Error("Method not implemented.");
  }

  static fromWasmAbi(val: any): IByteArray32 {
    return new ByteArray32(new Uint8Array(val));
  }

  static from_string(val: string): IByteArray32 {
    const array = new Uint8Array(val.match(/.{1,2}/g)?.map(byte => parseInt(byte, 16)) || []);
    return new ByteArray32(array);
  }
};

export interface IChannelContract {
  free(): void;
  update_balance(amount: bigint): void;
  create_state_boc(): Uint8Array;
  process_transaction(tx: ITransaction): Uint8Array;
  get_timeout(): any;
  set_timeout(timeout: any): void;
  get_recipient_acceptance(): any;
  set_recipient_acceptance(acceptance: any): void;
  get_challenger(): any;
  set_challenger(challenger: any): void;
  get_initiated_at(): any;
  set_initiated_at(initiated_at: any): void;
  get_final_state(): any;
  set_final_state(final_state: any): void;
  readonly balance: bigint;
  readonly id: string;
  readonly nonce: bigint;
  readonly op_code: ContractOpCode;
  readonly seqno: bigint;
  readonly status: ChannelStatus;
}

export interface ChannelContractConstructor {
  [x: string]: any;
  new(id: string): IChannelContract;
  new(id: string): IChannelContract;
  new (id: string): IChannelContract;
}

export const ChannelContract: ChannelContractConstructor = {
  new(id: string): IChannelContract {
    return {
      balance: 0n,
      id,
      nonce: 0n,
      op_code: ContractOpCode.InitChannel,
      seqno: 0n,
      status: ChannelStatus.Active,
      free() {},
      update_balance(amount: bigint) {},
      create_state_boc() { return new Uint8Array(); },
      process_transaction(tx: ITransaction) { return new Uint8Array(); },
      get_timeout() { return null; },
      set_timeout(timeout: any) {},
      get_recipient_acceptance() { return false; },
      set_recipient_acceptance(acceptance: any) {},
      get_challenger() { return null; },
      set_challenger(challenger: any) {},
      get_initiated_at() { return Date.now(); },
      set_initiated_at(initiated_at: any) {},
      get_final_state() { return null; },
      set_final_state(final_state: any) {}
    };
  }
} as unknown as ChannelContractConstructor;
export interface IPlonky2SystemHandle {
  free(): void;
  generate_proof_js(
    old_balance: bigint,
    old_nonce: bigint,
    new_balance: bigint,
    new_nonce: bigint,
    transfer_amount: bigint
  ): Uint8Array;
  verify_proof_js(proof_bytes: Uint8Array): boolean;
}

export interface Plonky2SystemHandleConstructor {
  [x: string]: any;
  new(): unknown;
  new(): unknown;
  new(): unknown;
  new (): IPlonky2SystemHandle;
}

export const Plonky2SystemHandle: Plonky2SystemHandleConstructor = {
  new(): IPlonky2SystemHandle {
    return {
      free() {},
      generate_proof_js(old_balance: bigint, old_nonce: bigint, new_balance: bigint, new_nonce: bigint, transfer_amount: bigint) {
        return new Uint8Array();
      },
      verify_proof_js(proof_bytes: Uint8Array) {
        return true;
      }
    };
  }
} as unknown as Plonky2SystemHandleConstructor;
export interface IProofGenerator {
  free(): void;
  generate_state_transition_proof(
    old_balance: bigint,
    new_balance: bigint,
    amount: bigint,
    channel_id?: Uint8Array
  ): any;
  verify_state_transition(
    bundle_js: any,
    old_balance: bigint,
    new_balance: bigint,
    amount: bigint
  ): boolean;
}

export interface ProofGeneratorConstructor {
  new (): IProofGenerator;
}

export const ProofGenerator: ProofGeneratorConstructor = {
  new(): IProofGenerator {
    return {
      free() {},
      generate_state_transition_proof(old_balance: bigint, new_balance: bigint, amount: bigint, channel_id?: Uint8Array) {
        return {};
      },
      verify_state_transition(bundle_js: any, old_balance: bigint, new_balance: bigint, amount: bigint): true {
        return true;
      }
    };
  }
} as unknown as ProofGeneratorConstructor;

export interface IProofMetadataJS {
  free(): void;
  channel_id?: Uint8Array;
  verified_at?: bigint;
  proof_type: number;
  created_at: bigint;
}

export interface ProofMetadataJSConstructor {
  [x: string]: any;
  new(arg0: number, timestamp: bigint): unknown;
  new(arg0: number, timestamp: bigint): unknown;
  new(arg0: number, timestamp: bigint): unknown;
  new (proof_type: number, created_at: bigint): IProofMetadataJS;
}

export const ProofMetadataJS: ProofMetadataJSConstructor = {
  new(proof_type: number, created_at: bigint): IProofMetadataJS {
    return {
      free() {},
      proof_type,
      created_at,
      channel_id: undefined,
      verified_at: undefined
    };
  }
} as unknown as ProofMetadataJSConstructor;
export interface IProofWithMetadataJS {
  free(): void;
  readonly proof: any;
  readonly metadata: any;
}

export interface ProofWithMetadataJSConstructor {
  [x: string]: any;
  new(arg0: { proof_bytes: Uint8Array; merkle_root: Uint8Array; public_inputs: Uint8Array; }, metadata: any): any;
  new(arg0: { proof_bytes: Uint8Array; merkle_root: Uint8Array; public_inputs: Uint8Array; }, metadata: any): any;
  new(arg0: { proof_bytes: Uint8Array; merkle_root: Uint8Array; public_inputs: Uint8Array; }, metadata: any): any;
  new (proof_js: any, metadata_js: any): IProofWithMetadataJS;
}

export const ProofWithMetadataJS: ProofWithMetadataJSConstructor = {
  new(proof_js: any, metadata_js: any): IProofWithMetadataJS {
    return {
      free() {},
      proof: proof_js,
      metadata: metadata_js
    };
  }
} as unknown as ProofWithMetadataJSConstructor;

export interface ITransaction {
  free(): void;
  readonly sender: string;
  readonly nonce: bigint;
  readonly sequence_number: bigint;
  readonly amount: bigint;
}

export interface TransactionConstructor {
  new (
    sender: string,
    nonce: bigint,
    sequence_number: bigint,
    amount: bigint
  ): ITransaction;
}

export const Transaction: TransactionConstructor = {
  new(sender: string, nonce: bigint, sequence_number: bigint, amount: bigint): ITransaction {
    return {
      free() {},
      sender,
      nonce,
      sequence_number,
      amount
    };
  }
} as unknown as TransactionConstructor;

export interface IZkProof {
  free(): void;
}

export interface ZkProofConstructor {
  new (): IZkProof;
}

export const ZkProof: ZkProofConstructor = {
  new(): IZkProof {
    return {
      free() {}
    };
  }
} as unknown as ZkProofConstructor;

export function create_channel_contract(id: string): IChannelContract {
  return ChannelContract.new(id);
}

export function create_proof(
  old_balance: BigInt,
  old_nonce: BigInt,
  new_balance: BigInt,
  new_nonce: BigInt,
  transfer_amount: BigInt
): Uint8Array {
  const handle = Plonky2SystemHandle.new();
  return handle.generate_proof_js(
    old_balance,
    old_nonce,
    new_balance,
    new_nonce,
    transfer_amount
  );
}

export function verify_proof(proof_bytes: Uint8Array): boolean {
  const handle = Plonky2SystemHandle.new();
  return handle.verify_proof_js(proof_bytes);
}

export function create_proof_with_metadata(
  proof_bytes: Uint8Array,
  merkle_root: Uint8Array,
  public_inputs: Uint8Array,
  timestamp: bigint
): any {
  const metadata = ProofMetadataJS.new(0, timestamp);
  return ProofWithMetadataJS.new({ proof_bytes, merkle_root, public_inputs }, metadata);
}

function fromWasmAbi(val: any, any: any) {
  throw new Error("Function not implemented.");
}


function from_string(val: any, string: any) {
  throw new Error("Function not implemented.");
}
