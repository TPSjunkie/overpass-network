import type { WalletInfo } from '@tonconnect/sdk';
import { useEffect, useState, useCallback, useRef } from 'react';
import AudioPlayer from './components/AudioPlayer';
import { useAudio } from './hooks/useAudio';
import { TonConnectUI } from '@tonconnect/ui-react';
import { getHttpEndpoint } from '@orbs-network/ton-access';
import { init_panic_hook, initiate_transaction} from '@wasm/overpass_rs';
import { BackendApi } from './wasm/api';
import { Address, TonClient } from '@ton/ton';

type MenuScreen = 'main' | 'send' | 'receive' | 'tokens' | 'wallets' | 'channels' | 'channelSettings' | 'transactions' | 'settings' | 'market';

type Token = {
  symbol: string;
  balance: string;
  price?: number;
};

type Transaction = {
  inMessage: any;
  outMessages: any;
  hash: any;
  now: number;
  id: string;
  type: 'in' | 'out';
  amount: string;
  address: string;
  timestamp: number;
};

type NetworkOption = {
  name: string;
  network: string;
};

type MenuItem = {
  label: string;
  screen?: MenuScreen;
  action?: () => void;
};

type InputFocus = 'amount' | 'address' | 'none';

const NETWORKS: NetworkOption[] = [
  { name: "TON Mainnet", network: "ton-mainnet" },
  { name: "TON Testnet", network: "ton-testnet" },
  { name: "Overpass Mainnet", network: "overpass-mainnet" },
  { name: "Overpass Testnet", network: "overpass-testnet" }
];

const backendApi = new BackendApi(process.env.REACT_APP_API_URL || '');
export const App = () => {
  // State declarations
  const { isAudioOn, volume, toggleAudio } = useAudio();
  const [connector, setConnector] = useState<TonConnectUI | null>(null);
  const [wallets, setWallets] = useState<WalletInfo[]>([]);
  const [connected, setConnected] = useState(false);
  const [userAddress, setUserAddress] = useState('');
  const [error, setError] = useState('');
  const [selectedIndex, setSelectedIndex] = useState(0);
  const [selectedSection, setSelectedSection] = useState<'networks' | 'wallets'>('networks');
  const [balance, setBalance] = useState('0');
  const [tokens, setTokens] = useState<Token[]>([]);
  const [transactions, setTransactions] = useState<Transaction[]>([]);
  const [currentScreen, setCurrentScreen] = useState<MenuScreen>('main');
  const [sendAmount, setSendAmount] = useState('');
  const [sendAddress, setSendAddress] = useState('');
  const [copied, setCopied] = useState(false);
  const [currentNetwork, setCurrentNetwork] = useState<string>('ton-mainnet');
  const [isLoading, setIsLoading] = useState(false);
  const [inputFocus, setInputFocus] = useState<InputFocus>('none');
  const [wasmInitialized, setWasmInitialized] = useState(false);
  
  // Sound effects refs and states
  const [playCoinSound, setPlayCoinSound] = useState(false);
  const [playLvlSound, setPlayLvlSound] = useState(false);
  const [playPowerupSound, setPlayPowerupSound] = useState(false);
  const [playPipeSound, setPlayPipeSound] = useState(false);
  const [playJumpSound, setPlayJumpSound] = useState(false);
  
  const coinSoundRef = useRef<HTMLAudioElement>(null);
  const lvlSoundRef = useRef<HTMLAudioElement>(null);
  const powerupSoundRef = useRef<HTMLAudioElement>(null);
  const pipeSoundRef = useRef<HTMLAudioElement>(null);
  const jumpSoundRef = useRef<HTMLAudioElement>(null);
    // Sound effect handlers
    const triggerCoinSound = useCallback(() => {
      if (isAudioOn) setPlayCoinSound(true);
    }, [isAudioOn]);
  
    const triggerPowerupSound = useCallback(() => {
      if (isAudioOn) setPlayPowerupSound(true);
    }, [isAudioOn]);
  
    const triggerPipeSound = useCallback(() => {
      if (isAudioOn) setPlayPipeSound(true);
    }, [isAudioOn]);
  
    const triggerJumpSound = useCallback(() => {
      if (isAudioOn) setPlayJumpSound(true);
    }, [isAudioOn]);
  
    // Menu items with WASM integration
    const menuItems: MenuItem[] = connected ? [
      { label: "WALLET", screen: "wallets" },
      { label: "SEND", screen: 'send' },
      { label: 'RECEIVE', screen: 'receive' },
      { label: 'TOKENS', screen: 'tokens' },
      { label: 'MARKET', screen: 'market' },    
      { 
        label: 'CHANNELS', 
        screen: 'channels',
        action: async () => {
          if (wasmInitialized) {
            try {
              const channelId = "test-channel";
              const messageCell = "test-message";
                          const keyPair = {
                            publicKey: new Uint8Array(32),
                            secretKey: new Uint8Array(64)
                          };              const result = await initiate_transaction(
                channelId,
                messageCell,
                keyPair,
                1,
                "test-group"
              );
              console.log("Channel transaction initiated:", result);
            } catch (e) {
              console.error("Channel creation failed:", e);
            }
          }
        }
      }, 
      { label: "TRANSACTIONS", screen: "transactions" },
      { label: 'DISCONNECT', action: () => handleDisconnect() }
    ] : [];
    // Token fetching with WASM integration
    const fetchTokens = useCallback(async (address: string): Promise<Token[]> => {
      try {
        setIsLoading(true);
        const endpoint = await getHttpEndpoint({
          network: currentNetwork === 'ton-mainnet' ? 'mainnet' : 'testnet',
        });
        const tonClientInstance = new TonClient({ endpoint });
        const addressObj = Address.parse(address);
        const transactions = await tonClientInstance.getTransactions(addressObj, { limit: 100 });
  
        const tokens: Token[] = []; 
        const processedAddresses = new Set<string>();
  
        for (const tx of transactions) {
          const infoValue = tx.inMessage?.info?.value;
          const msgBody = tx.inMessage?.body;
          
          if (msgBody && typeof msgBody === 'object' && 'beginParse' in msgBody && infoValue) {
            const slice = msgBody.beginParse();
            const bufferLength = Math.floor(slice.bits.length / 8);
            const buffer = slice.loadBuffer(bufferLength);
            const msgText = buffer.toString();
  
            if (msgText.includes('transfer') || msgText.includes('mint')) {
              const tokenAddress = tx.inMessage?.info?.src?.toString();
              if (tokenAddress && !processedAddresses.has(tokenAddress)) {
                try {
                  const tokenContract = Address.parse(tokenAddress);
                  const tokenData = await tonClientInstance.runMethod(tokenContract, 'get_token_data', []);
                  
                  if (tokenData.stack && Array.isArray(tokenData.stack)) {
                    const symbol = tokenData.stack[0]?.value?.toString() || 'Unknown Token';
                    const decimals = parseInt(tokenData.stack[1]?.value?.toString() || '0', 10);
  
                    const balanceData = await tonClientInstance.runMethod(
                      tokenContract,
                      'get_wallet_data',
                      []
                    );
  
                    if (balanceData.stack && Array.isArray(balanceData.stack)) {
                      const balance = balanceData.stack[0]?.value?.toString() || '0';
                      tokens.push({
                        symbol,
                        balance: (Number(balance) / Math.pow(10, decimals)).toString()
                      });
                      processedAddresses.add(tokenAddress);
                    }
                  }
                } catch (e) {
                  console.warn("Error processing token:", e);
                }
              }
            }
          }
        }
  
        const tonBalance = await tonClientInstance.getBalance(addressObj);
        tokens.unshift({
          symbol: "TON",
          balance: (Number(tonBalance) / 1e9).toString()
        });
  
        setTokens(tokens);
        return tokens;
      } catch (error) {
        console.error("Error fetching tokens:", error);
        return [];
      } finally {
        setIsLoading(false);
      }
    }, [currentNetwork]);
    // Transaction fetching with WASM integration
    const fetchTransactions = useCallback(async (address: string) => {
      try {
        setIsLoading(true);
        const endpoint = await getHttpEndpoint({
          network: currentNetwork === 'ton-mainnet' ? 'mainnet' : 'testnet'
        });
        const tonClient = new TonClient({ endpoint });
        const txs = await tonClient.getTransactions(Address.parse(address), { limit: 20 });
  
        const formattedTxs = txs.map((tx) => {
          const isIncoming = tx.inMessage?.info?.dest?.toString() === address;
          const amount = isIncoming 
            ? tx.inMessage?.info?.value?.toString() || '0'
            : tx.outMessages?.[0]?.info?.value?.toString() || '0';
    
          return {
            id: tx.hash.toString(),
            type: isIncoming ? 'in' : 'out',
            amount: (Number(amount) / 1e9).toFixed(2),
            address: isIncoming 
              ? tx.inMessage?.info?.src?.toString() || 'Unknown'
              : tx.outMessages?.[0]?.info?.dest?.toString() || 'Unknown',
            timestamp: tx.now * 1000,
            inMessage: tx.inMessage,
            outMessages: tx.outMessages,
            hash: tx.hash,
            now: tx.now
          };
        });
  
        setTransactions(formattedTxs as Transaction[]);
      } catch (error) {
        console.error('Error fetching transactions:', error);
        setError('Failed to load transactions');
      } finally {
        setIsLoading(false);
      }
    }, [currentNetwork]);
    // Balance fetching with WASM integration
    const fetchBalance = useCallback(async (address: string) => {
      try {
        const endpoint = await getHttpEndpoint({
          network: currentNetwork === 'ton-mainnet' ? 'mainnet' : 'testnet'
        });
        const tonClient = new TonClient({ endpoint });
        const balanceValue = await tonClient.getBalance(Address.parse(address));
        setBalance((Number(balanceValue) / 1e9).toFixed(2));
      } catch (error) {
        console.error('Error fetching balance:', error);
        setBalance('0');
      }
    }, [currentNetwork]);
  
    // Network handling with WASM integration
    const handleNetworkChange = useCallback(async (network: string) => {
      setCurrentNetwork(network);
      if (userAddress && wasmInitialized) {
        setIsLoading(true);
        try {
          await Promise.all([
            fetchBalance(userAddress),
            fetchTokens(userAddress),
            fetchTransactions(userAddress)
          ]);
          
          // Initialize WASM for new network
          if (network.includes('overpass')) {
            await init_panic_hook();
          }
        } catch (e) {
          setError('Failed to update network data');
        } finally {
          setIsLoading(false);
        }
      }
    }, [userAddress, fetchBalance, fetchTokens, fetchTransactions, wasmInitialized]);

  function handleDisconnect(): void {
    throw new Error('Function not implemented.');
  }
}

export default App;
  