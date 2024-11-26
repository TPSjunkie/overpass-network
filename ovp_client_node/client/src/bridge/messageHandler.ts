import { Address, Cell, beginCell } from '@ton/core';
// Regular imports for enums since we need their values
import { 
    OpCode,
    TransactionStatus,
    TransactionType
} from './Interfaces';
// Type-only imports for interfaces
import type {
    Transaction,
    ChannelState,
    WasmBridge,
    ChannelTransactionParams
} from './Interfaces';

class MessageHandler {
    private wasmBridge: WasmBridge;
    private channelStates: Map<string, ChannelState>;

    constructor(wasmBridge: WasmBridge) {
        this.wasmBridge = wasmBridge;
        this.channelStates = new Map();
    }

    private uint8ArrayToCell(data: Uint8Array | undefined): Cell | undefined {
        if (!data) return undefined;
        return beginCell()
            .storeBuffer(Buffer.from(data))
            .endCell();
    }

    private createTransaction(params: {
        opCode: OpCode;
        sender: string;
        recipient: string;
        amount: bigint;
        nonce: number;
        seqno: number;
        data: string;
        groupId?: number;
        channelId?: string;
        type?: TransactionType;
    }): Transaction {
        if (!params.sender) {
            throw new Error('Sender address is required');
        }

        const timestamp = Date.now();
        const id = `tx_${timestamp}_${params.nonce}`;

        return {
            id,
            opCode: params.opCode,
            from: params.sender,
            to: params.recipient,
            amount: params.amount,
            timestamp,
            channelId: params.channelId,
            groupId: params.groupId ?? 0,
            type: params.type || TransactionType.INTERNAL,
            status: TransactionStatus.Pending,
            address: Address.parse(params.recipient),
            nonce: params.nonce,
            seqno: params.seqno,
            merkleRoot: undefined,
            signature: undefined,
            proof: undefined
        };
    }

    async processChannelTransaction(params: ChannelTransactionParams): Promise<Transaction> {
        const channelState = this.channelStates.get(params.channelId);
        if (!channelState) {
            throw new Error(`Channel ${params.channelId} not found`);
        }

        const sender = channelState.participants[0];
        if (!sender) {
            throw new Error('Channel has no participants');
        }

        // Create and sign the transaction
        const transaction = this.createTransaction({
            opCode: params.opCode ?? OpCode.SendMessage,
            sender: sender, // Now guaranteed to be string
            recipient: params.recipient,
            amount: params.amount,
            nonce: channelState.nonce + 1,
            seqno: 0, // Will be set by the chain
            data: '',
            groupId: parseInt(params.groupId),
            channelId: params.channelId,
            type: params.transactionType
        });

        // If proof is provided as Cell, use it
        if (params.proof) {
            transaction.proof = params.proof;
        }

        // Sign the transaction
        await this.wasmBridge.Transaction.sign(transaction, 'dummy-key');

        // Verify the transaction
        const isValid = await this.wasmBridge.Transaction.verify(transaction);
        if (!isValid) {
            throw new Error('Transaction verification failed');
        }

        // Serialize and send the transaction
        const serialized = this.wasmBridge.Transaction.serialize(transaction);
        await this.wasmBridge.Transaction.send({
            opCode: transaction.opCode,
            payload: JSON.stringify(transaction),
            proof: transaction.proof ? Buffer.from(transaction.proof.toBoc()) : undefined,
            timestamp: transaction.timestamp
        });

        // Update channel state
        channelState.nonce += 1;
        channelState.lastUpdated = Date.now();
        channelState.transactionHistory.push(transaction.id);
        channelState.balance -= transaction.amount;

        // Calculate new merkle root
        channelState.merkleRoot = this.wasmBridge.ChannelState.calculateMerkleRoot(channelState);
        this.channelStates.set(params.channelId, channelState);

        return transaction;
    }

    async verifyTransaction(transaction: Transaction): Promise<boolean> {
        if (!transaction.signature) {
            throw new Error('Transaction is not signed');
        }

        return this.wasmBridge.Transaction.verify(transaction);
    }

    getChannelState(channelId: string): ChannelState | undefined {
        return this.channelStates.get(channelId);
    }

    private validateChannelState(state: ChannelState): boolean {
        return this.wasmBridge.ChannelState.validate(state);
    }
}

export default MessageHandler;