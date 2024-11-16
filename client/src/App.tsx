import type { WalletInfo } from '@tonconnect/sdk';
import { useEffect, useState, useCallback, useRef, type FC } from 'react';
import AudioPlayer from './components/AudioPlayer';
import { useAudio } from './hooks/useAudio';
import { TonConnectUI, TonConnectButton } from '@tonconnect/ui-react';
import { getHttpEndpoint } from '@orbs-network/ton-access';
import wasmInit, { initiate_transaction, init_panic_hook } from '@wasm/overpass_rs';
import { BackendApi } from './wasm/api';
import { Address, TonClient } from '@ton/ton';
import setBalance from './services/setBalance';

// Custom types for proper TypeScript support
interface TonClientInstance extends TonClient {
  getBalance(address: Address): Promise<bigint>;
  getTransactions(address: Address, opts: { limit: number }): Promise<any[]>;
}
// Type definitions
type MenuScreen = 'main' | 'send' | 'receive' | 'tokens' | 'wallets' | 'channels' | 
  'channelSettings' | 'transactions' | 'settings' | 'market';

type Token = {
  symbol: string;
  balance: string;
  price?: number;
};

type Transaction = {
  inMessage: any;
  outMessages: any[];
  hash: string;
  now: number;
  id: string;
  type: 'in' | 'out';
  amount: string;
  address: string;
  timestamp: number;
};

type InputFocus = 'amount' | 'address' | 'none';


  export const App: FC = () => {
    // State management
 
    const [wasmReady, setWasmReady] = useState<boolean>(false);   
    const [backendConnected, setBackendConnected] = useState<boolean>(false);

    // Financial state
    const [balance, setBalance] = useState<string>('0');
    const [tokens, setTokens] = useState<Token[]>([]);
    const [transactions, setTransactions] = useState<Transaction[]>([]);
    const [sendAmount, setSendAmount] = useState<string>('');
    const [sendAddress, setSendAddress] = useState<string>('');
    const [selectedIndex, setSelectedIndex] = useState<number>(0);
    const [selectedSection, setSelectedSection] = useState<string>('networks');

    // UI state
    const [currentScreen, setCurrentScreen] = useState<MenuScreen>('main');
    const [copied, setCopied] = useState<boolean>(false);
    const [currentNetwork, setCurrentNetwork] = useState<string>('ton-mainnet');
    const [isLoading, setIsLoading] = useState<boolean>(false);
    const [inputFocus, setInputFocus] = useState<InputFocus>('none');
    const [wasmInitialized, setWasmInitialized] = useState<boolean>(false);
    const [connector, setConnector] = useState<TonConnectUI | null>(null);
    const [connected, setConnected] = useState<boolean>(false);
    const [userAddress, setUserAddress] = useState<string>('');
    const [errorMessage, setErrorMessage] = useState<string>('');

    // Audio state and refs
    const { isAudioOn, volume, toggleAudio } = useAudio();
    const [playCoinSound, setPlayCoinSound] = useState<boolean>(false);
    const [playLvlSound, setPlayLvlSound] = useState<boolean>(false);
    const [playPowerupSound, setPlayPowerupSound] = useState<boolean>(false);
    const [playPipeSound, setPlayPipeSound] = useState<boolean>(false);
    const [playJumpSound, setPlayJumpSound] = useState<boolean>(false);

    const coinSoundRef = useRef<HTMLAudioElement>(null);
    const lvlSoundRef = useRef<HTMLAudioElement>(null);
    const powerupSoundRef = useRef<HTMLAudioElement>(null);
    const pipeSoundRef = useRef<HTMLAudioElement>(null);
    const jumpSoundRef = useRef<HTMLAudioElement>(null);

    // WASM instance ref
    const wasmInstance = useRef<any>(null);

    // Utility functions
    const formatAddress = useCallback((address: string): string => {
      if (address.length <= 8) return address;
      return `${address.slice(0, 4)}...${address.slice(-4)}`;
    }, []);

  // Enhanced data fetching implementations
  const fetchBalance = useCallback(async (address: string): Promise<void> => {
    try {
      const endpoint = await getHttpEndpoint({
        network: currentNetwork === 'ton-mainnet' ? 'mainnet' : 'testnet'
      });
      const tonClient = new TonClient({ endpoint });
      const balanceValue = await tonClient.getBalance(Address.parse(address));
      const formattedBalance = (Number(balanceValue) / 1e9).toFixed(2);

      // Update both local and backend state
      setBalance(formattedBalance);
      if (backendConnected) {
        await BackendApi.updateBalance({
          address,
          balance: formattedBalance
        });
      }

      // Update local balance service
      await setBalance(formattedBalance);

      triggerCoinSound();
    } catch (error) {
      console.error('Error fetching balance:', error);
      setBalance('0');
      setErrorMessage('Failed to fetch balance');
    }
  }, [currentNetwork, backendConnected, triggerCoinSound]);

  const fetchTransactions = useCallback(async (address: string): Promise<void> => {
    try {
      setIsLoading(true);

      // Fetch from blockchain
      const endpoint = await getHttpEndpoint({
        network: currentNetwork === 'ton-mainnet' ? 'mainnet' : 'testnet'
      });
      const tonClient = new TonClient({ endpoint });
      const txs = await tonClient.getTransactions(Address.parse(address), { limit: TRANSACTION_FETCH_LIMIT });

      // Process transactions
      const formattedTxs = txs.map((tx): Transaction => {
        const isIncoming = tx.inMessage?.info?.dest?.toString() === address;
        const amount = isIncoming 
          ? tx.inMessage?.info?.value?.toString() || '0'
          : tx.outMessages?.[0]?.info?.value?.toString() || '0';

        const transaction: Transaction = {
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

        return transaction;
      });

      // Update local state
      setTransactions(formattedTxs);

      // Sync with backend if connected
      if (backendConnected) {
        await BackendApi.syncTransactions({
          address,
          transactions: formattedTxs
        });
      }

    } catch (error) {
      console.error('Error fetching transactions:', error);
      setErrorMessage('Failed to load transactions');
      setTransactions([]);
    } finally {
      setIsLoading(false);
    }
  }, [currentNetwork, backendConnected]);

  // Wallet connection effect
  useEffect(() => {
    if (connector) {
      const unsubscribeHandler = connector.onStatusChange(
        async (wallet: WalletInfo | null) => {
          if (wallet) {
            setConnected(true);
            const address = wallet.account.address.toString();
            setUserAddress(address);
            triggerPowerupSound();
            
            try {
              await Promise.all([
                fetchBalance(address),
                fetchTransactions(address)
              ]);

              // Sync with backend
              if (backendConnected) {
                await BackendApi.syncWalletState({
                  address,
                  connected: true
                });
              }
            } catch (error) {
              console.error('Failed to sync wallet state:', error);
              setErrorMessage('Failed to sync wallet data');
            }
          } else {
            setConnected(false);
            setUserAddress('');
            triggerPipeSound();
            setCurrentScreen('main');
            setSelectedIndex(0);
            setSelectedSection('networks');

            // Update backend state
            if (backendConnected) {
              try {
                await BackendApi.syncWalletState({
                  connected: false
                });
              } catch (error) {
                console.error('Failed to sync disconnection:', error);
              }
            }
          }
        }
      );

      return () => {
        if (unsubscribeHandler) {
          unsubscribeHandler();
        }
      };
    }
      }, [
        connector,
        backendConnected,
        wasmReady,
        userAddress,
        fetchBalance,
        fetchTransactions,
      ]);

      const transaction = {
        validUntil: Math.floor(Date.now() / 1000) + 600, // 10 minutes
        messages: [{
          address: sendAddress,
          amount: amount.toString(),
          payload: transactionPayload
        }],
      };

      // Send transaction through connector
      await connector.sendTransaction(transaction);
      
      // Update backend state
      if (backendConnected) {
        await BackendApi.recordTransaction({
          from: userAddress,
          to: sendAddress,
          amount: sendAmount
        });
      }
      
      setSendAmount('');
      setSendAddress('');
      setCurrentScreen('main');
      triggerCoinSound();

      await Promise.all([
        fetchBalance(userAddress),
        fetchTransactions(userAddress)
      ]);
    } catch (error) {
      console.error('Transaction error:', error);
      setErrorMessage(error instanceof Error ? error.message : 'Transaction failed');
      triggerPipeSound();
    } finally {
      setIsLoading(false);
    }
  }, [
    connector,
    userAddress,
    sendAmount,
    sendAddress,
    wasmReady,
    backendConnected,
    validateAmount,
    validateAddress,
    triggerCoinSound,
    triggerPipeSound,
    fetchBalance,
    fetchTransactions
  ]);

  // Enhanced handlers with WASM and Backend integration    
const handleSend = useCallback(async (): Promise<void> => {
    try {
      if (!connector) {
        throw new Error('Wallet connector not initialized');
      }

      if (!wasmReady) {
        throw new Error('WASM components not initialized');
      }

      if (!validateAmount(sendAmount)) {
        throw new Error('Invalid amount');
      }

      if (!validateAddress(sendAddress)) {
        throw new Error('Invalid address');
      }

      setIsLoading(true);
      
      const amount = parseFloat(sendAmount) * 1e9; // Convert to nanotons

      // Use WASM to initiate transaction
      const transactionPayload = await initiate_transaction({
        to: sendAddress,
        amount: amount.toString(),
        wallet: userAddress
      }); 

  const handleDisconnect = useCallback(async (): Promise<void> => {
    try {
      if (!connector) {
        throw new Error('Wallet connector not initialized');
      }

        setIsLoading(true);
      
        // Deregister from backend
        if (backendConnected) {
          await BackendApi.deregisterWallet();
        }
        
        await connector.disconnect(); 
      setCurrentScreen('main');
      triggerPipeSound();
    } catch (error) {
      console.error('Disconnection error:', error);
      setErrorMessage('Failed to disconnect wallet');
    } finally {
      setIsLoading(false);
    }
  }, [connector, backendConnected, wasmReady, userAddress, setCurrentScreen, triggerPipeSound]);

  const handleDisconnect = useCallback(async (): Promise<void> => { 
    try {
      if (!connector) {
        throw new Error('Wallet connector not initialized');
      }

      setIsLoading(true);
      
      // Deregister from backend
      if (backendConnected) {
        await BackendApi.deregisterWallet();
      }
      
      await connector.disconnect();
      setCurrentScreen('main');
      triggerPipeSound();
    } catch (error) {
      console.error('Disconnection error:', error);
      setErrorMessage('Failed to disconnect wallet');
    } finally {
      setIsLoading(false);
    }
  }, [connector, backendConnected, wasmReady, userAddress, setCurrentScreen, triggerPipeSound]);  

  const handleSend = useCallback(async (): Promise<void> => {         
    try {
      if (!connector || !userAddress) {
        throw new Error('Wallet not connected');
      }

      if (!wasmReady) {
        throw new Error('WASM components not initialized');
      }

      if (!validateAmount(sendAmount)) {
        throw new Error('Invalid amount');
      }

      if (!validateAddress(sendAddress)) {
        throw new Error('Invalid address');
      }

      setIsLoading(true);
      
      const amount = parseFloat(sendAmount) * 1e9; // Convert to nanotons
      
      // Use WASM to initiate transaction
      const transactionPayload = await initiate_transaction({
        to: sendAddress,
        amount: amount.toString(),
        wallet: userAddress
      }); 

  // Enhanced data fetching implementations
  const fetchBalance = useCallback(async (address: string): Promise<void> => {
    try {
      const endpoint = await getHttpEndpoint({
        network: currentNetwork === 'ton-mainnet' ? 'mainnet' : 'testnet'
      });
      
      const tonClient = new TonClient({ endpoint }) as TonClientInstance;
      const balanceValue = await tonClient.getBalance(Address.parse(address));
      const formattedBalance = (Number(balanceValue) / 1e9).toFixed(2);
      
      // Update both local and backend state
      setBalance(formattedBalance);
      if (backendConnected) {
        await BackendApi.updateBalance({
          address,
          balance: formattedBalance
        });
      }
      
      // Update local balance service
      await setBalance(formattedBalance);
      
      triggerCoinSound();
    } catch (error) {
      console.error('Error fetching balance:', error);
      setBalance('0');
      setErrorMessage('Failed to fetch balance');
    }
  }, [currentNetwork, backendConnected, triggerCoinSound]);

  const fetchTransactions = useCallback(async (address: string): Promise<void> => {
    try {
      setIsLoading(true);
      
      // Fetch from blockchain
      const endpoint = await getHttpEndpoint({
        network: currentNetwork === 'ton-mainnet' ? 'mainnet' : 'testnet'
      });
      const tonClient = new TonClient({ endpoint }) as TonClientInstance;
      const txs = await tonClient.getTransactions(Address.parse(address), { limit: TRANSACTION_FETCH_LIMIT });

      // Process transactions
      const formattedTxs = txs.map((tx): Transaction => {
        const isIncoming = tx.inMessage?.info?.dest?.toString() === address;
        const amount = isIncoming 
          ? tx.inMessage?.info?.value?.toString() || '0'
          : tx.outMessages?.[0]?.info?.value?.toString() || '0';

        const transaction: Transaction = {
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

        return transaction;
      });

      // Update local state
      setTransactions(formattedTxs);

      // Sync with backend if connected
      if (backendConnected) {
        await BackendApi.syncTransactions({
          address,
          transactions: formattedTxs
        });
      }

    } catch (error) {
      console.error('Error fetching transactions:', error);
      setErrorMessage('Failed to load transactions');
      setTransactions([]);
    } finally {
      setIsLoading(false);
    }
  }, [currentNetwork, backendConnected]);

  // Sync effect for wallet status changes
  useEffect(() => {
    if (connector) {
      const unsubscribeHandler = connector.onStatusChange(
        async (wallet: WalletInfo | null) => {
          if (wallet) {
            setConnected(true);
            const address = wallet.account.address.toString();
            setUserAddress(address);
            triggerPowerupSound();
            
            try {
              await Promise.all([
                fetchBalance(address),
                fetchTransactions(address)
              ]);

              // Sync with backend
              if (backendConnected) {
                await BackendApi.syncWalletState({
                  address,
                  connected: true
                });
              }
            } catch (error) {
              console.error('Failed to sync wallet state:', error);
              setErrorMessage('Failed to sync wallet data');
            }
          } else {
            setConnected(false);
            setUserAddress('');
            triggerPipeSound();
            setCurrentScreen('main');
            setSelectedIndex(0);
            setSelectedSection('networks');

            // Update backend state
            if (backendConnected) {
              try {
                await BackendApi.syncWalletState({
                  connected: false
                });
              } catch (error) {
                console.error('Failed to sync disconnection:', error);
              }
            }
          }
        }
      );

      return () => {
        if (unsubscribeHandler) {
          unsubscribeHandler();
        }
      };
    }
  }, [
    connector,
    backendConnected,
    triggerPowerupSound,
    triggerPipeSound,
    fetchBalance,
    fetchTransactions
  ]);
      
      const transaction = {
        validUntil: Math.floor(Date.now() / 1000) + 600, // 10 minutes
        messages: [{
          address: sendAddress,
          amount: amount.toString(),
          payload: transactionPayload
        }],
      };

      // Send transaction through connector
      await connector.sendTransaction(transaction);
      
      // Update backend state
      if (backendConnected) {
        await BackendApi.recordTransaction({
          from: userAddress,
          to: sendAddress,
          amount: sendAmount
        });
      }
      
      setSendAmount('');
      setSendAddress('');
      setCurrentScreen('main');
      triggerCoinSound();

      await Promise.all([
        fetchBalance(userAddress),
        fetchTransactions(userAddress)
      ]);
    } catch (error) {
      console.error('Transaction error:', error);
      setErrorMessage(error instanceof Error ? error.message : 'Transaction failed');
      triggerPipeSound();
    } finally {
      setIsLoading(false);
    }
  }, [
    connector,
    userAddress,
    sendAmount,
    sendAddress,
    wasmReady,
    backendConnected,
    validateAmount,
    validateAddress,
    triggerCoinSound,
    triggerPipeSound
  ]);

  const validateAmount = useCallback((amount: string): boolean => {
    const numAmount = parseFloat(amount);
    return !isNaN(numAmount) && numAmount > 0 && numAmount <= parseFloat(balance);
  }, [balance]);

  const validateAddress = useCallback((address: string): boolean => {
    try {
      Address.parse(address);
      return true;
    } catch {
      return false;
    }
  }, []);

  // Sound triggers
  const triggerCoinSound = useCallback((): void => {
    if (isAudioOn) {
      setPlayCoinSound(true);
    }
  }, [isAudioOn]);

  const triggerPowerupSound = useCallback((): void => {
    if (isAudioOn) {
      setPlayPowerupSound(true);
    }
  }, [isAudioOn]);

  const triggerPipeSound = useCallback((): void => {
    if (isAudioOn) {
      setPlayPipeSound(true);
    }
  }, [isAudioOn]);

  // Initialize WASM
  useEffect(() => {
    const initWasm = async (): Promise<void> => {
      try {
        const wasm = await wasmInit();
        wasmInstance.current = wasm;
        setWasmInitialized(true);
      } catch (error) {
        console.error("Failed to initialize WASM:", error);
      }
    };
    initWasm();
  }, []);

  // Initialize TonConnect
  useEffect(() => {
    const initTonConnect = async (): Promise<void> => {
      try {
        const tonConnect = new TonConnectUI({
          manifestUrl: 'https://overpass.network/tonconnect-manifest.json',
          buttonRootId: 'connect-wallet',
        });
        
        await tonConnect.connectionRestored;
        setConnector(tonConnect);
        
        const walletsList = tonConnect.getWallets();
        console.log(walletsList);
        
        const isConnected = tonConnect.connected;
        setConnected(isConnected);
        
        if (isConnected && tonConnect.account?.address) {
          setUserAddress(tonConnect.account.address.toString());
        }
      } catch (error) {
        console.error('Failed to initialize TonConnect:', error);
        setErrorMessage('Failed to connect to wallet services');
      }
    };    
    initTonConnect();
  }, []);

  // Core handlers
  const handleConnect = useCallback(async (): Promise<void> => {
    try {
      if (!connector) {
        throw new Error('Wallet connector not initialized');
      }
      
      setIsLoading(true);
      connector.connectWallet();
      triggerPowerupSound();
    } catch (error) {
      console.error('Connection error:', error);
      setErrorMessage('Failed to connect wallet');
      triggerPipeSound();
    } finally {
      setIsLoading(false);
    }
  }, [connector, triggerPowerupSound, triggerPipeSound]);

  const handleDisconnect = useCallback(async (): Promise<void> => {
    try {
      if (!connector) {
        throw new Error('Wallet connector not initialized');
      }

      setIsLoading(true);
      await connector.disconnect();
      setCurrentScreen('main');
      triggerPipeSound();
    } catch (error) {
      console.error('Disconnection error:', error);
      setErrorMessage('Failed to disconnect wallet');
    } finally {
      setIsLoading(false);
    }
  }, [connector, triggerPipeSound]);

  const handleSend = useCallback(async (): Promise<void> => {
    try {
      if (!connector || !userAddress) {
        throw new Error('Wallet not connected');
      }

      if (!validateAmount(sendAmount)) {
        throw new Error('Invalid amount');
      }

      if (!validateAddress(sendAddress)) {
        throw new Error('Invalid address');
      }

      setIsLoading(true);
      
      const amount = parseFloat(sendAmount) * 1e9; // Convert to nanotons
      
      const transaction = {
        validUntil: Math.floor(Date.now() / 1000) + 600, // 10 minutes
        messages: [
          {
            address: sendAddress,
            amount: amount.toString(),
          },
        ],
      };

      await connector.sendTransaction(transaction);
      
      setSendAmount('');
      setSendAddress('');
      setCurrentScreen('main');
      triggerCoinSound();

      const address = userAddress;
      await Promise.all([
        fetchBalance(address),
        fetchTransactions(address)
      ]);
    } catch (error) {
      console.error('Transaction error:', error);
      setErrorMessage(error instanceof Error ? error.message : 'Transaction failed');
      triggerPipeSound();
    } finally {
      setIsLoading(false);
    }
  }, [
    connector,
    userAddress,
    sendAmount,
    sendAddress,
    validateAmount,
    validateAddress,
    triggerCoinSound,
    triggerPipeSound,
    fetchBalance,
    fetchTransactions
  ]);

  // Data fetching implementations
  const fetchBalance = useCallback(async (address: string): Promise<void> => {
    try {
      const endpoint = await getHttpEndpoint({
        network: currentNetwork === 'ton-mainnet' ? 'mainnet' : 'testnet'
      });
      const tonClient = new TonClient({ endpoint });
      const balanceValue = await tonClient.getBalance(Address.parse(address));
      setBalance((Number(balanceValue) / 1e9).toFixed(2));
      triggerCoinSound();
    } catch (error) {
      console.error('Error fetching balance:', error);
      setBalance('0');
    }
  }, [currentNetwork, triggerCoinSound]);

  const fetchTransactions = useCallback(async (address: string): Promise<void> => {
    try {
      setIsLoading(true);
      const endpoint = await getHttpEndpoint({
        network: currentNetwork === 'ton-mainnet' ? 'mainnet' : 'testnet'
      });
      const tonClient = new TonClient({ endpoint });
      const txs = await tonClient.getTransactions(Address.parse(address), { limit: TRANSACTION_FETCH_LIMIT });

      const formattedTxs = txs.map((tx): Transaction => {
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

      setTransactions(formattedTxs);
    } catch (error) {
      console.error('Error fetching transactions:', error);
      setErrorMessage('Failed to load transactions');
    } finally {
      setIsLoading(false);
    }
  }, [currentNetwork]);

  // Wallet connection effect
  useEffect(() => {
    if (connector) {
      const unsubscribeHandler = connector.onStatusChange(
        async (wallet: WalletInfo | null) => {
          if (wallet) {
            setConnected(true);
            setUserAddress(wallet.account.address.toString());
            triggerPowerupSound();
            
            try {
              const fetchWalletData = async (): Promise<void> => {
                const balance = await connector.getBalance();
                if (balance !== undefined) {
                  setBalance(balance);
                }
                
                const tokens = await connector.getTokens();
                if (tokens !== undefined) {
                  setTokens(tokens);
                }
                
                const transactions = await connector.getTransactions();
                if (transactions !== undefined) {
                  setTransactions(transactions);
                }
              };
              await fetchWalletData();
            } catch (error) {
              console.error('Failed to fetch wallet data:', error);
              setErrorMessage('Failed to fetch wallet data');
            }
          } else {
            setConnected(false);
            setUserAddress('');
            triggerPipeSound();
            setCurrentScreen('main');
            setSelectedIndex(0);
            setSelectedSection('networks');
          }
        }
      );

      return () => {
        if (unsubscribeHandler) {
          unsubscribeHandler();
        }
      };
    }
  }, [connector, triggerPowerupSound, triggerPipeSound  ]);

  return (
    <div className="min-h-screen bg-gradient-to-b from-gray-900 to-gray-800 text-white">
      <div className="container mx-auto px-4 py-8">
        <header className="flex items-center justify-between mb-8">
          <div className="flex items-center space-x-4">
            <h1 className="text-2xl font-bold bg-clip-text text-transparent bg-gradient-to-r from-blue-400 to-purple-600">
              Overpass Wallet
            </h1>
            <div className="flex items-center space-x-2">
              <button
                onClick={toggleAudio}
                className={`p-2 rounded-full ${
                  isAudioOn ? 'bg-green-500' : 'bg-gray-600'
                } transition-colors duration-200`}
                type="button"
                aria-label={isAudioOn ? 'Mute sound effects' : 'Enable sound effects'}
              >
                <span className="sr-only">
                  {isAudioOn ? 'Sound On' : 'Sound Off'}
                </span>
              </button>
              {!wasmReady && (
                <div className="text-yellow-500 text-sm">
                  Initializing...
                </div>
              )}
            </div>
          </div>
          <div className="flex items-center space-x-4">
            {backendConnected && (
              <span className="text-green-500 text-sm">●</span>
            )}
            <div id="connect-wallet">
              <TonConnectButton />
            </div>
            {connected && (
              <button
                onClick={handleDisconnect}
                className="bg-red-500 hover:bg-red-600 text-white px-4 py-2 rounded-lg transition-colors duration-200"
                type="button"
                disabled={isLoading}
              >
                Disconnect
              </button>
            )}
          </div>
        </header>

        <main>
          {!wasmReady && (
            <div className="bg-yellow-500/20 text-yellow-500 px-4 py-3 rounded-lg mb-4">
              Initializing wallet components...
            </div>
          )}

          {connected && currentScreen === 'main' && (
            <div>
              <div className="bg-gray-800 rounded-xl p-6 shadow-lg mb-6">
                <h2 className="text-xl font-semibold mb-4">Balance</h2>
                <div className="text-3xl font-bold">{balance} TON</div>
                {backendConnected && (
                  <div className="text-sm text-gray-400 mt-2">
                    Last synced: {new Date().toLocaleTimeString()}
                  </div>
                )}
              </div>

              <div className="bg-gray-800 rounded-xl p-6 shadow-lg">
                <h2 className="text-xl font-semibold mb-4">Tokens</h2>
                <div className="space-y-4">
                  {tokens.length > 0 ? (
                    tokens.map((token) => (
                      <div key={token.symbol} className="flex items-center justify-between">
                        <div className="font-medium">{token.symbol}</div>
                        <div>{token.balance}</div>
                      </div>
                    ))
                  ) : (
                    <div className="text-gray-400 text-center py-4">
                      No tokens found
                    </div>
                  )}
                </div>
              </div>
            </div>
          )}

          {currentScreen === 'send' && connected && (
            <div className="bg-gray-800 rounded-xl p-6 shadow-lg">
              <h2 className="text-xl font-semibold mb-6">Send TON</h2>
              <form 
                onSubmit={(e: React.FormEvent<HTMLFormElement>) => { 
                  e.preventDefault(); 
                  handleSend(); 
                }} 
                className="space-y-6"
              >
                <div className="space-y-4">
                  <div>
                    <label htmlFor="amount" className="block text-sm font-medium mb-2">
                      Amount
                    </label>
                    <input
                      id="amount"
                      type="text"
                      value={sendAmount}
                      onChange={(e: React.ChangeEvent<HTMLInputElement>) => setSendAmount(e.target.value)}
                      onFocus={() => setInputFocus('amount')}
                      onBlur={() => setInputFocus('none')}
                      className={`w-full bg-gray-700 rounded-lg p-3 focus:ring-2 ${
                        inputFocus === 'amount' ? 'ring-blue-500' : 'ring-transparent'
                      } transition-all duration-200`}
                      placeholder="0.0"
                      disabled={!wasmReady || isLoading}
                    />
                    <p className="mt-1 text-sm text-gray-400">
                      Available: {balance} TON
                    </p>
                  </div>

                  <div>
                    <label htmlFor="address" className="block text-sm font-medium mb-2">
                      Recipient Address
                    </label>
                    <input
                      id="address"
                      type="text"
                      value={sendAddress}
                      onChange={(e: React.ChangeEvent<HTMLInputElement>) => setSendAddress(e.target.value)}
                      onFocus={() => setInputFocus('address')}
                      onBlur={() => setInputFocus('none')}
                      className={`w-full bg-gray-700 rounded-lg p-3 focus:ring-2 ${
                        inputFocus === 'address' ? 'ring-blue-500' : 'ring-transparent'
                      } transition-all duration-200`}
                      placeholder="Enter TON address"
                      disabled={!wasmReady || isLoading}
                    />
                  </div>
                </div>

                <button
                  type="submit"
                  disabled={!wasmReady || !validateAmount(sendAmount) || !validateAddress(sendAddress) || isLoading}
                  className={`w-full py-3 rounded-lg font-medium transition-colors duration-200 ${
                    !wasmReady || !validateAmount(sendAmount) || !validateAddress(sendAddress) || isLoading
                      ? 'bg-gray-600 cursor-not-allowed'
                      : 'bg-blue-500 hover:bg-blue-600'
                  }`}
                >
                  {isLoading ? 'Sending...' : 'Send TON'}
                </button>
              </form>
            </div>
          )}

          {currentScreen === 'transactions' && connected && (
            <div className="bg-gray-800 rounded-xl p-6 shadow-lg">
              <h2 className="text-xl font-semibold mb-6">Transactions</h2>
              <div className="space-y-4">
                {transactions.map((tx) => (
                  <div
                    key={tx.id}
                    className="flex items-center justify-between p-4 bg-gray-700 rounded-lg"
                  >
                    <div className="flex items-center space-x-4">
                      <div
                        className={`w-10 h-10 rounded-full flex items-center justify-center ${
                          tx.type === 'in' ? 'bg-green-500/20' : 'bg-red-500/20'
                        }`}
                      >
                        <span className={tx.type === 'in' ? 'text-green-500' : 'text-red-500'}>
                          {tx.type === 'in' ? '↓' : '↑'}
                        </span>
                      </div>
                      <div>
                        <p className="font-medium">{formatAddress(tx.address)}</p>
                        <p className="text-sm text-gray-400">
                          {new Date(tx.timestamp).toLocaleString()}
                        </p>
                      </div>
                    </div>
                    <div className="text-right">
                      <p className={`font-medium ${
                        tx.type === 'in' ? 'text-green-500' : 'text-red-500'
                      }`}>
                        {tx.type === 'in' ? '+' : '-'}{tx.amount} TON
                      </p>
                    </div>
                  </div>
                ))}
                {transactions.length === 0 && (
                  <div className="text-center py-8 text-gray-400">
                    No transactions found
                  </div>
                )}
              </div>
            </div>
          )}

          {/* Error Display */}
          {errorMessage && (
            <div 
              className="fixed bottom-4 right-4 bg-red-500 text-white px-6 py-3 rounded-lg shadow-lg"
              role="alert"
            >
              {errorMessage}
            </div>
          )}
        </main>
      </div>

      {/* Audio Players */}
      <AudioPlayer
        ref={coinSoundRef}
        src="/sounds/coin.mp3"
        play={playCoinSound}
        volume={volume}
        onEnded={() => setPlayCoinSound(false)}
      />
      <AudioPlayer
        ref={lvlSoundRef}
        src="/sounds/level-up.mp3"
        play={playLvlSound}
        volume={volume}
        onEnded={() => setPlayLvlSound(false)}
      />
      <AudioPlayer
        ref={powerupSoundRef}
        src="/sounds/powerup.mp3"
        play={playPowerupSound}
        volume={volume}
        onEnded={() => setPlayPowerupSound(false)}
      />
      <AudioPlayer
        ref={pipeSoundRef}
        src="/sounds/pipe.mp3"
        play={playPipeSound}
        volume={volume}
        onEnded={() => setPlayPipeSound(false)}
      />
      <AudioPlayer
        ref={jumpSoundRef}
        src="/sounds/jump.mp3"
        play={playJumpSound}
        volume={volume}
        onEnded={() => setPlayJumpSound(false)}
      />

    <div className="min-h-screen bg-gradient-to-b from-gray-900 to-gray-800 text-white">
      <div className="container mx-auto px-4 py-8">
        <header className="flex items-center justify-between mb-8">
          <div className="flex items-center space-x-4">
            <h1 className="text-2xl font-bold bg-clip-text text-transparent bg-gradient-to-r from-blue-400 to-purple-600">
              Overpass Wallet
            </h1>
            <div className="flex items-center space-x-2">
              <button
                onClick={toggleAudio}
                className={`p-2 rounded-full ${
                  isAudioOn ? 'bg-green-500' : 'bg-gray-600'
                } transition-colors duration-200`}
                type="button"
              >
                <span className="sr-only">
                  {isAudioOn ? 'Sound On' : 'Sound Off'}
                </span>
              </button>
            </div>
          </div>
          <div>
            {connected ? (
              <button
                onClick={handleDisconnect}
                className="bg-red-500 hover:bg-red-600 text-white px-4 py-2 rounded-lg transition-colors duration-200"
                type="button"
              >
                Disconnect
              </button>
            ) : (
              <button
                onClick={handleConnect}
                className="bg-blue-500 hover:bg-blue-600 text-white px-4 py-2 rounded-lg transition-colors duration-200"
                type="button"
              >
                Connect Wallet
              </button>
            )}

          {/* Error Display */}
          {errorMessage && (
            <div className="fixed bottom-4 right-4 bg-red-500 text-white px-6 py-3 rounded-lg shadow-lg">
              {errorMessage}
            </div>
          )}
        </main>
      </div>

      {/* Audio Players */}
      <AudioPlayer
        ref={coinSoundRef}
        src="/sounds/coin.mp3"
        play={playCoinSound}
        volume={volume}
        onEnded={() => setPlayCoinSound(false)}
      />
      <AudioPlayer
        ref={lvlSoundRef}
        src="/sounds/level-up.mp3"
        play={playLvlSound}
        volume={volume}
        onEnded={() => setPlayLvlSound(false)}
      />
      <AudioPlayer
        ref={powerupSoundRef}
        src="/sounds/powerup.mp3"
        play={playPowerupSound}
        volume={volume}
        onEnded={() => setPlayPowerupSound(false)}
      />
      <AudioPlayer
        ref={pipeSoundRef}
        src="/sounds/pipe.mp3"
        play={playPipeSound}
        volume={volume}
        onEnded={() => setPlayPipeSound(false)}
      />
      <AudioPlayer
        ref={jumpSoundRef}
        src="/sounds/jump.mp3"
        play={playJumpSound}
        volume={volume}
        onEnded={() => setPlayJumpSound(false)}
      />
    </div>
  );
};

export default App;

          {/* Send Transaction Form */}
          {currentScreen === 'send' && connected && (
            <div className="bg-gray-800 rounded-xl p-6 shadow-lg">
              <h2 className="text-xl font-semibold mb-6">Send TON</h2>
              <form 
                onSubmit={(e: React.FormEvent<HTMLFormElement>) => { 
                  e.preventDefault(); 
                  handleSend(); 
                }} 
                className="space-y-6"
              >
                <div className="space-y-4">
                  <div>
                    <label className="block text-sm font-medium mb-2" htmlFor="amount">Amount</label>
                    <input
                      id="amount"
                      type="text"
                      value={sendAmount}
                      onChange={(e: React.ChangeEvent<HTMLInputElement>) => setSendAmount(e.target.value)}
                      onFocus={() => setInputFocus('amount')}
                      onBlur={() => setInputFocus('none')}
                      className={`w-full bg-gray-700 rounded-lg p-3 focus:ring-2 ${
                        inputFocus === 'amount' ? 'ring-blue-500' : 'ring-transparent'
                      } transition-all duration-200`}
                      placeholder="0.0"
                    />
                    <p className="mt-1 text-sm text-gray-400">
                      Available: {balance} TON
                    </p>
                  </div>

                  <div>
                    <label className="block text-sm font-medium mb-2" htmlFor="address">Recipient Address</label>
                    <input
                      id="address"
                      type="text"
                      value={sendAddress}
                      onChange={(e: React.ChangeEvent<HTMLInputElement>) => setSendAddress(e.target.value)}
                      onFocus={() => setInputFocus('address')}
                      onBlur={() => setInputFocus('none')}
                      className={`w-full bg-gray-700 rounded-lg p-3 focus:ring-2 ${
                        inputFocus === 'address' ? 'ring-blue-500' : 'ring-transparent'
                      } transition-all duration-200`}
                      placeholder="Enter TON address"
                    />
                  </div>
                </div>

                <button
                  type="submit"
                  disabled={!validateAmount(sendAmount) || !validateAddress(sendAddress) || isLoading}
                  className={`w-full py-3 rounded-lg font-medium transition-colors duration-200 ${
                    !validateAmount(sendAmount) || !validateAddress(sendAddress) || isLoading
                      ? 'bg-gray-600 cursor-not-allowed'
                      : 'bg-blue-500 hover:bg-blue-600'
                  }`}
                >
                  {isLoading ? 'Sending...' : 'Send TON'}
                </button>
              </form>
            </div>
          )}

          {/* Transaction History */}
          {currentScreen === 'transactions' && connected && (
            <div className="bg-gray-800 rounded-xl p-6 shadow-lg">
              <h2 className="text-xl font-semibold mb-6">Transactions</h2>
              <div className="space-y-4">
                {transactions.map((tx) => (
                  <div
                    key={tx.id}
                    className="flex items-center justify-between p-4 bg-gray-700 rounded-lg"
                  >
                    <div className="flex items-center space-x-4">
                      <div
                        className={`w-10 h-10 rounded-full flex items-center justify-center ${
                          tx.type === 'in' ? 'bg-green-500/20' : 'bg-red-500/20'
                        }`}
                      >
                        <span className={tx.type === 'in' ? 'text-green-500' : 'text-red-500'}>
                          {tx.type === 'in' ? '↓' : '↑'}
                        </span>
                      </div>
                      <div>
                        <p className="font-medium">{formatAddress(tx.address)}</p>
                        <p className="text-sm text-gray-400">
                          {new Date(tx.timestamp).toLocaleString()}
                        </p>
                      </div>
                    </div>
                    <div className="text-right">
                      <p className={`font-medium ${
                        tx.type === 'in' ? 'text-green-500' : 'text-red-500'
                      }`}>
                        {tx.type === 'in' ? '+' : '-'}{tx.amount} TON
                      </p>
                    </div>
                  </div>
                ))}
                {transactions.length === 0 && (
                  <div className="text-center py-8 text-gray-400">
                    No transactions found
                  </div>
                )}
              </div>
            </div>
          )}
          </div>
        </header>

        <main>
          {/* Display balance and tokens */}
          {connected && currentScreen === 'main' && (
            <div>
              <div className="bg-gray-800 rounded-xl p-6 shadow-lg mb-6">
                <h2 className="text-xl font-semibold mb-4">Balance</h2>
                <div className="text-3xl font-bold">{balance} TON</div>
              </div>

              <div className="bg-gray-800 rounded-xl p-6 shadow-lg">
                <h2 className="text-xl font-semibold mb-4">Tokens</h2>
                <div className="space-y-4">
                  {tokens.map((token) => (
                    <div key={token.symbol} className="flex items-center justify-between">
                      <div className="font-medium">{token.symbol}</div>
                      <div>{token.balance}</div>
                    </div>
                  ))}
                </div>
              </div>
            </div>
          )}