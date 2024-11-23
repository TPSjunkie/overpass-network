// src/utils/balanceCache.ts

/// This file contains the BalanceCache class, which is used to cache the balances of the user's channels.

/// It is used to reduce the number of requests to the Overpass API, which can be expensive.
import { GroupBalances }  from "../services/getGroupBalances";
import { GroupChannels } from "../services/getGroupChannels";

/// The BalanceCache class is a singleton, so it can be accessed from anywhere in the application.
export class BalanceCache {
    getBalances(arg0: string): import("react").ReactNode {
      throw new Error('Method not implemented.');
    }
    private static instance: BalanceCache;
    private balances: { [key: string]: number };
    private channels: { [key: string]: string[] };
    private constructor() {
        this.balances = {};
        this.channels = {};
    }
    public static getInstance(): BalanceCache {
        if (!BalanceCache.instance) {
            BalanceCache.instance = new BalanceCache();
        }
        return BalanceCache.instance;
    }
    public getBalance(channelId: string): number {
        return this.balances[channelId] || 0;
    }
    public getChannels(group: string): string[] {
        return this.channels[group] || [];
    }
    public setBalance(channelId: string, balance: number) {
        this.balances[channelId] = balance;
    }
    public setChannels(group: string, channels: string[]) {
        this.channels[group] = channels;
    }
}