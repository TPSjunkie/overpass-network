import { useState, useCallback } from 'react';
import { useTonConnectUI } from '@tonconnect/ui-react';
import type { TonConnectUI, WalletInfo as UIWalletInfo } from '@tonconnect/ui-react';
import type { CHAIN } from '@tonconnect/protocol';
interface WalletInfoRemote {
  name: string;
  universalLink: string;
  bridgeUrl: string;
}


interface LocalWalletInfo {
  address: string;
  publicKey: string;
  walletType: string;
  chain: CHAIN;
}

interface SendTransactionRequest {
  validUntil: number;
  messages: Array<{
    address: string;
    amount: string;
    payload?: string;
  }>;
}

interface ExtendedWalletInfo extends LocalWalletInfo {
  [x: string]: any;
  balance?: string;
  icon: string;
  address: string;
}

interface WalletConnection {
  jsBridgeKey?: string;
  universalUrl?: string;
  bridgeUrl?: string;
  injected?: boolean;
}

interface TonConnector {
  connected: boolean;
  connecting: boolean;
  address: string | null;
  network: string | null;
  wallet: WalletConnection | null;
  getWallets: () => Promise<UIWalletInfo[]>;
  connect: (wallet: WalletConnection) => Promise<any>;
  disconnect: () => Promise<void>;
  sendTransaction: (transaction: SendTransactionRequest) => Promise<void>;
}

const DEFAULT_WALLET_CONNECTION: WalletConnection = {
  jsBridgeKey: 'tonkeeper',
  bridgeUrl: 'https://bridge.tonapi.io/bridge',
  universalUrl: 'https://app.tonkeeper.com/ton-connect',
  injected: false
};

export function useTonConnect() {
  const [walletInfo, setWalletInfo] = useState<ExtendedWalletInfo | null>(null);
  const [channelAddress] = useState<string | null>(null);
  const [channelState] = useState<any | null>(null);
  const [isLoading, setIsLoading] = useState<boolean>(false);
  const [error, setError] = useState<string | null>(null);
  const [tonConnectUI] = useTonConnectUI();

  const getWalletInfo = useCallback(
    async (instance: TonConnectUI): Promise<ExtendedWalletInfo | null> => {
      try {
        const connector = instance.connector as unknown as TonConnector;
        const wallets = await connector.getWallets();
        if (!wallets || wallets.length === 0) return null;

        const wallet = wallets[0] as unknown as LocalWalletInfo & { 
          address: string; 
          imageUrl: string;
        };

        let balance = '0';
        try {
          if (wallet.address) {
            // Balance fetching logic would go here
            // Example: balance = await getBalanceForAddress(wallet.address);
          }
        } catch (err) {
          console.warn('Failed to fetch balance:', err);
        }

        return {
          ...wallet,
          balance,
          icon: wallet.imageUrl,
          address: wallet.address
        };
      } catch (err) {
        console.error('Error getting wallet info:', err);
        return null;
      }
    },
    []
  );

  const handleConnect = useCallback(async () => {
    if (!tonConnectUI) return;
    try {
      setIsLoading(true);
      const connector = tonConnectUI.connector as unknown as TonConnector;
      await connector.connect(DEFAULT_WALLET_CONNECTION);
      const newWalletInfo = await getWalletInfo(tonConnectUI);
      setWalletInfo(newWalletInfo);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to connect wallet');
    } finally {
      setIsLoading(false);
    }
  }, [tonConnectUI, getWalletInfo]);

  const handleDisconnect = useCallback(async () => {
    if (!tonConnectUI) return;
    try {
      setIsLoading(true);
      const connector = tonConnectUI.connector as unknown as TonConnector;
      await connector.disconnect();
      setWalletInfo(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to disconnect wallet');
    } finally {
      setIsLoading(false);
    }
  }, [tonConnectUI]);

  const handleOpenModal = useCallback(async () => {
    if (!tonConnectUI) return;
    try {
      setIsLoading(true);
      tonConnectUI.open({
        modalProps: {
          title: "Connect Wallet",
          description: "Please connect your wallet to continue",
          actionButton: {
            text: "Connect",
            onClick: handleConnect,
          },
        },
      });
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to open modal');
    } finally {
      setIsLoading(false);
    }
  }, [tonConnectUI]);

  const handleCloseModal = useCallback(async () => {
    if (!tonConnectUI) return;
    try {
      setIsLoading(true);
      if ('close' in tonConnectUI) {
        (tonConnectUI as any).close();
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to close modal');
    } finally {
      setIsLoading(false);
    }
  }, [tonConnectUI]);

  const subscribeToConnection = useCallback(() => {
    if (!tonConnectUI) return;
    
    const connector = tonConnectUI.connector as unknown as TonConnector;
    if (connector.connected) {
      getWalletInfo(tonConnectUI).then(newWalletInfo => {
        if (newWalletInfo) {
          setWalletInfo(newWalletInfo);
        }
      }).catch(err => {
        console.error('Failed to get wallet info:', err);
      });
    }
  }, [tonConnectUI, getWalletInfo]);

  return {
    tonConnectUI,
    walletInfo,
    channelAddress,
    channelState,
    isLoading,
    error,
    handleConnect,
    handleDisconnect,
    handleOpenModal,
    handleCloseModal,
    subscribeToConnection
  };
}