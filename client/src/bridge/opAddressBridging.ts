import { 
  Address, 
  Cell, 
  toNano, 
  beginCell
} from '@ton/core';
import type { SendMode, StateInit, Contract } from '@ton/core';
import { 
  TonClient, 
  WalletContractV4
} from '@ton/ton';
import type { Transaction } from "../wasm/overpass_rs";
import type { WalletExtension } from '@/types/wasm-types';

interface TransactionOpts {
  limit: number;
  lt?: string;
  hash: string;
  to?: string;
  inclusive?: boolean;
  archival?: boolean;
}

interface WalletContract extends Contract {
  createTransfer(params: {
    secretKey: Buffer;
    seqno: number;
    messages: Array<{
      to: Address | string;
      value: bigint;
      body?: Cell;
      bounce?: boolean;
      sendMode?: number;
    }>;
    sendMode?: number;
  }): Cell;
}

interface OpenedWallet {
  address: Address;
  init?: { code: Cell; data: Cell };
  createStateInit(): StateInit;
  createTransfer(params: {
    secretKey: Buffer;
    seqno: number;
    messages: Array<{
      to: Address | string;
      value: bigint;
      body?: Cell;
      bounce?: boolean;
      sendMode?: number;
    }>;
    sendMode?: number;
  }): Cell;
  getState(): Promise<{
    seqno: number;
    lastTransaction?: {
      hash: string;
      lt: string;
    };
  } | null>;
  sendTransfer(transfer: Cell): Promise<{
    transaction: {
      hash: string;
      lt: string;
    };
  }>;
}

interface TransactionResponse {
  hash: string;
  status: 'pending' | 'completed' | 'failed';
}

interface WalletTransfer {
  to: Address | string;
  value: bigint;
  body?: Cell;
  bounce?: boolean;
  sendMode?: number;
}

interface WalletState {
  seqno: number;
  balance: bigint;
  lastTransaction?: string;
}

const TON_CLIENT_CONFIG = {
  endpoint: 'https://toncenter.com/api/v2/jsonRPC',
  timeout: 30000,
  maxRetries: 3
} as const;

const SEND_MODES = {
  CARRY_ALL_REMAINING_BALANCE: 128,
  CARRY_ALL_REMAINING_INCOMING_VALUE: 64,
  DESTROY_IF_ZERO: 32,
  PAY_GAS_SEPARATELY: 1,
  IGNORE_ERRORS: 2
} as const;

const DEFAULT_TRANSACTION_OPTS: TransactionOpts = {
  limit: 1,
  hash: '',
  lt: undefined,
  to: undefined,
  inclusive: true,
  archival: false
};

class TONWalletManager {
  private readonly client: TonClient;
  
  constructor(config = TON_CLIENT_CONFIG) {
    this.client = new TonClient(config);
  }

  private createInternalMessage(params: WalletTransfer): {
    to: Address;
    value: bigint;
    body?: Cell;
    bounce: boolean;
    sendMode?: number;
  } {
    const to = typeof params.to === 'string' ? Address.parse(params.to) : params.to;
    return {
      to,
      value: params.value,
      bounce: params.bounce ?? true,
      body: params.body,
      sendMode: params.sendMode ?? SEND_MODES.PAY_GAS_SEPARATELY
    };
  }

  private createWalletInstance(publicKey: Buffer, workchain = 0): OpenedWallet {
    const wallet = WalletContractV4.create({
      publicKey: publicKey.subarray(32),
      workchain
    }) as unknown as OpenedWallet;
    
    return wallet;
  }

  async opAddressBridgeWithPayload(
    walletExtension: WalletExtension,
    addressStr: string
  ): Promise<TransactionResponse> {
    if (!walletExtension?.address) {
      throw new Error('Invalid wallet extension or address');
    }

    const wallet = this.createWalletInstance(
      Buffer.from(walletExtension.address.toString(), 'hex')
    );
    
    const state = await wallet.getState();
    
    if (!state) {
      throw new Error('Failed to get wallet state');
    }

    const transfer = wallet.createTransfer({
      secretKey: Buffer.from(walletExtension.address.toString(), 'hex'),
      seqno: state.seqno,
      messages: [this.createInternalMessage({
        to: addressStr,
        value: toNano('0.1'),
        body: beginCell().endCell()
      })],
      sendMode: SEND_MODES.PAY_GAS_SEPARATELY
    });

    const response = await wallet.sendTransfer(transfer);
    
    if (!response?.transaction?.hash) {
      throw new Error('Transaction failed to process');
    }
    
    return {
      hash: response.transaction.hash,
      status: 'pending'
    };
  }

  async generateTONAddress(secretKey: Buffer): Promise<string> {
    if (!secretKey || secretKey.length < 32) {
      throw new Error('Invalid secret key');
    }
    
    const wallet = this.createWalletInstance(secretKey);
    return wallet.address.toString();
  }

  async signTransaction(
    transactionData: Cell | Promise<Cell>,
    secretKey: Buffer,
    toAddress: Address,
    value: bigint = toNano('1')
  ): Promise<TransactionResponse> {
    try {
      const resolvedCell = transactionData instanceof Cell ? 
        transactionData : 
        await transactionData;

      const wallet = this.createWalletInstance(secretKey);
      const state = await wallet.getState();
      
      if (!state) {
        throw new Error('Failed to get wallet state');
      }

      const transfer = wallet.createTransfer({
        secretKey,
        seqno: state.seqno,
        messages: [this.createInternalMessage({
          to: toAddress,
          value,
          body: resolvedCell
        })],
        sendMode: SEND_MODES.PAY_GAS_SEPARATELY
      });

      const response = await wallet.sendTransfer(transfer);
      
      if (!response?.transaction?.hash) {
        throw new Error('Transaction signing failed');
      }

      return {
        hash: response.transaction.hash,
        status: 'pending'
      };
    } catch (error) {
      console.error('Error signing transaction:', error);
      throw error instanceof Error ? error : new Error('Unknown error during transaction signing');
    }
  }

  async checkTransactionStatus(hash: string): Promise<Transaction[]> {
    if (!this.validateTONAddress(hash)) {
      throw new Error('Invalid transaction hash format');
    }

    try {
      const parsedAddress = Address.parse(hash);
      
      const result = await this.client.getTransactions(
        parsedAddress,
        DEFAULT_TRANSACTION_OPTS
      );
      
      return result as unknown as Transaction[];
    } catch (error) {
      console.error("Error checking transaction status:", error);
      throw error instanceof Error ? error : new Error('Failed to check transaction status');
    }
  }

  async createWalletContract(secretKey: Buffer): Promise<OpenedWallet> {
    if (!secretKey || secretKey.length < 32) {
      throw new Error('Invalid secret key');
    }

    return this.createWalletInstance(secretKey);
  }

  async getWalletBalance(address: string | Address): Promise<bigint> {
    try {
      const parsedAddress = typeof address === 'string' ? 
        Address.parse(address) : 
        address;

      return await this.client.getBalance(parsedAddress);
    } catch (error) {
      console.error("Error getting wallet balance:", error);
      throw error instanceof Error ? error : new Error('Failed to get wallet balance');
    }
  }

  validateTONAddress(addressStr: string): boolean {
    try {
      Address.parse(addressStr);
      return true;
    } catch {
      return false;
    }
  }

  async getWalletState(wallet: OpenedWallet): Promise<WalletState> {
    const [state, balance] = await Promise.all([
      wallet.getState(),
      this.getWalletBalance(wallet.address)
    ]);

    if (!state) {
      throw new Error('Failed to get wallet state');
    }

    return {
      seqno: state.seqno,
      balance,
      lastTransaction: state.lastTransaction?.hash
    };
  }

  static formatNano(amount: bigint): string {
    return `${amount.toString()} TON`;
  }

  static parseNano(amount: string): bigint {
    return toNano(amount);
  }
}

// Export singleton instance
export const tonWallet = new TONWalletManager();

// Export types
export type { 
  TransactionResponse,
  WalletTransfer,
  WalletState,
  OpenedWallet,
  TransactionOpts
};

// Export constants
export {
  TON_CLIENT_CONFIG,
  SEND_MODES,
  DEFAULT_TRANSACTION_OPTS
};