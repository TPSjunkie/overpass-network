// src/utils/wasmTransactionSender.ts

import type { Transaction, WalletExtension } from "@/types/wasm-types/index";
import { getHttpEndpoint } from "@orbs-network/ton-access";

import { Cell } from "recharts";

// Get the WASM module path
const wasmPath = /* @vite-ignore */ new URL(
  '@/wasm/overpass_rs_bg.wasm',
  import.meta.url
).toString();

interface WasmInstance {
  send_transaction(to: string, arg1: any, data: any, channelId: string, groupId: string, transactionType: TransactionType): unknown;
  open_wallet(name: any): unknown;
  memory: WebAssembly.Memory;
  initiate_transaction: (
    channelId: string,
    messageCell: string,
    keyPair: any,
    transactionType: number,
    groupId: string
  ) => Promise<string>;
  init_panic_hook: () => void;
}

let wasmInstance: WasmInstance | null = null;

async function loadWasm(): Promise<WasmInstance> {
  if (wasmInstance) return wasmInstance;

  try {
    const response = await fetch(wasmPath);
    const wasmBuffer = await response.arrayBuffer();
    const wasmModule = await WebAssembly.instantiate(wasmBuffer, {
      env: {
        memory: new WebAssembly.Memory({ initial: 256 }),
      },
    });

    wasmInstance = wasmModule.instance.exports as unknown as WasmInstance;
    return wasmInstance;
  } catch (error) {
    console.error('Failed to load WASM module:', error);
    throw error;
  }
}

export interface TransactionParams {
  recipient: string;
  amount: string;
  payload?: string;
  stateInit?: string;
  flags?: string;
  bounce?: boolean;
  channelId?: string;
  groupId?: string;
  transactionType?: TransactionType;
  queryId?: string;
  timeout?: number;
  message?: string;
  messageId?: string;
  messageBody?: string;
  messageType?: string;
}
export interface ChannelTransactionParams extends TransactionParams {
  channelId: string;
  groupId: string;
  transactionType: TransactionType;
  queryId?: string;
  timeout?: number;
  message?: string;
  messageType?: string;
  messageId?: string;
  messageBody?: string;
}
export enum TransactionType {
  REGULAR = 0,
  CHANNEL_INIT = 1,
  CHANNEL_CLOSE = 2,
  CHANNEL_WITHDRAW = 3
}

export class WasmError extends Error {
  code?: number;
  details?: string;
}

export class WasmTransactionSender {
    beginCell: any;
    static sendTransaction(transaction: Transaction, params: ChannelTransactionParams) {
        throw new Error("Method not implemented.");
    }
    static create(walletExtension: any): WasmTransactionSender | PromiseLike<WasmTransactionSender | null> | null {
        throw new Error("Method not implemented.");
    }
    private static instance: WasmTransactionSender;
    private wasmModuleInitialized: boolean = false;
    private initializationPromise: Promise<void> | null = null;

    private constructor() {
        // Private constructor to prevent direct instantiation
        this.wasmModuleInitialized = false;
        this.initializationPromise = null;
    }

    public static getInstance(): WasmTransactionSender {
      if (!WasmTransactionSender.instance) {
        WasmTransactionSender.instance = new WasmTransactionSender();
      }
      return WasmTransactionSender.instance;
    }

    public async open<T extends WalletExtension>(walletExtension: T): Promise<void> {
      await this.initializeWasmModule();

      if (!wasmInstance) {
        throw new Error('WASM module not initialized');
      }

      try {
        const result = wasmInstance.open_wallet(walletExtension.name);
        if (result !== 0) {
          throw new WasmError('Failed to open wallet extension');
        }
      } catch (error) {
        console.error('Error opening wallet extension:', error);
        throw error;
      }
    }

    private async initializeWasmModule(): Promise<void> {
      if (this.wasmModuleInitialized) return;
    
      if (!this.initializationPromise) {
        this.initializationPromise = new Promise(async (resolve, reject) => {
          try {
            await loadWasm();
            if (wasmInstance?.init_panic_hook) {
              wasmInstance.init_panic_hook();
            }
            this.wasmModuleInitialized = true;
            resolve();
          } catch (error) {
            console.error('WASM initialization error:', error);
            reject(new Error('Failed to initialize WASM module'));
          }
        });
      }

      return this.initializationPromise;
    }

    private validateTransaction(params: ChannelTransactionParams): void {
      if (!params.recipient) {
        throw new Error('Recipient address is required');
      }

      if (BigInt(params.amount) <= BigInt(0)) {
        throw new Error('Amount must be greater than 0');
      }

      if (!params.channelId || !params.groupId) {
        throw new Error('Channel ID and Group ID are required');
      }
    }

    private createMessageCell(params: ChannelTransactionParams): typeof Cell | undefined {
      if (!params.message) {
        return undefined;
      }
      const message = params.message;
      const messageType = params.messageType || 'text';
      const messageId = params.messageId || '';
      const messageBody = params.messageBody || '';
      if (messageType === 'text') {
        return this.beginCell()
          .storeUint(0, 32)
          .storeString(message)
          .endCell();
      } else if (messageType === 'binary') {
        return this.beginCell()
          .storeUint(1, 32)
          .storeBuffer(Buffer.from(message, 'base64'))
          .endCell();
      } else if (messageType === 'json') {
        return this.beginCell()
          .storeUint(2, 32)
          .storeString(message)
          .endCell();
      }
    }
private createCell(params: ChannelTransactionParams): typeof Cell {
    const cell = this.beginCell()
      .storeUint(params.transactionType, 8)
      .storeString(params.recipient)
      .storeUint(BigInt(params.amount), 16)
      .storeString(params.channelId)
      .storeString(params.groupId)
      .storeString(params.queryId || '')
      .storeUint(params.timeout || 0, 32)
      .storeString(params.messageId || '')
      .storeUint(params.messageType === 'json' ? 2 : params.messageType === 'binary' ? 1 : 0, 8)
      .storeString(params.message || '')
      .storeString(params.messageBody || '')
      .storeString(params.messageId || '')
      .storeString(params.messageType || '')
      .storeString(params.messageBody || '');
    return cell.endCell();
  }
}