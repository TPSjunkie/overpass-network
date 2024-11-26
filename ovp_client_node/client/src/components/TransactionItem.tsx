// src/components/TransactionItem.tsx

import React from 'react';
import { Transaction } from '../types/wasm-types';

interface TransactionItemProps {
  transaction: Transaction;
}

const TransactionItem: React.FC<TransactionItemProps> = ({ transaction }) => {
  return (
    <div className="transaction-item bg-pip-boy-panel p-4 rounded-lg shadow-pip-boy mb-4">
      <div className="flex justify-between items-center">
        <div>
          <p className="text-pip-boy-green font-semibold">{transaction.type === 'incoming' ? 'Received' : 'Sent'}</p>
          <p className="text-pip-boy-text break-all">{transaction.sender} → {transaction.recipient}</p>
        </div>
        <p className={`font-semibold ${transaction.type === 'incoming' ? 'text-pip-boy-button-green' : 'text-pip-boy-button-red'}`}>
          {transaction.amount} TON
        </p>
      </div>
      <div className="mt-2">
        <p className="text-pip-boy-text text-sm">Date: {new Date(transaction.date).toLocaleString()}</p>
        {transaction.payload && <p className="text-pip-boy-text text-sm">Payload: {transaction.payload}</p>}
      </div>
    </div>
  );
};

export default TransactionItem;
