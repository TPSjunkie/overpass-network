// src/components/TransactionDetails.tsx

import React, { useState, useEffect } from 'react';
import { Transaction } from '../types/wasm-types';
import { useTonConnect } from '../hooks/useTonConnect';
import WalletInfo from './WalletInfo';
import SendTransaction from './SendTransaction';
import TransactionHistory from './TransactionHistory';
import TransactionManager from '../utils/TransactionManager';

interface TransactionDetailsProps {
  transaction: Transaction;
}

const TransactionDetails: React.FC<TransactionDetailsProps> = ({ transaction }) => {
  const { walletInfo } = useTonConnect();
  const [transactions, setTransactions] = useState<Transaction[]>([]);

  const fetchTransactions = async () => {
    try {
      if (walletInfo) {
        const address = walletInfo.address;
        const fetchedTransactions = await TransactionManager.fetchTransactions(address);
        if (Array.isArray(fetchedTransactions)) {
          setTransactions(fetchedTransactions);
        } else {
          console.error('Fetched transactions is not an array');
        }
      }
    } catch (error) {
      console.error('Failed to fetch transactions:', error);
    }
  };

  useEffect(() => {
    fetchTransactions();
  }, [walletInfo]);

  return (
    <div className="pip-boy-screen p-4 rounded-lg shadow-pip-boy">
      <WalletInfo />
      <div className="pip-boy-panel mt-4 p-4 rounded-lg">
        <h3 className="text-xl font-semibold glow-text mb-4">Transaction Details</h3>
        <div className="grid grid-cols-2 gap-4">
          <DetailItem label="ID" value={transaction.id} />
          <DetailItem label="Date" value={new Date(transaction.timestamp).toLocaleString()} />
          <DetailItem 
            label="Type" 
            value={transaction.type} 
            className={transaction.type === 'incoming' ? 'text-green-400' : 'text-red-400'}
          />
          <DetailItem label="Amount" value={`${transaction.amount} TON`} />
          <DetailItem label="Sender" value={transaction.sender} />
          <DetailItem label="Recipient" value={transaction.recipient} />
          <DetailItem label="Status" value={transaction.status} />
          {transaction.description && (
            <DetailItem label="Description" value={transaction.description} className="col-span-2" />
          )}
        </div>
      </div>
      <SendTransaction onSendTransaction={async () => {
        // Implement send transaction logic
        await fetchTransactions();
      }} />
      <TransactionHistory transactions={transactions} fetchTransactions={fetchTransactions} />
    </div>
  );
};

const DetailItem: React.FC<{ label: string; value: string; className?: string }> = ({ label, value, className = '' }) => (
  <div className={`pip-boy-text ${className}`}>
    <p className="font-semibold">{label}:</p>
    <p className="glitch-text">{value}</p>
  </div>
);

export default TransactionDetails;