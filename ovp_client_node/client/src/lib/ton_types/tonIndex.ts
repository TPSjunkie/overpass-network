// src/types/ton.d.ts
declare module '@tonconnect/sdk' {
    export interface WalletInfoRemote {
      name: string;
      universalLink: string;
      bridgeUrl: string;
    }
  
    export interface WalletInfoInjected {
      name: string;
      jsBridgeKey: string;
    }
  
    export type WalletInfo = WalletInfoRemote | WalletInfoInjected;
  
    export interface SendTransactionRequest {
      validUntil: number;
      messages: Array<{
        address: string;
        amount: string;
        payload?: string;
      }>;
    }
  }