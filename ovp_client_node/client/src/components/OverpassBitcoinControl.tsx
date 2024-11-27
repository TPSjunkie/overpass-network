// ./src/components/BitcoinControl.tsx

/// This is the main component for the bitcoin control page. 
//It contains themain content and the sidebar.

import { useState, useEffect, useCallback } from 'react';
import { useNavigate } from 'react-router-dom';
import { useWallet } from '@/hooks/useWallet';
import { useBitcoinBridge } from '@/hooks/useBitcoinBridge';
import { useBitcoinBridgeState } from '@/hooks/useBitcoinBridgeState';
import { useBitcoinBridgeConfig } from '@/hooks/useBitcoinBridgeConfig';
import { useBitcoinBridgeWallet } from '@/hooks/useBitcoinBridgeWallet';
import { useBitcoinBridgeChannels } from '@/hooks/useBitcoinBridgeChannels';
import { useBitcoinBridgeTransactions } from '@/hooks/useBitcoinBridgeTransactions';
import { useBitcoinBridgeEvents } from '@/hooks/useBitcoinBridgeEvents';
import { useBitcoinBridgeStorage } from '@/hooks/useBitcoinBridgeStorage';
import { useBitcoinBridgeNetwork } from '@/hooks/useBitcoinBridgeNetwork';

// 
/// This is the main component for the bitcoin control page. 
//It contains themain content and the sidebar.      

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