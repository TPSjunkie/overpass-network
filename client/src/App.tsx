import type { WalletInfo } from '@tonconnect/sdk';
import { useEffect, useState, useCallback, useRef } from 'react';
import AudioPlayer from './components/AudioPlayer';
import { useAudio } from './hooks/useAudio';
import { TonConnectUI } from '@tonconnect/ui-react';
import { getHttpEndpoint } from '@orbs-network/ton-access';
import { Address, TonClient } from '@ton/ton';
import { Cell } from '@ton/core';

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

const App = () => {
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

  // Menu items
  const menuItems: MenuItem[] = connected ? [
    { label: "WALLET", screen: "wallets" },
    { label: "SEND", screen: 'send' },
    { label: 'RECEIVE', screen: 'receive' },
    { label: 'TOKENS', screen: 'tokens' },
    { label: 'MARKET', screen: 'market' },    
    { label: 'CHANNELS', screen: 'channels' }, 
    { label: "TRANSACTIONS", screen: "transactions" },
    { label: 'DISCONNECT', action: () => handleDisconnect() }
  ] : [];

  // Token fetching
  const fetchTokens = useCallback(async (address: string): Promise<Token[]> => {
    try {
      setIsLoading(true);
      const endpoint = await getHttpEndpoint({
        network: currentNetwork === 'ton-mainnet' ? 'mainnet' : 'testnet',
      });
      const client = new TonClient({ endpoint });
      const addressObj = Address.parse(address);
      const transactions = await client.getTransactions(addressObj, { limit: 50 });

      const tokens: Token[] = [];
      const processedAddresses = new Set<string>();

      for (const tx of transactions) {
        const infoValue = tx.inMessage?.info?.value;
        const msgBody = tx.inMessage?.body;
        
        if (msgBody instanceof Cell && infoValue) {
          const slice = msgBody.beginParse();
          const bufferLength = Math.floor(slice.bits.length / 8);
          const buffer = slice.loadBuffer(bufferLength);
          const msgText = buffer.toString();

          if (msgText.includes('transfer') || msgText.includes('mint')) {
            const tokenAddress = tx.inMessage?.info?.src?.toString();
            if (tokenAddress && !processedAddresses.has(tokenAddress)) {
              try {
                const tokenContract = Address.parse(tokenAddress);
                const tokenData = await client.runMethod(tokenContract, 'get_token_data', []);
                
                if (tokenData.stack && Array.isArray(tokenData.stack)) {
                  const symbol = tokenData.stack[0]?.value?.toString() || 'Unknown Token';
                  const decimals = parseInt(tokenData.stack[1]?.value?.toString() || '0', 10);

                  const balanceData = await client.runMethod(
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

      const tonBalance = await client.getBalance(addressObj);
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

  // Transaction fetching
  const fetchTransactions = useCallback(async (address: string) => {
    try {
      setIsLoading(true);
      const endpoint = await getHttpEndpoint({
        network: currentNetwork === 'ton-mainnet' ? 'mainnet' : 'testnet'
      });
      const client = new TonClient({ endpoint });
      const txs = await client.getTransactions(Address.parse(address), { limit: 20 });

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

  // Balance fetching
  const fetchBalance = useCallback(async (address: string) => {
    try {
      const endpoint = await getHttpEndpoint({
        network: currentNetwork === 'ton-mainnet' ? 'mainnet' : 'testnet'
      });
      const client = new TonClient({ endpoint });
      const balanceValue = await client.getBalance(Address.parse(address));
      setBalance((Number(balanceValue) / 1e9).toFixed(2));
    } catch (error) {
      console.error('Error fetching balance:', error);
      setBalance('0');
    }
  }, [currentNetwork]);
  // Network handling
  const handleNetworkChange = useCallback(async (network: string) => {
    setCurrentNetwork(network);
    if (userAddress) {
      setIsLoading(true);
      try {
        await Promise.all([
          fetchBalance(userAddress),
          fetchTokens(userAddress),
          fetchTransactions(userAddress)
        ]);
      } catch (e) {
        setError('Failed to update network data');
      } finally {
        setIsLoading(false);
      }
    }
  }, [userAddress, fetchBalance, fetchTokens, fetchTransactions]);

  // Wallet connection handling
  const handleConnect = useCallback(async (wallet: WalletInfo) => {
    try {
      setError('');
      if (!connector) {
        throw new Error('Connector is not initialized');
      }

      if (wallet && 'jsBridgeKey' in wallet) {
        connector.connectWallet();
        return;
      }

      if (wallet && 'universalLink' in wallet) {
        connector.connectWallet();
        if ('universalLink' in wallet) {
            window.location.href = wallet
        }
        return;
      }

      throw new Error('Invalid wallet information');
    } catch (e) {
      setError(e instanceof Error ? e.message : 'An unknown error occurred');
    }
  }, [connector]);
  const handleDisconnect = useCallback(() => {    if (connector) {
      connector.disconnect();
      setCurrentScreen('main');
      setSelectedIndex(0);
      setSelectedSection('networks');
      setSendAmount('');
      setSendAddress('');
      setInputFocus('none');
      setError('');
    }
  }, [connector]);

  // Initial setup
  useEffect(() => {
    const initConnector = async () => {
      const newConnector = new TonConnectUI({
        manifestUrl: 'https://overpass.network/tonconnect-manifest.json'
      });
      
      setConnector(newConnector);

      try {
        const wallet = newConnector
        if (wallet) {
          setConnected(true);
          const address = wallet.account.address;
          setUserAddress(address);
          
          await Promise.all([
            fetchTokens(address),
            fetchTransactions(address),
            fetchBalance(address)
          ]);
        } else {
          setConnected(false);
          setUserAddress("");
          setBalance("0");
          setTokens([]);
          setTransactions([]);
        }
      } catch (error) {
        if (error instanceof Error) {
          setError(error.message);
        } else {
          setError('An unknown error occurred');
        }
      }
    };    initConnector();
  }, [fetchTokens, fetchTransactions, fetchBalance]);

  // Sound effect handlers
  useEffect(() => {
    const sounds = [
      { ref: coinSoundRef, playing: playCoinSound, setter: setPlayCoinSound },
      { ref: lvlSoundRef, playing: playLvlSound, setter: setPlayLvlSound },
      { ref: powerupSoundRef, playing: playPowerupSound, setter: setPlayPowerupSound },
      { ref: pipeSoundRef, playing: playPipeSound, setter: setPlayPipeSound },
      { ref: jumpSoundRef, playing: playJumpSound, setter: setPlayJumpSound }
    ];

    sounds.forEach(({ ref, playing, setter }) => {
      if (playing && ref.current) {
        ref.current.play();
        setter(false);
      }
    });
  }, [playCoinSound, playLvlSound, playPowerupSound, playPipeSound, playJumpSound]);

// Screen rendering logic
const renderScreen = useCallback(() => {
    switch (currentScreen) {
        case 'main':
            return (
                <div className="space-y-2">
                    {connected ? (
                        <>
                            <div className="text-sm mb-4">BALANCE:</div>
                            <div className="text-xl mb-4">{balance} TON</div>
                            <div className="space-y-1">
                                {menuItems.map((item, index) => (
                                    <div 
                                        key={item.label}
                                        className={`flex items-center gap-2 text-xs ${
                                            index === selectedIndex ? 'bg-[#0f380f] text-[#9bbc0f]' : ''
                                        } px-2 py-1`}
                                    >
                                        {index === selectedIndex ? '>' : ' '} {item.label}
                                    </div>
                                ))}
                            </div>
                        </>
                    ) : (
                        <div className="flex flex-col items-center">
                            <div className="self-start">
                                <img
                                    className="w-20 h-20"
                                    src="/chameleon.gif"
                                    alt="Chameleon"
                                    width="16"
                                    height="16"
                                  />
                                <img 
                                    className="w-20 h-20" 
                                    src="/Coin.gif" 
                                    alt="Coin" 
                                    width="16" 
                                    height="16" 
                                />
                            </div>
                            <img 
                                src="/15.png" 
                                alt="Overpass Logo" 
                                className="w-39 h-36 mx-auto mb-4" 
                            />
                            <div className="text-center space-y-4 w-full">
                                <div className="text-sm mb-2">WALLET STATUS</div>
                                <button 
                                    onClick={() => setCurrentScreen('wallets')}
                                    className="w-full bg-[#0f380f] text-[#9bbc0f] px-4 py-2 mb-2 text-xs
                                        hover:bg-[#0f380f]/90 transition-colors duration-200
                                        shadow-[inset_-1px_-1px_1px_rgba(255,255,255,0.2),
                                        inset_2px_2px_2px_rgba(0,0,0,0.3)]"
                                >
                                    CONNECT WALLET
                                </button>
                                
                                {connector && (
                                    <button 
                                        onClick={handleDisconnect}
                                        className="w-full border border-[#0f380f] text-[#0f380f] px-4 py-2 text-xs
                                            hover:bg-[#0f380f] hover:text-[#9bbc0f] transition-colors duration-200"
                                    >
                                        DISCONNECT
                                    </button>
                                )}

                                <div className="text-xs mt-4 opacity-75">
                                    {connector ? (
                                        <>Press A: Connect<br />Press B: Disconnect</>
                                    ) : (
                                        'Press A to connect wallet'
                                    )}
                                </div>
                            </div>
                            
                            {error && (
                                <div className="mt-4 text-xs bg-[#0f380f] text-[#9bbc0f] px-3 py-2 w-full text-center">
                                    {error}
                                </div>
                            )}
                        </div>
                    )}
                </div>
            );
            
        default:
            return null;
    }
}, [currentScreen, connected, balance, menuItems, selectedIndex, connector, 
    error, handleDisconnect]);

return (

    <div className="app-container w-full h-screen flex flex-col bg-[#8b956d] overflow-hidden">
        <AudioPlayer 
            src="/assets/AWESOME.mp3" 
            isPlaying={isAudioOn} 
            volume={volume} 
            loop={true} 
        />

        <audio ref={coinSoundRef} src="/Coin1.mp3" />
        <audio ref={lvlSoundRef} src="/LvF.mp3" />
        <audio ref={powerupSoundRef} src="/Powerup.mp3" />
        <audio ref={pipeSoundRef} src="/Pipe.mp3" />
        <audio ref={jumpSoundRef} src="/Jump.mp3" />
        


        <div className="flex-1 flex flex-col">
            {renderScreen()}
        </div>
    </div>

);};

export default App;

function parseEther(sendAmount: string) {
  throw new Error('Function not implemented.');
}
