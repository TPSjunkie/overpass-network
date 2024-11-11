// ./src/utils/stateManager.ts
import { writable } from "svelte/store";
export const state = writable({
    isConnected: false,
    walletAddress: null,
    balance: 0,
    transactions: [],
    channels: [],
    groups: {
        checking: [],
        savings: [],
        custom: [],
    },
});
