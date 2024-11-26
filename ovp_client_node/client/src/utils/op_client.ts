import { type ChannelTransactionParams } from "./wasmTransactionSender";
import { type ContractProvider, type Sender, Address } from '@ton/core';
import type { WalletExtension, Transaction } from '../types/wasm-types';

import transactionFetcher from "./transactionFetcher";


export class OpClient {
    private walletExtension: WalletExtension | null = null;
    static instance: Promise<OpClient>;
    static ContractProvider: any;

    constructor() {}

    async open(walletExtension: WalletExtension) {
        this.walletExtension = walletExtension;
        return {
            getChannelState: async (provider: ContractProvider, channelId: number) => {
                if (!this.walletExtension) throw new Error("Wallet extension not initialized");
                return await this.walletExtension.get_channel(provider, channelId);
            },
            getBalance: async (provider: ContractProvider, channelId: number) => {
                if (!this.walletExtension) throw new Error("Wallet extension not initialized");
                return this.walletExtension.getChannelBalance(provider, channelId);
            },
            getChannelCount: async (provider: ContractProvider) => {
                if (!this.walletExtension) throw new Error("Wallet extension not initialized");
                return this.walletExtension.get_channel_count(provider);
            },
            sendTransaction: async (transaction: Transaction, provider: ContractProvider, params: ChannelTransactionParams) => {
                if (!this.walletExtension) {
                    throw new Error("Wallet extension not initialized");
                }
                return this.walletExtension.sendTransaction(
                    transaction,
                    provider,
                    params
                );
            },
            createChannel: async (                provider: ContractProvider,
                via: Sender,
                opts: {
                    channelId: number;
                    initialBalance: bigint;
                    group: number;
                    value: bigint;
                    queryId?: number;
                }
            ) => {
                if (!this.walletExtension)
                    throw new Error("Wallet extension not initialized");
                return this.walletExtension.sendCreateChannel(
                    provider,
                    via,
                    opts.channelId,
                    opts.initialBalance,
                    opts.group,
                    opts.value,
                    opts.queryId,
                    opts
                );
            },
            closeChannel: async (
                provider: ContractProvider,
                via: Sender,
                opts: {
                    channelId: number;
                    value: bigint;
                    queryId?: number;
                }
            ) => {
                if (!this.walletExtension)
                    throw new Error("Wallet extension not initialized");
                return this.walletExtension.sendCloseChannel(
                    provider,
                    via,
                    opts.channelId,
                    opts.value,
                    opts.queryId,
                    opts
                );
            },
            preAuthorize: async (
                provider: ContractProvider,
                via: Sender,
                opts: {
                    channelId: number;
                    intermediateAddress: Address;
                    value: bigint;
                    queryId?: number;
                }
            ) => {
                if (!this.walletExtension)
                    throw new Error("Wallet extension not initialized");
                return this.walletExtension.sendPreAuthorize(
                    provider,
                    via,
                    opts.channelId,
                    opts.intermediateAddress,
                    opts.value,
                    opts.queryId,
                    opts
                );
            },
            clearPending: async (
                provider: ContractProvider,
                via: Sender,
                opts: {
                    channelId: number;
                    value: bigint;
                    queryId?: number;
                }
            ) => {
                if (!this.walletExtension)
                    throw new Error("Wallet extension not initialized");
                return this.walletExtension.sendClearPending(
                    provider,
                    via,
                    opts.channelId,
                    opts.value,
                    opts.queryId,
                    opts
                );
            }
        };
    }

    async close() {
        this.walletExtension = null;
    }

    static async initialize() {
        const client = new OpClient();
        // Remove the call to open() here as it requires a WalletExtension parameter
        return client;
    }

    async fetchGroupBalances(provider: ContractProvider) {
        if (!this.walletExtension)
            throw new Error("Wallet extension not initialized");
        return this.walletExtension.fetchGroupBalances(provider);
    }

    async getWalletExtension(): Promise<WalletExtension | null> {
        return this.walletExtension;
    }
}
OpClient.instance = OpClient.initialize();

export const getOpClient = () => OpClient.instance;

export const getTransactionFetcher = () =>
    transactionFetcher;

export const getWalletExtension = async () => (await OpClient.instance).getWalletExtension();

export const getWalletExtensionOrThrow = async () => {
    const walletExtension = await (await OpClient.instance).getWalletExtension();
    if (!walletExtension) {
        throw new Error("Wallet extension not initialized");
    }
    return walletExtension;
};

export const getWalletExtensionOrNull = async () => (await OpClient.instance).getWalletExtension();

export const getWalletExtensionOrUndefined = async () => {
    const walletExtension = await (await OpClient.instance).getWalletExtension();
    return walletExtension || undefined;
};

export const getWalletExtensionOrDefault = async <T>(defaultValue: T): Promise<WalletExtension | T> => {
    const walletExtension = await (await OpClient.instance).getWalletExtension();
    return walletExtension || defaultValue;
};

export const getWalletExtensionOrDefaultAsync = async <T>(
    defaultValue: T
): Promise<WalletExtension | T> => {
    const walletExtension = await (await OpClient.instance).getWalletExtension();
    if (walletExtension) {
        return walletExtension;
    }
    return defaultValue;
};

export const getWalletExtensionOrDefaultOrThrow = async <T>(
    defaultValue: T
): Promise<WalletExtension | T> => {
    const walletExtension = await (await OpClient.instance).getWalletExtension();
    if (walletExtension) {
        return walletExtension;
    }
    throw new Error("Wallet extension not initialized");
};

export const getWalletExtensionOrDefaultOrThrowAsync = async <T>(
    defaultValue: T
): Promise<WalletExtension | T> => {
    const walletExtension = await (await OpClient.instance).getWalletExtension();
    if (walletExtension) {
        return walletExtension;
    }
    throw new Error("Wallet extension not initialized");
};

export const getWalletExtensionOrDefaultOrNull = async <T>(
    defaultValue: T
): Promise<WalletExtension | T | null> => {    
    const walletExtension = await (await OpClient.instance).getWalletExtension();
    if (walletExtension) {
        return walletExtension;
    }
    return defaultValue;
};
