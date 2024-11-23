// ./src/components/WalletInterface.tsx

import React, { useState, useEffect } from "react";
import { useTonWallet, useTonAddress } from '@tonconnect/ui-react';
import GroupUIManager from "@/utils/GroupUIManager";
import TransactionManager from "@/utils/TransactionManager";
import { Wallet } from "lucide-react";
import initWasm from "@/wasm/wasmINIT/initWASM";
import getOverpassData from '../../hooks/getOverpassData';
import { toast } from 'react-toastify';
import { BigNumber } from 'bignumber.js';

interface Balances {
    checking: BigNumber;
    savings: BigNumber;
    custom: BigNumber;
    total: BigNumber;
}

interface Transaction {
    id: string;
    date: string;
    description: string;
    amount: BigNumber;
    type: 'credit' | 'debit';
}

const WalletInterface: React.FC = () => {
    const userFriendlyAddress = useTonAddress();
    const wallet = useTonWallet();

    const [balances, setBalances] = useState<Balances>({
        checking: new BigNumber(0),
        savings: new BigNumber(0),
        custom: new BigNumber(0),
        total: new BigNumber(0)
    });
    const [selectedGroup, setSelectedGroup] = useState<string>('checking');
    const [amount, setAmount] = useState<string>('');
    const [transactions, setTransactions] = useState<Transaction[]>([]);
    const [isLoading, setIsLoading] = useState<boolean>(false);

    useEffect(() => {
        const initializeWallet = async () => {
            try {
                await initWasm();
                await updateBalances();
                await fetchTransactions();
            } catch (error) {
                console.error('Error initializing wallet:', error);
                toast.error('Failed to initialize wallet. Please try again.');
            }
        };

        if (wallet && userFriendlyAddress) {
            initializeWallet();
        }
    }, [wallet, userFriendlyAddress, selectedGroup]);

    const updateBalances = async () => {
        setIsLoading(true);
        try {
            const groupBalances = await GroupUIManager.getGroupBalances(userFriendlyAddress);
            setBalances({
                checking: new BigNumber(groupBalances.checking.toString()),
                savings: new BigNumber(groupBalances.savings.toString()),
                custom: new BigNumber(groupBalances.custom.toString()),
                total: new BigNumber(groupBalances.total.toString())
            });
        } catch (error) {
            console.error('Error updating balances:', error);
            toast.error('Failed to update balances. Please try again.');
            setBalances({
                checking: new BigNumber(0),
                savings: new BigNumber(0),
                custom: new BigNumber(0),
                total: new BigNumber(0)
            });
        } finally {
            setIsLoading(false);
        }
    };

    const fetchTransactions = async () => {
        setIsLoading(true);
        try {
            const fetchedTransactions = await TransactionManager.fetchTransactions(userFriendlyAddress);
            setTransactions(fetchedTransactions.map(tx => ({
                ...tx,
                amount: new BigNumber(tx.amount),
                type: tx.type === 'incoming' ? 'credit' : tx.type === 'outgoing' ? 'debit' : tx.type
            } as Transaction)));
        } catch (error) {
            console.error("Error fetching transactions:", error);
            toast.error('Failed to fetch transactions. Please try again.');
            setTransactions([]);
        } finally {
            setIsLoading(false);
        }
    };

    const renderTransactions = () => {
        if (isLoading) {
            return <p>Loading transactions...</p>;
        }

        if (transactions.length === 0) {
            return <p>No transactions available.</p>;
        }

        return transactions.map((transaction) => (
            <div key={transaction.id} className="transaction-item">
                <span>{transaction.date}</span>
                <span>{transaction.description}</span>
                <span className={transaction.type === 'credit' ? 'text-green-500' : 'text-red-500'}>
                    {transaction.type === 'credit' ? '+' : '-'} {transaction.amount.toFormat(2)} TON
                </span>
            </div>
        ));
    };

    const handleGroupChange = (group: string) => {
        setSelectedGroup(group);
    };

    const handleAmountChange = (e: React.ChangeEvent<HTMLInputElement>) => {
        const value = e.target.value;
        if (/^\d*\.?\d*$/.test(value) || value === '') {
            setAmount(value);
        }
    };

    const handleTransfer = async () => {
        if (!amount || isNaN(parseFloat(amount)) || parseFloat(amount) <= 0) {
            toast.error('Please enter a valid amount');
            return;
        }

        setIsLoading(true);
        try {
            const amountBN = new BigNumber(amount);
            if (amountBN.isGreaterThan(balances[selectedGroup as keyof Balances])) {
                throw new Error('Insufficient funds');
            }

            await GroupUIManager.transferFunds(userFriendlyAddress, selectedGroup, amountBN);
            toast.success('Transfer successful');
            await updateBalances();
            await fetchTransactions();
            setAmount('');
        } catch (error) {
            console.error('Error transferring funds:', error);
            toast.error(error instanceof Error ? error.message : 'Transfer failed. Please try again.');
        } finally {
            setIsLoading(false);
        }
    };

    if (!wallet || !userFriendlyAddress) {
        return <div>Please connect your wallet to use this interface.</div>;
    }

    return (
        <div className="wallet-interface">
            <h2 className="text-2xl font-bold mb-4">Wallet Interface</h2>
            <div className="balances mb-6">
                <h3 className="text-xl font-semibold mb-2">Balances</h3>
                <p>Checking: {balances.checking.toFormat(2)} TON</p>
                <p>Savings: {balances.savings.toFormat(2)} TON</p>
                <p>Custom: {balances.custom.toFormat(2)} TON</p>
                <p className="font-bold">Total: {balances.total.toFormat(2)} TON</p>
            </div>
            <div className="transfer-section mb-6">
                <h3 className="text-xl font-semibold mb-2">Transfer Funds</h3>
                <select
                    value={selectedGroup}
                    onChange={(e) => handleGroupChange(e.target.value)}
                    className="mb-2 p-2 border rounded"
                    disabled={isLoading}
                >
                    <option value="checking">Checking</option>
                    <option value="savings">Savings</option>
                    <option value="custom">Custom</option>
                </select>
                <input
                    type="text"
                    value={amount}
                    onChange={handleAmountChange}
                    placeholder="Enter amount"
                    className="mb-2 p-2 border rounded"
                    disabled={isLoading}
                />
                <button
                    onClick={handleTransfer}
                    className="bg-blue-500 text-white p-2 rounded"
                    disabled={isLoading}
                >
                    {isLoading ? 'Processing...' : 'Transfer'}
                </button>
            </div>
            <div className="transactions">
                <h3 className="text-xl font-semibold mb-2">Transactions</h3>
                {renderTransactions()}
            </div>
        </div>
    );
};

export default WalletInterface;
