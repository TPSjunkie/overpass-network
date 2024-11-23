import { beginCell, TonClient } from '@ton/ton';
import { Address, Cell } from '@ton/ton';
import { getHttpEndpoint } from '@orbs-network/ton-access';import axios from 'axios';

const RETRY_ATTEMPTS = 3;
const RETRY_DELAY = 1000;

interface ApiConfig {
  network: 'mainnet' | 'testnet';
  timeout?: number;
}

interface Stack {
  readBigNumber(): bigint;
  readString(): string;
  remaining: number;
}

interface MethodResponse {
  stack: Stack;
}

interface TokenBalance {
  balance: string;
  decimals: number;
}

interface TokenData {
  symbol: string;
  balance: string;
}

interface Transaction {
  id: string;
  type: 'in' | 'out';
  amount: string;
  address: string;
  timestamp: number;
}

interface RunMethodResult {
  type: string;
  exit_code: number;
  gas_consumed: number;
  stack: {
    type: string;
    value: string;
  }[];
}

class TonApiClient {
  private client: TonClient | null = null;
  private retryCount = 0;

  async initialize(config: ApiConfig): Promise<TonClient> {
    try {
      const endpoint = await getHttpEndpoint({
        network: config.network
      });

      this.client = new TonClient({
        endpoint
      });
      
      return this.client;
    } catch (error) {
      console.error('Failed to initialize TON client:', error);
      throw new Error('Failed to initialize TON client');
    }
  }

  private async retry<T>(operation: () => Promise<T>): Promise<T> {
    try {
      return await operation();
    } catch (error) {
      if (this.retryCount < RETRY_ATTEMPTS && this.isRetryableError(error)) {
        console.warn(`Retrying operation, attempt ${this.retryCount + 1}`);
        this.retryCount++;
        await new Promise(resolve => setTimeout(resolve, RETRY_DELAY * this.retryCount));
        return this.retry(operation);
      }
      throw error;
    }
  }

  private isRetryableError(error: unknown): boolean {
    return (
      axios.isAxiosError(error) &&
      (error.response?.status === 500 || 
       error.response?.status === 503 || 
       error.code === 'ECONNABORTED' ||
       error.code === 'ETIMEDOUT')
    );
  }

  private convertToMethodResponse(result: RunMethodResult): MethodResponse {
    return {
      stack: {
        readBigNumber: () => BigInt(result.stack[0]?.value || '0'),
        readString: () => result.stack[0]?.value || '',
        remaining: result.stack.length
      }
    };
  }

  private async getTokenBalance(tokenContract: Address, walletAddress: Address): Promise<TokenBalance> {
    if (!this.client) throw new Error('Client not initialized');
    
    const methodResponse = await this.client.runMethod(tokenContract, 'balanceOf', [{ type: 'slice', cell: beginCell().storeAddress(walletAddress).endCell() }]) as unknown as RunMethodResult;
    const convertedResponse = this.convertToMethodResponse(methodResponse);
    const balance = convertedResponse.stack.readBigNumber();
    const decimals = convertedResponse.stack.readBigNumber();
    
    return {
      balance: balance.toString(),
      decimals: Number(decimals.toString())
    };
  }
  async getTokens(address: string): Promise<TokenData[]> {
    if (!this.client) throw new Error('Client not initialized');

    return this.retry(async () => {
      const addressObj = Address.parse(address);
      const transactions = await this.client!.getTransactions(addressObj, { limit: 50 });

      const tokens: TokenData[] = [];
      const processedAddresses = new Set<string>();

      for (const tx of transactions) {
        if (tx.inMessage?.info.type === 'internal' && tx.inMessage.info.src) {
          const tokenAddress = tx.inMessage.info.src.toString();
          if (!processedAddresses.has(tokenAddress)) {
            try {
              const tokenContract = Address.parse(tokenAddress);
              const result = await this.client!.runMethod(tokenContract, 'get_token_data', []) as unknown as RunMethodResult;
              const tokenData = this.convertToMethodResponse(result);

              if (tokenData.stack.remaining > 0) {
                const tokenBalance = await this.getTokenBalance(tokenContract, addressObj);
                tokens.push({
                  symbol: tokenData.stack.readString() || 'Unknown Token',
                  balance: tokenBalance.balance
                });
              }
              processedAddresses.add(tokenAddress);
            } catch (e) {
              console.warn('Error processing token:', e);
            }
          }
        }
      }
      return tokens;
    });  }

  async getBalance(address: string): Promise<string> {
    if (!this.client) throw new Error('Client not initialized');

    return this.retry(async () => {
      const addressObj = Address.parse(address);
      const balance = await this.client!.getBalance(addressObj);
      return (Number(balance) / 1e9).toFixed(2);
    });
  }

  async getTransactions(address: string): Promise<Transaction[]> {
    if (!this.client) throw new Error('Client not initialized');

    return this.retry(async () => {
      const addressObj = Address.parse(address);
      const txs = await this.client!.getTransactions(addressObj, { limit: 20 });
      
      return txs.map(tx => {
        const isIncoming = tx.inMessage?.info.type === "internal";
        const amount = isIncoming
          ? tx.inMessage?.info.value
          : tx.outMessages?.[0]?.info.value;
        return {
          id: tx.hash.toString(),
          type: isIncoming ? 'in' : 'out',
          amount: (Number(amount) / 1e9).toFixed(2),
          address: isIncoming 
            ? tx.inMessage?.info.src?.toString() || 'Unknown'
            : tx.outMessages?.[0]?.info.dest?.toString() || 'Unknown',
          timestamp: tx.now * 1000
        };
      });
    });
  }
}

export const tonApiClient = new TonApiClient();
