// src/components/GroupBalances.tsx

import React from "react";
import { useGroupBalances, type GroupBalance } from "../hooks/useGroupBalances";
import { useGroupChannels } from "../hooks/useGroupChannels";

interface GroupBalancesProps {
  groupId: number;
}

const GroupBalancesComponent: React.FC<GroupBalancesProps> = ({ groupId }) => {
  const { data: balances, isLoading: balancesLoading } = useGroupBalances();
  const { data: channels, isLoading: channelsLoading } = useGroupChannels(groupId);

  if (balancesLoading || channelsLoading) {
    return <div>Loading...</div>;
  }

  if (!balances || !channels) {
    return <div>No data available</div>;
  }

  return (
    <div>
      <h2>Group Balances</h2>
      <ul>
        {balances.map((balance: GroupBalance, index: number) => (
          <li key={index}>
            {channels[index]?.name || `Channel ${index + 1}`}: {balance.amount} TON
          </li>
        ))}
      </ul>
    </div>
  );
};

export default GroupBalancesComponent;
