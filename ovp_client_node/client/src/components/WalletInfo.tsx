// src/components/WalletInfo.tsx

import React from 'react';

// Define the props interface
interface WalletInfoProps {
  address: string;
  balance: string;
}

// Define the WalletInfo component with typed props
const WalletInfo: React.FC<WalletInfoProps> = ({ address, balance }) => {
  return (
    <div className="wallet-info">
      <p><strong>Address:</strong> {address}</p>
      <p><strong>Balance:</strong> {balance}</p>
    </div>
  );
};

export default WalletInfo;
