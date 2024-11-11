import { useState, useCallback } from 'react';
import { 
  TonClient, 
  Address, 
  type Transaction as TonTransaction,
  Cell,
  beginCell as tonBeginCell,
} from "@ton/ton";
import { getHttpEndpoint } from '@orbs-network/ton-access';

interface Token {
  symbol: string;
  balance: string;
}

interface SimplifiedTransaction {
  id: string;
  type: 'in' | 'out';
  amount: string;
  address: string;
  timestamp: number;
  lt?: string;
}

const delay = (ms: number) => new Promise(resolve => setTimeout(resolve, ms));

export const useTonOperations = () => {
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const getClient = useCallback(async (network: string): Promise<TonClient | null> => {
    try {
      const endpoint = await getHttpEndpoint({
        network: network === 'ton-mainnet' ? 'mainnet' : 'testnet'
      });
      return new TonClient({ endpoint });
    } catch (e) {
      console.error('Failed to initialize TON client:', e);
      return null;
    }
  }, []);

  const readValue = (stack: any): string => {
    try {
      if (stack && typeof stack.readBigNumber === 'function') {
        return stack.readBigNumber().toString();
      }
      return '0';
    } catch (e) {
      console.warn('Error reading stack value:', e);
      return '0';
    }
  };

  const fetchTokensWithRetry = useCallback(async (
    address: string,
    network: string,
    retries = 3
  ): Promise<Token[]> => {
    setIsLoading(true);
    setError(null);

    for (let attempt = 0; attempt < retries; attempt++) {
      try {
        const client = await getClient(network);
        if (!client) {
          throw new Error('Failed to initialize TON client');
        }

        const addressObj = Address.parse(address);
        const tonBalance = await client.getBalance(addressObj);
        const tokens: Token[] = [{
          symbol: 'TON',
          balance: (Number(tonBalance) / 1e9).toString()
        }];

        try {
          const transactions = await client.getTransactions(addressObj, { limit: 50 });
          const processedAddresses = new Set<string>();

          for (const tx of transactions) {
            if (tx.inMessage?.body instanceof Cell) {
              try {
                const msgBody = tx.inMessage.body.toBoc().toString();
                if (msgBody.includes('transfer') || msgBody.includes('mint')) {
                  const tokenAddress = tx.inMessage.info.src?.toString();
                  if (tokenAddress && !processedAddresses.has(tokenAddress)) {
                    const tokenContract = Address.parse(tokenAddress);
                    const tokenData = await client.runMethod(tokenContract, 'get_token_data');
                    
                    // Read symbol and decimals from stack
                    const symbol = readValue(tokenData.stack);
                    const decimals = parseInt(readValue(tokenData.stack)) || 9;
                    
                    // Create wallet address cell
                    const walletAddressCell = tonBeginCell()
                      .storeAddress(addressObj)
                      .endCell();

                    const balance = await client.runMethod(tokenContract, 'get_wallet_balance', [{
                      type: 'slice',
                      cell: walletAddressCell
                    }]);

                    // Read balance from stack
                    const balanceValue = readValue(balance.stack);
                    
                    tokens.push({
                      symbol: symbol || "Unknown Token",
                      balance: (Number(balanceValue) / Math.pow(10, decimals)).toString(),
                    });
                    
                    processedAddresses.add(tokenAddress);
                  }
                }
              } catch (e) {
                console.warn('Error processing token:', e);
              }
            }
          }
        } catch (e) {
          console.warn('Error fetching transactions for token discovery:', e);
        }

        setIsLoading(false);
        return tokens;
      } catch (error) {
        console.error(`Attempt ${attempt + 1} failed:`, error);
        if (attempt === retries - 1) {
          setError('Failed to load tokens. Please try again.');
          setIsLoading(false);
          return [{
            symbol: 'TON',
            balance: '0'
          }];
        }
        await delay(1000 * (attempt + 1));
      }
    }

    return [];
  }, [getClient]);

  const fetchTransactionsWithRetry = useCallback(async (
    address: string,
    network: string,
    retries = 3
  ): Promise<SimplifiedTransaction[]> => {
    setIsLoading(true);
    setError(null);

    for (let attempt = 0; attempt < retries; attempt++) {
      try {
        const client = await getClient(network);
        if (!client) {
          throw new Error('Failed to initialize TON client');
        }

        const txs = await client.getTransactions(Address.parse(address), { limit: 20 });
        
        const formattedTxs: SimplifiedTransaction[] = txs.map((tx) => {
          const isIncoming = tx.inMessage?.info.dest?.toString() === address;
          let amount = '0';

          if (isIncoming && tx.inMessage?.info.type === 'internal') {
            amount = tx.inMessage.info.value?.toString() || '0';
          } else if (!isIncoming && tx.outMessages?.[0]?.info.type === 'internal') {
            amount = tx.outMessages[0].info.value?.toString() || '0';
          }

          return {
            id: tx.hash.toString(),
            type: isIncoming ? 'in' : 'out',
            amount: (Number(amount) / 1e9).toFixed(2),
            address: isIncoming 
              ? tx.inMessage?.info.src?.toString() || 'Unknown'
              : tx.outMessages?.[0]?.info.dest?.toString() || 'Unknown',
            timestamp: tx.now,
            lt: tx.lt.toString()
          };
        });
        setIsLoading(false);
        return formattedTxs;
      } catch (error) {
        console.error(`Attempt ${attempt + 1} failed:`, error);
        if (attempt === retries - 1) {
          setError('Failed to load transactions. Please try again.');
          setIsLoading(false);
          return [];
        }
        await delay(1000 * (attempt + 1));
      }
    }

    return [];
  }, [getClient]);

  const fetchBalanceWithRetry = useCallback(async (
    address: string,
    network: string,
    retries = 3
  ): Promise<string> => {
    setIsLoading(true);
    setError(null);

    for (let attempt = 0; attempt < retries; attempt++) {
      try {
        const client = await getClient(network);
        if (!client) {
          throw new Error('Failed to initialize TON client');
        }

        const balanceValue = await client.getBalance(Address.parse(address));
        setIsLoading(false);
        return (Number(balanceValue) / 1e9).toFixed(2);
      } catch (error) {
        console.error(`Attempt ${attempt + 1} failed:`, error);
        if (attempt === retries - 1) {
          setError('Failed to load balance. Please try again.');
          setIsLoading(false);
          return '0';
        }
        await delay(1000 * (attempt + 1));
      }
    }

    return '0';
  }, [getClient]);

  return {
    fetchTokens: fetchTokensWithRetry,
    fetchTransactions: fetchTransactionsWithRetry,
    fetchBalance: fetchBalanceWithRetry,
    isLoading,
    error
  };
};

export { RetryableError, RetroLoading } from './Components';