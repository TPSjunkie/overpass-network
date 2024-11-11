// ./src/components/TransactionInitiator.tsx
import React from "react";
import { useTonConnect } from "../hooks/useTonConnect";
import { useGroupChannels } from "../hooks/useGroupChannels";
import { useGroupBalances } from "../hooks/useGroupBalances";

interface GroupChannel {
  id: number;
  address: string;
}

const TransactionInitiator: React.FC = () => {
  const { walletInfo } = useTonConnect();
  const { data: groupChannels = [] } = useGroupChannels(1);
  return (
    <div>
      <h2>Transaction Initiator</h2>
      <ul>
        {groupChannels.map((channel: GroupChannel) => (
          <li key={channel.id}>
            Channel {channel.id}: {channel.address}
          </li>
        ))}
      </ul>
    </div>
  );
};
export default TransactionInitiator;