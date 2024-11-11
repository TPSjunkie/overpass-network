// src/components/ChannelControls.tsx
import { useState, useCallback, useEffect } from 'react';
import type { ReactNode } from 'react';
import { useTonConnect } from '@/hooks/useTonConnect';
import { useGroupChannels } from '../hooks/useGroupChannels';
import { BalanceCache } from '../utils/balanceCache';
import { WasmTransactionSender, TransactionType, type ChannelTransactionParams } from '@/utils/wasmTransactionSender';

import { createCell } from '../utils/ton';

interface ExtendedWalletInfo {
  address: string;
  wallet: any;
}

interface Channel {
  id: number;
  balance?: string;
  address?: string;
}

interface ChannelControlsProps {
  children?: ReactNode;
}

const ChannelControls: React.FC<ChannelControlsProps> = ({ children }) => {
  const { walletInfo, channelAddress } = useTonConnect();
  const { data: channels, isLoading, error } = useGroupChannels(1);
  const balanceCache = BalanceCache.getInstance();
  const [selectedIndex, setSelectedIndex] = useState<number | null>(null);
  const [operationError, setOperationError] = useState<string | null>(null);
  const [isProcessing, setIsProcessing] = useState(false);

  const createChannelTransaction = async (params: ChannelTransactionParams): Promise<void> => {
    const sender = WasmTransactionSender.getInstance();
    await sender.sendChannelTransaction(
      params.recipient,
      params.amount,
      params.payload,
      params.channelId,
      params.groupId,
      params.transactionType,
      params.stateInit,
      walletInfo?.address,
      params,
      getContract()
    );
    setOperationError(null);
    setIsProcessing(false);
    setSelectedIndex(null);
  };

  const handleChannelOperation = useCallback(async (channelId: number, amount: bigint,
    operationType: 'send' | 'withdraw'
  ): Promise<void> => {
    if (!walletInfo?.address || !channelAddress) {
      setOperationError('Wallet not connected');
      return;
    }

    setIsProcessing(true);
    setOperationError(null);

    try {
      const selectedChannel = channels?.find((ch: { id: number; }) => ch.id === channelId);
      if (!selectedChannel) throw new Error('Channel not found');
    
      const recipientAddress = selectedChannel.address;
      if (!recipientAddress) throw new Error('Invalid channel address');

      const messageCell = createCell({
        address: recipientAddress,
        amount,
        payload: "Hello, world!"
      });

      const transactionParams: ChannelTransactionParams = {
        recipient: recipientAddress,
        amount: amount.toString(), // Convert bigint to string
        payload: messageCell.toBoc().toString('base64'), // Convert Cell to base64 string
        stateInit: undefined,
        channelId: channelId.toString(),
        groupId: '1',
        transactionType: operationType === 'send' ? TransactionType.CHANNEL_INIT : TransactionType.CHANNEL_WITHDRAW,
        flags: '0', // Changed to string
        bounce: false // Changed to boolean
      };
  
    const currentBalance = BigInt(balanceCache.getBalance(channelId.toString()));
  
    // Update local balance immediately for better UX
        const newBalance =
          operationType === "send"
            ? currentBalance - amount
            : currentBalance + amount;
  
      balanceCache.setBalance(channelId.toString(), newBalance.toString());      
      await createChannelTransaction(transactionParams);
    
      setSelectedIndex(null);
      setOperationError(null);

    } catch (err) {
      console.error('Channel operation failed:', err);
      setOperationError(err instanceof Error ? err.message : 'Operation failed');
      balanceCache.getBalance(channelId.toString());
    } finally {
      setIsProcessing(false);
    }

  }, [walletInfo, channelAddress, channels, balanceCache, createChannelTransaction, setSelectedIndex, setOperationError, setIsProcessing]);

  const handleKeyPress = useCallback((event: KeyboardEvent) => {
    if (selectedIndex === null || isProcessing) return;

    const channel = channels?.[selectedIndex];
    if (!channel) return;

    const amount = BigInt(1e9);

    switch (event.key.toLowerCase()) {
      case 'a':
        handleChannelOperation(channel.id, amount, 'send');
        break;
      case 'b':
        handleChannelOperation(channel.id, amount, 'withdraw');
        break;
    }
  }, [selectedIndex, channels, isProcessing, handleChannelOperation]);

  useEffect(() => {
    window.addEventListener('keydown', handleKeyPress);
    return () => window.removeEventListener('keydown', handleKeyPress);
  }, [handleKeyPress]);

  return <>{children}</>;
}
  useEffect(() => {
    window.addEventListener('keydown', handleKeyPress);
    return () => window.removeEventListener('keydown', handleKeyPress);
  }, [handleKeyPress]);

  if (isLoading) {
    return (
      <div className="bg-[#4a533a] p-6 rounded-lg shadow-lg">
        <p className="text-[#9bbc0f]">Loading channels...</p>
      </div>
    );
  }

  if (error) {
    return (
      <div className="bg-[#4a533a] p-6 rounded-lg shadow-lg">
        <p className="text-red-500">
          {typeof error === 'string' ? error : error instanceof Error ? error.message : 'Unknown error'}
        </p>
      </div>
    );
  }

  return (
    <div className="space-y-2">
      <div className="text-sm mb-4">ACTIVE CHANNELS:</div>
      
      <div className="space-y-1">
        {channels?.map((channel: Channel, index: number) => (
          <div 
            key={channel.id}
            onClick={() => setSelectedIndex(index)}
            className={`flex items-center justify-between text-xs cursor-pointer
              ${index === selectedIndex ? 'bg-[#0f380f] text-[#9bbc0f]' : ''}
              ${isProcessing ? 'opacity-50 cursor-not-allowed' : ''}
              px-2 py-1 hover:bg-[#0f380f] hover:text-[#9bbc0f] transition-colors`}
          >
            <div className="flex items-center gap-2">
              {typeof index === 'number' && typeof selectedIndex === 'number' && 
                <span>{index === selectedIndex ? '>' : ' '}</span>}
              <span>CH-{channel.id.toString()}</span>
            </div>
            <span>
              {(balanceCache.getBalances(channel.id.toString()) || '0')} TON
            </span>
          </div>
        ))}
      </div>

      <div className="text-xs mt-4">
        <div>A: SEND TON</div>
        <div>B: WITHDRAW TON</div>
        {!walletInfo && (
          <div className="text-[#4a533a]">CONNECT WALLET TO USE CHANNELS</div>
        )}
        {operationError && (
          <div className="text-red-500 mt-2">{operationError}</div>
        )}
        {isProcessing && (
          <div className="text-[#9bbc0f]">Processing transaction...</div>
        )}
      </div>
    </div>
  );
};

export default ChannelControls;

function getContract(): import("@ton/core").Contract {
  throw new Error('Function not implemented.');
}
