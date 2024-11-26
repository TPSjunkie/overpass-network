// src/components/Balance.tsx

import React from 'react';

interface BalanceProps {
  balance: string;
  // Add any other props you need here
  // For example, you might need a function to handle balance updates
  // onBalanceUpdate?: (newBalance: string) => void;
  // You can add more props as needed for your component  
}

const Balance: React.FC<BalanceProps> = ({ balance }) => {
  return (
    <div className="balance bg-pip-boy-panel p-4 rounded-lg shadow-pip-boy w-full max-w-md">
      <h3 className="text-xl font-semibold text-pip-boy-green">Balance</h3>
      <p className="text-2xl text-pip-boy-text">{balance} TON</p>
    </div>
  );
};

export default Balance;
