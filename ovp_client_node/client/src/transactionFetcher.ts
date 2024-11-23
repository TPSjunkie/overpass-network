// src/utils/transactionFetcher.ts

import { Address, TonClient } from '@ton/ton';
import type { Transaction, ChannelState } from '@/types/wasm-types';
import { channelManager } from './channelManager';
import { fetchTransactions as blockchainFetchTransactions } from "@/utils/offChain";
async function fetchTransactions(
    address: string,
    client: TonClient,
): Promise<Transaction[]> {
    const transactions = await blockchainFetchTransactions(address, client);

    return transactions.map((tx) => ({
        id: tx.hash().toString("hex"),
        amount: Number(
            tx.inMessage?.info.type === "internal"
                ? tx.inMessage.info.value.coins
                : 0,
        ),
        timestamp: tx.now * 1000,
        date: new Date(tx.now * 1000).toISOString(),
        from: tx.inMessage?.info.type === "internal"
            ? tx.inMessage.info.src.toString()
            : "Unknown",
        to: tx.inMessage?.info.type === "internal"
            ? tx.inMessage.info.dest.toString()
            : "Unknown",
        type: tx.inMessage?.info.type === "internal" ? "incoming" : "outgoing",
        status: "completed",
        address: address,
        channelId: (arg0: string, arg1: string, payload: any, channelId: any, groupId: any, transactionType: any) => {
            // Implement the channelId function logic here
            return "";
        },
        groupId: "",
        description: "",
        fee: 0,
        metadata: {},
        relatedTransactions: [],
        tags: [],
        attachments: [],
        notes: "",
        category: "",
        subcategory: "",
        nonce: 0,
        seqno: 0,
        hash: tx.hash().toString("hex"),
        senderSignature: "",
        merkleRoot: "",
        payload: (arg0: string, arg1: string, payload: any, channelId: any, groupId: any, transactionType: any) => {
            // Implement the payload function logic here
            return undefined;
        },
        OPcode: 0,
        statusMessage: "",
        statusColor: "",
        recipient: tx.inMessage?.info.type === "internal"
            ? tx.inMessage.info.dest.toString()
            : "Unknown",
        value: tx.inMessage?.info.type === "internal"
            ? tx.inMessage.info.value.coins
            : BigInt(0),
        data: tx.inMessage?.body?.toString() ?? "",
        transactionType: (arg0: string, arg1: string, payload: any, channelId: any, groupId: any, transactionType: any) => {
            return tx.inMessage?.info.type === "internal" ? "incoming" : "outgoing";
        }
    }));
}
// For the getChannelState function

async function getChannelState(channelId: string, client: TonClient): Promise<ChannelState> {
    const channelAddress = channelManager.getChannelAddress(channelId);
    if (channelAddress === undefined || channelAddress === null) {
        throw new Error(`Channel with ID ${channelId} not found`);
    }   
    const channelData = await client.getContractState(Address.parse(channelAddress));   
    // Return the channel state
    return {
        address: channelAddress,
        balance: Number((channelData as any).balance),
        isActive: true,
        
    };
}

// For the getChannelBalance function
async function getChannelBalance(channelId: string, client: TonClient): Promise<number> {
    const channelAddress = channelManager.getChannelAddress(channelId);
    if (channelAddress === undefined || channelAddress === null) {
        throw new Error(`Channel with ID ${channelId} not found`);
    }

    const channelData = await client.getContractState(Address.parse(channelAddress));
    // Add type assertion or proper type checking
    if (channelData === null || channelData === undefined || typeof (channelData as any).balance === "undefined") {
        throw new Error("Failed to retrieve channel data");
    }
    return Number((channelData as any).balance);
}

async function getChannelTransactions(channelId: string, client: TonClient): Promise<Transaction[]> {
    const channelAddress = channelManager.getChannelAddress(channelId);
    if (!channelAddress) {
        throw new Error(`Channel with ID ${channelId} not found`);
    }
    return await fetchTransactions(channelAddress, client);
}

export {
    fetchTransactions,
    getChannelState,
    getChannelBalance,
    getChannelTransactions
};