import { useCallback, useState } from 'react';
import { 
  TonClient, 
  Address, 
  fromNano,
  type ContractProvider 
} from "@ton/ton";
import { getHttpEndpoint } from '@orbs-network/ton-access';
import { JettonMaster } from '@ton/ton';
import { JettonWallet } from '@/contractsTON/JettonsWallet';
import { delay } from 'framer-motion';

interface TokenData {
  symbol: string;
  balance: string;
  price?: number;
  address?: string;
  decimals?: number;
  isJetton?: boolean;
  metadata?: {
    name?: string;
    symbol?: string;
    image?: string;
    description?: string;
  };
}

interface JettonMetadata {
  uri?: string;
  name?: string;
  description?: string;
  image?: string;
  symbol?: string;
  decimals?: number;
}

export const useTokenService = () => {
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const delay = (ms: number) => new Promise(resolve => setTimeout(resolve, ms));

  const getClient = async (network: string): Promise<TonClient | null> => {
    try {
      const endpoint = await getHttpEndpoint({
        network: network === 'ton-mainnet' ? 'mainnet' : 'testnet'
      });
      return new TonClient({ endpoint });
    } catch (e) {
      console.error('Failed to initialize TON client:', e);
      return null;
    }
  };

  const parseJettonMetadata = (content: any): JettonMetadata => {
    try {
      if (typeof content === 'string' && content.slice(0, 4) === 'data') {
        const dataContent = content.slice(5);
        const parsed = JSON.parse(dataContent);
        return {
          name: parsed.name,
          symbol: parsed.symbol,
          description: parsed.description,
          image: parsed.image,
          decimals: parsed.decimals || 9
        };
      }
      return {};
    } catch (e) {
      console.warn('Error parsing Jetton metadata:', e);
      return {};
    }
  };

  const getJettonData = async (
    client: TonClient,
    jettonAddress: Address
  ): Promise<TokenData | null> => {
    try {
      const jettonMaster = JettonMaster.create(jettonAddress);
      const provider = client.provider(jettonAddress) as unknown as ContractProvider;
      const data = await jettonMaster.getJettonData(provider);
      
      const metadata = parseJettonMetadata(data.content.toString());
      
      return {
        symbol: metadata.symbol || 'Unknown Token',
        balance: '0', // Will be updated per wallet
        address: jettonAddress.toString(),
        decimals: metadata.decimals || 9,
        isJetton: true,
        metadata: {
          name: metadata.name,
          symbol: metadata.symbol,
          image: metadata.image,
          description: metadata.description
        }
      };
    } catch (e) {
      console.warn('Error fetching Jetton data:', e);
      return null;
    }
  };

  const getJettonBalance = async (
    client: TonClient,
    jettonMaster: JettonMaster,
    walletAddress: Address
  ): Promise<string> => {
    try {
      const provider = client.provider(
        jettonMaster.address
      ) as unknown as ContractProvider;
      const result = await jettonMaster.getWalletAddress(provider, walletAddress);
      const balance = result.balance;
      return fromNano(balance); 
    } catch (e) {
      console.warn("Error fetching Jetton balance:", e);
      return "0";
    }
  };


  const fetchTokensWithRetry = useCallback(async (network: string, address: string, retries: number = 3): Promise<TokenData[]> => {
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
        
        // Start with TON balance
        const tokens: TokenData[] = [{
          symbol: 'TON',
          balance: fromNano(tonBalance),
          decimals: 9,
          isJetton: false
        }];

        // Known Jetton addresses for the network
        const knownJettons = network === 'ton-mainnet' 
          ? [
              // Mainnet Jetton addresses can be added here
              'EQBCFwW8uFUh-amdRmNY9NyeDEaeDYXd9ggJGsicpqVcHq7B',
            ] 
          : [];

        // Fetch Jetton data and balances
        const jettonPromises = knownJettons.map(async (jettonAddress) => {
          try {
            const jettonAddressObj = Address.parse(jettonAddress);
            const jettonData = await getJettonData(client, jettonAddressObj);
            
            if (jettonData) {
              const jettonMaster = JettonMaster.create(jettonAddressObj);
              const balance = await getJettonBalance(client, jettonMaster, addressObj);
              return {
                ...jettonData,
                balance
              };
            }
          } catch (e) {
            console.warn(`Failed to fetch Jetton data for ${jettonAddress}:`, e);
          }
          return null;
        });

        const jettonResults = await Promise.allSettled(jettonPromises);
        const validJettons = jettonResults
          .filter((result): result is PromiseFulfilledResult<TokenData | null> => 
            result.status === 'fulfilled' && result.value !== null
          )
          .map(result => result.value!);

        tokens.push(...validJettons);

        setIsLoading(false);
        return tokens;
      } catch (error) {
        console.error(`Attempt ${attempt + 1} failed:`, error);
        if (attempt === retries - 1) {
          setError('Failed to load tokens. Please try again.');
          setIsLoading(false);
          // Return TON-only list as fallback
          return [{
            symbol: 'TON',
            balance: '0',
            decimals: 9,
            isJetton: false
          }];
        }
        await delay(1000 * (attempt + 1)); // Exponential backoff
      }
    }

    return [];
  }, []);

  return {
    fetchTokens: fetchTokensWithRetry,
    isLoading,
    error
  };
};

// GameBoy-style UI components
export const TokenItem = ({ 
  token, 
  selected 
}: { 
  token: TokenData; 
  selected: boolean;
}) => (
  <div 
    className={`p-2 ${
      selected ? 'bg-[#0f380f] text-[#9bbc0f]' : 'border border-[#0f380f]'
    }`}
  >
    <div className="flex justify-between items-center">
      <div className="flex flex-col">
        <span className="font-bold text-xs">{token.symbol}</span>
        {token.metadata?.name && (
          <span className="text-[0.6rem] opacity-75">{token.metadata.name}</span>
        )}
      </div>
      <div className="text-right">
        <div className="text-xs">{parseFloat(token.balance).toFixed(4)}</div>
        {token.isJetton && (
          <div className="text-[0.6rem] opacity-75">JETTON</div>
        )}
      </div>
    </div>
  </div>
);

export const TokenLoading = () => (
  <div className="flex flex-col items-center justify-center p-4">
    <div className="text-xs animate-pulse">LOADING TOKENS</div>
    <div className="mt-2 flex space-x-1">
      {[0, 1, 2].map((i) => (
        <div
          key={i}
          className="w-2 h-2 bg-[#0f380f] animate-bounce"
          style={{ animationDelay: `${i * 200}ms` }}
        />
      ))}
    </div>
  </div>
);

export const TokenError = ({ 
  message, 
  onRetry 
}: { 
  message: string; 
  onRetry: () => void;
}) => (
  <div className="flex flex-col items-center justify-center p-4">
    <div className="text-xs mb-4 px-4 py-2 bg-[#0f380f] text-[#9bbc0f]">
      {message}
    </div>
    <button
      onClick={onRetry}
      className="px-4 py-2 text-xs border border-[#0f380f] hover:bg-[#0f380f] hover:text-[#9bbc0f] transition-colors"
    >
      PRESS A TO RETRY
    </button>
  </div>
);