// src/components/TransactionHistory.tsx

import React, { useEffect } from 'react';
import { Transaction } from "@/types/wasm-types/index";

interface TransactionHistoryProps {
    transactions: Transaction[];
    fetchTransactions: () => Promise<Transaction[]>;
}

const TransactionHistory: React.FC<TransactionHistoryProps> = ({ transactions, fetchTransactions }) => {
    useEffect(() => {
        const getTransactions = async () => {
            const fetchedTransactions = await fetchTransactions();
            // Handle fetched transactions if needed
        };
        getTransactions();
    }, [fetchTransactions]);

    return (
        <div className="transaction-list space-y-2 overflow-y-auto max-h-64">
            {transactions.length === 0 ? (
                <p className="text-white">No transactions yet.</p>
            ) : (
                transactions.map((tx, index) => (
                    <div key={index} className="bg-gray-700 p-2 rounded-lg">
                        <p className="text-white">
                            <strong>{tx.type}:</strong> {tx.amount} TON
                        </p>
                        <p className="text-gray-300">
                            <strong>To/From:</strong> {tx.address}
                        </p>
                    </div>
                ))
            )}
        </div>
    );
};

export default TransactionHistory;
