// src/components/WalletInterface.tsx

import React, { useState, useEffect } from 'react';
import GroupUIManager from '../utils/GroupUIManager';
import TransactionManager from '../utils/TransactionManager';
import type { Transaction } from '../types/wasm-types';

const WalletInterface: React.FC = () => {
  const [balances, setBalances] = useState({ checking: 0, savings: 0, custom: 0, total: 0 });
  const [selectedGroup, setSelectedGroup] = useState('checking');
  const [amount, setAmount] = useState('');
  const [transactions, setTransactions] = useState<Transaction[]>([]);

  useEffect(() => {
    updateBalances();
    fetchTransactions();
  }, []);

  const updateBalances = async () => {
    const address = await GroupUIManager.getWalletAddress();
    const groupBalances = await GroupUIManager.getGroupBalances(address);
    setBalances({
      checking: Number(groupBalances.checking),
      savings: Number(groupBalances.savings),
      custom: Number(groupBalances.custom),
      total: Number(groupBalances.total)
    });
  };

  const fetchTransactions = async () => {
    const address = await GroupUIManager.getWalletAddress();
    const fetchedTransactions = await TransactionManager.getTransactions(address);
    if (fetchedTransactions !== undefined) {
      setTransactions(fetchedTransactions);
    }
  };

  const handleTransaction = async () => {
    try {
      const newTransaction = await TransactionManager.initiateTransaction(Number(amount), selectedGroup);
      await updateBalances();
      await fetchTransactions();
      setAmount('');
    } catch (error) {
      console.error('Transaction failed:', error);
      // Handle error (e.g., show error message to user)
    }
  };

  return (
    <div className="bg-gray-800 p-6 rounded-lg shadow-lg">
      <h2 className="text-2xl font-bold text-green-400 mb-4">Overpass Wallet</h2>
      {/* Balance display and transaction form (as before) */}
      {/* ... */}
      <div className="mt-8">
        <h3 className="text-xl font-semibold text-green-300 mb-4">Transaction History</h3>
        <div className="overflow-x-auto">
          <table className="min-w-full bg-gray-700 text-white">
            <thead>
              <tr>
                <th className="px-4 py-2">Date</th>
                <th className="px-4 py-2">Type</th>
                <th className="px-4 py-2">Amount</th>
                <th className="px-4 py-2">Status</th>
              </tr>
            </thead>
            <tbody>
              {transactions.map((tx) => (
                <tr key={tx.id}>
                  <td className="px-4 py-2">{new Date(tx.timestamp).toLocaleString()}</td>
                  <td className="px-4 py-2">{tx.type}</td>
                  <td className="px-4 py-2">{tx.amount} TON</td>
                  <td className="px-4 py-2">{tx.status}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
};
export default WalletInterface;