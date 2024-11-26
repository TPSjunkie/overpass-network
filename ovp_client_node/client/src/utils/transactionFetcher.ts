// src/utils/transactionFetcher.ts
import { TonClient } from '@ton/ton';
import type { Transaction, TransactionResponse, ChannelState, WalletExtension } from '../types/wasm-types/index';
import { channelManager } from './channelManager';
import { fetchTransactions as blockchainFetchTransactions } from "@/utils/offChain";

import type { ChannelTransactionParams } from './wasmTransactionSender';
import initWasm from '@wasm/overpass_rs';

let walletExtension: WalletExtension;

async function initializeWasm(): Promise<WalletExtension> {
    if (!walletExtension) {
        const wasmModule = await initWasm();

        if (wasmModule !== undefined && typeof wasmModule === 'object' && 'WalletExtension' in wasmModule) {
            walletExtension = new (wasmModule as any).WalletExtension();
        } else {
            throw new Error('WalletExtension not found in the WASM module');
        }
    }

    return walletExtension;
}
export const getWalletExtension = (): WalletExtension => {
    if (!walletExtension) {
        throw new Error('WalletExtension not initialized');
    }
    return walletExtension;
};
export async function fetchTransactions(
    address: string,
    client: TonClient
): Promise<Transaction[]> {
    if (!walletExtension) {
        await initializeWasm();
    }
    const mapTransactionResponse = (tx: TransactionResponse, address: string): Transaction => ({
        id: tx.id,
        amount: Number(tx.amount),
        timestamp: Number(tx.timestamp),
        from: tx.sender,
        to: tx.recipient,
        type: determineTransactionType(tx.type),
        status: tx.status,
        address: address,
        groupId: tx.groupId,
        recipient: tx.recipient,
        value: undefined,
        data: function (to: string, arg1: any, data: any, channelId: string, groupId: string, transactionType: Transaction): unknown {
            throw new Error('Function not implemented.');
        },
        payload: function (arg0: string, arg1: string, payload: any, channelId: any, groupId: any, transactionType: any): unknown {
            throw new Error('Function not implemented.');
        },
        channelId: function (arg0: string, arg1: string, payload: any, channelId: any, groupId: any, transactionType: any): unknown {
            throw new Error('Function not implemented.');
        },
        transactionType: function (arg0: string, arg1: string, payload: any, channelId: any, groupId: any, transactionType: any): unknown {
            throw new Error('Function not implemented.');
        }
    });

    function determineTransactionType(type: string): "internal" | "incoming" | "outgoing" {
        if (type === 'offchain') return 'internal';
        if (type === 'in') return 'incoming';
        if (type === 'out') return 'outgoing';
        return 'internal';
    }

    const onChainTransactions = await blockchainFetchTransactions(address, client);
    const offChainTransactions = await fetchOffChainTransactions(address);

    const channelId = channelManager.getChannelAddress(address);
    if (channelId) {
        const channelState = await getChannelState(channelId);
        return [
            ...onChainTransactions,
            ...offChainTransactions,
            ...channelState.transactions
        ].map(tx => mapTransactionResponse(tx, address));
    } else {
        return [...onChainTransactions, ...offChainTransactions].map(
            tx => mapTransactionResponse(tx, address)
        );
    }
}
async function fetchOffChainTransactions(address: string): Promise<TransactionResponse[]> {
    if (!walletExtension) {
        await initializeWasm();
    }

    try {
        const offChainTxs = walletExtension.get_transactions(address);
        if (!Array.isArray(offChainTxs)) {
            throw new Error('Invalid response from walletExtension.get_transactions');
        }
        return offChainTxs.map((tx: any): TransactionResponse => ({
            id: tx.hash,
            amount: tx.amount,
            timestamp: tx.timestamp,
            sender: tx.sender,
            recipient: tx.recipient,
            type: 'internal',
            status: tx.status,
            address: tx.address,
            tree: tx.tree,
            channelId: tx.channelId,
            groupId: tx.groupId,
            description: tx.description,
            nonce: tx.nonce,
            seqno: tx.seqno,
            senderSignature: tx.senderSignature,
            payload: tx.payload,
            OPcode: tx.OPcode,
            statusMessage: tx.statusMessage,
            statusColor: tx.statusColor,
            merkleRoot: tx.merkleRoot,
            date: new Date(tx.timestamp).toISOString(),
            hash: ''
        }));
    } catch (error) {
        console.error(`Error fetching off-chain transactions: ${error}`);
        throw new Error(`Failed to fetch off-chain transactions for address ${address}`);
    }
}
async function getChannelState(channelId: string): Promise<ChannelState> {
    if (!walletExtension) {
        await initializeWasm();
    }

    const channelAddress = channelManager.getChannelAddress(channelId);
    if (!channelAddress) {
        throw new Error(`Channel with ID ${channelId} not found`);
    }

    try {
        const offChainState = walletExtension.get_channel(Number(channelId));
        return {
            balance: Number(offChainState.balance),
            members: [],
            transactions: []
        };
    } catch (error) {
        console.error(`Error fetching channel state: ${error}`);
        throw new Error(`Failed to fetch channel state for ID ${channelId}`);
    }
}

async function getChannelBalance(channelId: string): Promise<bigint> {
    if (!walletExtension) {
        await initializeWasm();
    }

    const channelAddress = channelManager.getChannelAddress(channelId);
    if (!channelAddress) {
        throw new Error(`Channel with ID ${channelId} not found`);
    }

    try {
        const offChainState = walletExtension.get_channel(Number(channelId));
        return BigInt(offChainState.balance);
    } catch (error) {
        console.error(`Error fetching channel balance: ${error}`);
        throw new Error(`Failed to fetch balance for channel ID ${channelId}`);
    }
}

async function getChannelTransactions(
channelId: string, client: TonClient, params: ChannelTransactionParams): Promise<Transaction[]> {
    if (!walletExtension) {
        await initializeWasm();
    }

    const channelAddress = channelManager.getChannelAddress(channelId);
    if (!channelAddress) {
        throw new Error(`Channel with ID ${channelId} not found`);
    }

    const onChainTransactions = await fetchTransactions(channelAddress, client);
    const offChainTransactions = await fetchOffChainTransactions(channelAddress);

    return [...onChainTransactions, ...offChainTransactions.map(tx => mapTransactionResponse(tx, channelAddress))];
}

const transactionFetcher = {
    fetchTransactions,
    fetchOffChainTransactions,
    getChannelState,
    getChannelBalance,
    getChannelTransactions,
};

export default transactionFetcher;

function mapTransactionResponse(tx: TransactionResponse, channelAddress: string): any {
    throw new Error('Function not implemented.');
}
