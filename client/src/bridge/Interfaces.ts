// Interfaces.ts

import { Address, Cell } from '@ton/core';

export enum OpCode {
    CreateChildContract = 0x01,
    ExecuteContract = 0x02,
    UpdateState = 0x03,
    TransferFunds = 0x04,
    SendMessage = 0x05,
    CreateChannel = 0x06,
    CloseChannel = 0x07,
    UpdateChannel = 0x08
}

export enum TransactionStatus {
    Pending = 0,
    Completed = 1,
    Failed = 2,
    Rejected = 3
}

export enum TransactionType {
    SENDER = 'sender',
    RECIPIENT = 'recipient',
    INTERNAL = 'internal',
    CHANNEL = 'channel'
}

export interface Transaction {
    id: string;
    opCode: OpCode;
    from: string;
    to: string;
    amount: bigint;
    timestamp: number;
    channelId?: string;
    groupId: number;
    type: TransactionType;
    status: TransactionStatus;
    merkleRoot?: Cell;
    signature?: string;
    address: Address;
    proof?: Cell;
    nonce?: number;
    seqno?: number;
}

export interface ChannelState {
    id: string;
    opCode: OpCode;
    balance: bigint;
    nonce: number;
    group: number;
    merkleRoot: Cell;
    participants: string[];
    status: 'active' | 'closed' | 'pending';
    lastUpdated: number;
    transactionHistory: string[];
    init?: {
        code: Cell;
        data: Cell;
    };
}

export interface ChannelTransactionParams {
    channelId: string;
    transactionType: TransactionType;
    groupId: string;
    recipient: string;
    amount: bigint;
    proof?: Cell;
    opCode?: OpCode;
}

// Rest of interfaces remain the same...
export interface AnalyticsData {
    totalTransactions: number;
    totalAmount: bigint;
    averageTransactionAmount: number;
    transactionsByType: {
        incoming: number;
        outgoing: number;
        internal: number;
    };
    dailyVolume: Map<string, bigint>;
    activeChannels: number;
    successRate: number;
    averageConfirmationTime: number;
    largestTransaction: bigint;
    recentActivity: Transaction[];
}

export interface WasmBridge {
    Transaction: {
        send(transaction: { opCode: OpCode; payload: string; proof: Uint8Array | undefined; timestamp: number; }): unknown;
        deserialize(data: Uint8Array): Transaction;
        serialize(tx: Transaction): Uint8Array;
        sign(tx: Transaction, privateKey: string): Promise<void>;
        verify(tx: Transaction): Promise<boolean>;
    };
    ChannelState: {
        deserialize(data: Uint8Array): ChannelState;
        serialize(state: ChannelState): Uint8Array;
        validate(state: ChannelState): boolean;
        calculateMerkleRoot(state: ChannelState): Cell;
    };
}

export interface StorageProvider {
    getItem(key: string): Promise<string | null>;
    setItem(key: string, value: string): Promise<void>;
    removeItem(key: string): Promise<void>;
    clear(): Promise<void>;
    getAllKeys(): Promise<string[]>;
}

export interface NetworkProvider {
    sendTransaction(data: Uint8Array): Promise<void>;
    getStatus(txId: string): Promise<'pending' | 'completed' | 'failed'>;
    getConfirmations(txId: string): Promise<number>;
}