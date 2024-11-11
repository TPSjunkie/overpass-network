
import { TonConnectUI, useTonConnectUI } from '@tonconnect/ui-react';
import {  } from "@/bridge/Interfaces";

import { useState } from 'react';
import { useTonConnect } from '@/hooks/useTonConnect';
import { Address } from '@ton/core';

export const ChannelController = () => {
  const { walletInfo } = useTonConnect();
  const [channelId, setChannelId] = useState<number>(0);
  const [amount, setAmount] = useState<string>('0');
  
  const handleChannelOperation = async (channelId: number, amount: bigint) => {
    if (walletInfo?.address) {
      controlChannel(Address.parse(walletInfo.address), channelId, amount);
    }
  };
  
  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    handleChannelOperation(channelId, BigInt(amount));
  };
  
  return (
    <div className="channel-controller">
      <form onSubmit={handleSubmit}>
        <div>
          <label htmlFor="channelId">Channel ID:</label>
          <input
            type="number"
            id="channelId"
            value={channelId}
            onChange={(e) => setChannelId(Number(e.target.value))}
            min="0"
            required
          />
        </div>
        <div>
          <label htmlFor="amount">Amount:</label>
          <input
            type="text"
            id="amount"
            value={amount}
            onChange={(e) => setAmount(e.target.value)}
            pattern="[0-9]*"
            required
          />
        </div>
        <button type="submit" disabled={!walletInfo?.address}>
          Control Channel
        </button>
      </form>
    </div>
  );
}

function controlChannel(arg0: Address, channelId: number, amount: bigint) {
  throw new Error('Function not implemented.');
}
