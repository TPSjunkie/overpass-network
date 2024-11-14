import type { WalletInfo } from '@tonconnect/sdk';
import { useEffect, useState, useCallback, useRef } from 'react';
import AudioPlayer from './components/AudioPlayer';
import { useAudio } from './hooks/useAudio';
import { TonConnectUI } from '@tonconnect/ui-react';
import { getHttpEndpoint } from '@orbs-network/ton-access';
import wasmInit, { initiate_transaction, init_panic_hook } from '@wasm/overpass_rs';
import { BackendApi } from './wasm/api';
import { Address, TonClient } from '@ton/ton';
import { connector } from './connector/connection-ton';

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
  outMessages: any;
  hash: any;
  now: number;
  id: string;
  type: 'in' | 'out';
  amount: string;
  address: string;
  timestamp: number;
};



type InputFocus = 'amount' | 'address' | 'none';

// Constants

const TRANSACTION_FETCH_LIMIT = 20;

// App component  
export const App = () => {
  return null  
}
  // Financial state
  const [balance, setBalance] = useState('0');
  const [tokens, setTokens] = useState<Token[]>([]);
  const [transactions, setTransactions] = useState<Transaction[]>([]);
  const [sendAmount, setSendAmount] = useState('');
  const [sendAddress, setSendAddress] = useState('');

  // UI state
  const [currentScreen, setCurrentScreen] = useState<MenuScreen>('main');
  const [copied, setCopied] = useState(false);
  const [currentNetwork, setCurrentNetwork] = useState<string>('ton-mainnet');
  const [isLoading, setIsLoading] = useState(false);
  const [inputFocus, setInputFocus] = useState<InputFocus>('none');
  const [wasmInitialized, setWasmInitialized] = useState(false);

  // Audio state and refs
  const { isAudioOn, volume, toggleAudio } = useAudio();
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

  // WASM instance ref
  const wasmInstance = useRef<any>(null);

  // Initialize WASM
  useEffect(() => {
    const initWasm = async () => {

  // Initialize TonConnect
  useEffect(() => {
    const initTonConnect = async () => {
      try {
        const tonConnect = new TonConnectUI({
          manifestUrl: 'https://<your-domain>/tonconnect-manifest.json',
          buttonRootId: 'connect-wallet',
        });
        
        await tonConnect.connectionRestored;
        setConnector(tonConnect);
        
        const walletsList = tonConnect.getWallets();
        console.log(walletsList);
        
        const isConnected = tonConnect.connected;
        setConnected(isConnected);
        
        if (isConnected) {
          const address = tonConnect.account?.address;
          if (address) {
            setUserAddress(address);
          }
        }
      } catch (error) {
        console.error('Failed to initialize TonConnect:', error);
        setError('Failed to connect to wallet services');
      }
    };    
    initTonConnect();
  }, []);
  // Fetch balance and transactions when the wallet is connected
  useEffect(() => {
    // Effect logic here
    
    // Return a cleanup function if needed
    return () => {
      // Cleanup logic here
    }
  }, [])

  if (connector) {
    if (connector) {  
      const unsubscribe = setConnector.onStatusChange(async (wallet: { account: { address: { toString: () => any; }; }; }) => {  
        if (wallet) {  
          setConnected(true);  
          setUserAddress(wallet.account.address.toString());  
          triggerPowerupSound();  
          const balance = await setConnector.getBalance();  
          if (balance !== undefined) {  
            setBalance(balance);  
          }  
          const tokens = await connector.getTokens();  
        if (tokens !== undefined) {  
          setTokens(tokens);  
        }  
          const transactions = await setConnector.getTransactions();  
          if (transactions !== undefined) {  
            setTransactions(transactions);  
          }  
        } else {  
          setConnected(false);  
          setUserAddress('');  
          triggerPipeSound();       
        setCurrentScreen('main');  
      setSelectedIndex(0);  
      setSelectedSection('networks');  
      });  
      return () => {  
      
  // Fetch balance and transactions when the wallet is connected
  useEffect(() => {    
    if (connector) {
      const unsubscribe = connector.onStatusChange(async (wallet) => {
        if (wallet) {
          setConnected(true);
          setUserAddress(wallet.account.address.toString());
          triggerPowerupSound();
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
        } else {
          setConnected(false);
          setUserAddress('');
          triggerPipeSound();
        }
        setCurrentScreen('main'); 
      setSelectedIndex(0);  
      setSelectedSection('networks');
      });
    return () => {
        unsubscribe();
      };
    }
  }, [connector]);
  // Fetch balance and transactions when the wallet is connected
  // Utility Functions
  const formatAddress = useCallback((address: string): string => {
    if (address.length <= 8) return address;
    return `${address.slice(0, 4)}...${address.slice(-4)}`;
  }, []);


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
  const triggerCoinSound = useCallback(() => {
    if (isAudioOn) {
      setPlayCoinSound(true);
    }
  }, [isAudioOn]);

  const triggerPowerupSound = useCallback(() => {
    if (isAudioOn) {
      setPlayPowerupSound(true);
    }
  }, [isAudioOn]);

  const triggerPipeSound = useCallback(() => {
    if (isAudioOn) {
      setPlayPipeSound(true);
    }
  }, [isAudioOn]);



  // Core Handlers
  const handleConnect = useCallback(async () => {
    try {
      if (!connector) {
        throw new Error('Wallet connector not initialized');
      }
      
      setIsLoading(true);
      connector.connectWallet();
      triggerPowerupSound();
    } catch (error) {
      console.error('Connection error:', error);
      setError('Failed to connect wallet');
      triggerPipeSound();
    } finally {
      setIsLoading(false);
    }
  }, [connector, triggerPowerupSound, triggerPipeSound]);

  const handleDisconnect = useCallback(async () => {
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
      setError('Failed to disconnect wallet');
    } finally {
      setIsLoading(false);
    }
  }, [connector, triggerPipeSound]);

  const handleSend = useCallback(async () => {
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
      
      // Refresh balances and transactions
      await Promise.all([
        fetchBalance(userAddress),
        fetchTransactions(userAddress)
      ]);
    } catch (error) {
      console.error('Transaction error:', error);
      setError(error instanceof Error ? error.message : 'Transaction failed');
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
    fetchBalance,
    fetchTransactions,
    triggerCoinSound,
    triggerPipeSound,
  ]);

  // Data fetching implementations
  const fetchBalance = useCallback(async (address: string) => {
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


  const fetchTransactions = useCallback(async (address: string) => {
    try {
      setIsLoading(true);
      const endpoint = await getHttpEndpoint({
        network: currentNetwork === 'ton-mainnet' ? 'mainnet' : 'testnet'
      });
      const tonClient = new TonClient({ endpoint });
      const txs = await tonClient.getTransactions(Address.parse(address), { limit: TRANSACTION_FETCH_LIMIT });

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

      setTransactions(formattedTxs);
    } catch (error) {
      console.error('Error fetching transactions:', error);
      setError('Failed to load transactions');
    } finally {
      setIsLoading(false);
    }
  }, [currentNetwork]);

  // UI Components
  return (
    <div className="min-h-screen bg-gradient-to-b from-gray-900 to-gray-800 text-white">
      <div className="container mx-auto px-4 py-8">
        {/* Header */}
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
              >
                {isAudioOn ? (
                  <span className="sr-only">Sound On</span>
                ) : (
                  <span className="sr-only">Sound Off</span>
                )}
              </button>
            </div>
          </div>
          <div>
            {connected ? (
              <button
                onClick={handleDisconnect}
                className="bg-red-500 hover:bg-red-600 text-white px-4 py-2 rounded-lg transition-colors duration-200"
              >
                Disconnect
              </button>
            ) : (
              <button
                onClick={handleConnect}
                className="bg-blue-500 hover:bg-blue-600 text-white px-4 py-2 rounded-lg transition-colors duration-200"
              >
                Connect Wallet
              </button>
            )}
          </div>
        </header>

        {/* Main Content */}
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

          {/* Send Transaction Form */}
          {currentScreen === 'send' && connected && (
            <div className="bg-gray-800 rounded-xl p-6 shadow-lg">
              <h2 className="text-xl font-semibold mb-6">Send TON</h2>
              <form onSubmit={(e) => { e.preventDefault(); handleSend(); }} className="space-y-6">
                <div className="space-y-4">
                  <div>
                    <label className="block text-sm font-medium mb-2">Amount</label>
                    <input
                      type="text"
                      value={sendAmount}
                      onChange={(e) => setSendAmount(e.target.value)}
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
                    <label className="block text-sm font-medium mb-2">Recipient Address</label>
                    <input
                      type="text"
                      value={sendAddress}
                      onChange={(e) => setSendAddress(e.target.value)}
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

          {/* Error Display */}
          {error && (
            <div className="fixed bottom-4 right-4 bg-red-500 text-white px-6 py-3 rounded-lg shadow-lg">
              {error}
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
      function triggerPowerupSound() {
        throw new Error('Function not implemented.');
      }

      function triggerPipeSound() {
        throw new Error('Function not implemented.');
      }

      function setError(arg0: string) {
        throw new Error('Function not implemented.');
      }

      function setError(arg0: string) {
        throw new Error('Function not implemented.');
      }

      function setError(arg0: string) {
        throw new Error('Function not implemented.');
      }

      function setError(arg0: string) {
        throw new Error('Function not implemented.');
      }

      function setError(arg0: string) {
        throw new Error('Function not implemented.');
      }

      function setConnector(tonConnect: TonConnectUI) {
        throw new Error('Function not implemented.');
      }

      function setConnected(arg0: boolean) {
        throw new Error('Function not implemented.');
      }

      function setUserAddress(arg0: any) {
        throw new Error('Function not implemented.');
      }

      function setSelectedIndex(arg0: number) {
        throw new Error('Function not implemented.');
      }

      function setSelectedSection(arg0: string) {
        throw new Error('Function not implemented.');
      }

      function setError(arg0: string) {
        throw new Error('Function not implemented.');
      }

      function setError(arg0: string) {
        throw new Error('Function not implemented.');
      }

      function setError(arg0: string) {
        throw new Error('Function not implemented.');
      }

      function setError(arg0: string) {
        throw new Error('Function not implemented.');
      }

      function setConnector(tonConnect: TonConnectUI) {
        throw new Error('Function not implemented.');
      }

      function setError(arg0: string) {
        throw new Error('Function not implemented.');
      }

      function setConnector(tonConnect: TonConnectUI) {
        throw new Error('Function not implemented.');
      }

      function setError(arg0: string) {
        throw new Error('Function not implemented.');
      }

      function setError(arg0: string) {
        throw new Error('Function not implemented.');
      }

      function setError(arg0: string) {
        throw new Error('Function not implemented.');
      }

      function setError(arg0: string) {
        throw new Error('Function not implemented.');
      }

      function setError(arg0: string) {
        throw new Error('Function not implemented.');
      }

      function setError(arg0: string) {
        throw new Error('Function not implemented.');
      }

      function setError(arg0: string) {
        throw new Error('Function not implemented.');
      }

      function setError(arg0: string) {
        throw new Error('Function not implemented.');
      }

      function setError(arg0: string) {
        throw new Error('Function not implemented.');
      }

      function setError(arg0: string) {
        throw new Error('Function not implemented.');
      }

      function setError(arg0: string) {
        throw new Error('Function not implemented.');
      }

      function setError(arg0: string) {
        throw new Error('Function not implemented.');
      }

      function setError(arg0: string) {
        throw new Error('Function not implemented.');
      }

      function setError(arg0: string) {
        throw new Error('Function not implemented.');
      }

      function setError(arg0: string) {
        throw new Error('Function not implemented.');
      }

      function setError(arg0: string) {
        throw new Error('Function not implemented.');
      }

