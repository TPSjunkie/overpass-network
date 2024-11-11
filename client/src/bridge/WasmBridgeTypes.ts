import { OpCode } from "./Interfaces";
import type { SparseMerkleTree } from "@/types/wasm-types";



// Define the missing interfaces
interface Transaction {
  opCode: OpCode;
  sender: string;
  recipient: string;
  amount: bigint;
  nonce: number;
  seqno: number;
}

interface ContractState {
  owner: string;
  code?: Uint8Array;
  balance: bigint;
}

interface ChannelState {
  id: number;
  participants: string[];
  balances: bigint[];
  expiresAt: number;
}

interface MerkleProof {
  root: string;
  path: string[];
  siblings: string[];
  directions: number[];
}

interface Message {
  msgType: number;
  sender: string;
  recipient: string;
  amount: bigint;
  payload?: Uint8Array;
  proof?: string;
}

export interface WasmBridge {
    KeyStore: any;
    Crypto: any;
  Transaction: {
    updateBalance(sender: string, arg1: bigint): unknown;
    emitEvent(arg0: { type: string; sender: string; recipient: string; amount: bigint; timestamp: number; }): unknown;
    processPayload(payload: Uint8Array): unknown;
    confirmDelivery(recipient: string, messageId: any): unknown;
    logDelivery(arg0: { recipient: string; timestamp: number; status: string; }): unknown;
    new(opCode: OpCode, sender: string, recipient: string, amount: bigint, nonce: number, seqno: number): Transaction;
    sign(transaction: Transaction, privateKey: string): Promise<void>;
    verifySignature(transaction: Transaction): Promise<boolean>;
    serialize(transaction: Transaction): Uint8Array;
    deserialize(bytes: Uint8Array): Transaction;
  };

  ContractState: {
    new(owner: string, code?: Uint8Array): ContractState;
    updateBalance(state: ContractState, newBalance: bigint): void;
    calculateCodeHash(state: ContractState): Uint8Array;
    serialize(state: ContractState): Uint8Array;
    deserialize(bytes: Uint8Array): ContractState;
  };

    ChannelState: {
        new(
            id: number,
            participants: string[],
            initialBalances: bigint[],
            expiresAt: number
        ): ChannelState;
        updateBalances(state: ChannelState, newBalances: bigint[]): void;
        serialize(state: ChannelState): Uint8Array;
        deserialize(bytes: Uint8Array): ChannelState;
    };

    MerkleProof: {
        new(
            root: string,
            path: string[],
            siblings: string[],
            directions: number[]
        ): MerkleProof;
        verify(proof: MerkleProof, leaf: string): Promise<boolean>;
    };

    SparseMerkleTree: {
        new: any;
        new(depth: number): SparseMerkleTree;
        new(depth: number): SparseMerkleTree;
        new(depth: number): SparseMerkleTree;
        new(depth: number): SparseMerkleTree;
        insert(tree: SparseMerkleTree, key: string, value: string): Promise<void>;
        getProof(tree: SparseMerkleTree, key: string): Promise<MerkleProof>;
        verify(tree: SparseMerkleTree, proof: MerkleProof, key: string, value: string): Promise<boolean>;
        serialize(tree: SparseMerkleTree): Uint8Array;
        deserialize(bytes: Uint8Array): SparseMerkleTree;
    };

    Message: {
        verifyProof(message: Message): unknown;
        new: any;
        new(SendMessage: OpCode, sender: string, recipient: string, amount: bigint): unknown;
        new(SendMessage: OpCode, sender: string, recipient: string, amount: bigint): unknown;
        new(SendMessage: OpCode, sender: string, recipient: string, amount: bigint): unknown;
        new(SendMessage: OpCode, sender: string, recipient: string, amount: bigint): unknown;
        new(SendMessage: OpCode, sender: string, recipient: string, amount: bigint): unknown;
        new(SendMessage: OpCode, sender: string, recipient: string, amount: bigint): unknown;
        new(SendMessage: OpCode, sender: string, recipient: string, amount: bigint): unknown;
        new(SendMessage: OpCode, sender: string, recipient: string, amount: bigint): unknown;
        new(SendMessage: OpCode, sender: string, recipient: string, amount: bigint): unknown;
        new(
            msgType: number,
            sender: string,
            recipient: string,
            amount: bigint
        ): Message;
        setPayload(message: Message, payload: Uint8Array): void;
        setProof(message: Message, proof: string): void;
        serialize(message: Message): Uint8Array;
        deserialize(bytes: Uint8Array): Message;
    };
}

export type WasmModule = {
    initializeWasm(): Promise<WasmBridge>;
};

