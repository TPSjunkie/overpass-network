// src/components/SendTransaction.tsx

import React, { useState } from 'react';

interface SendTransactionProps {
  onSendTransaction: (to: string, amount: string, payload: string) => Promise<void>;
}

const SendTransaction: React.FC<SendTransactionProps> = ({ onSendTransaction }) => {
  const [to, setTo] = useState('');
  const [amount, setAmount] = useState('');
  const [payload, setPayload] = useState('');

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!to || !amount) {
      // Optionally, display an error message
      return;
    }
    await onSendTransaction(to, amount, payload);
    setTo('');
    setAmount('');
    setPayload('');
  };

  return (
    <form onSubmit={handleSubmit} className="send-transaction-form space-y-4">
      <div>
        <label htmlFor="to" className="block text-pip-boy-text mb-1">To:</label>
        <input
          id="to"
          type="text"
          value={to}
          onChange={(e) => setTo(e.target.value)}
          className="w-full p-2 rounded bg-pip-boy-dark-green text-pip-boy-text border border-pip-boy-border"
          required
        />
      </div>
      <div>
        <label htmlFor="amount" className="block text-pip-boy-text mb-1">Amount (TON):</label>
        <input
          id="amount"
          type="number"
          value={amount}
          onChange={(e) => setAmount(e.target.value)}
          className="w-full p-2 rounded bg-pip-boy-dark-green text-pip-boy-text border border-pip-boy-border"
          required
          min="0"
          step="any"
        />
      </div>
      <div>
        <label htmlFor="payload" className="block text-pip-boy-text mb-1">Payload (Optional):</label>
        <input
          id="payload"
          type="text"
          value={payload}
          onChange={(e) => setPayload(e.target.value)}
          className="w-full p-2 rounded bg-pip-boy-dark-green text-pip-boy-text border border-pip-boy-border"
        />
      </div>
      <button type="submit" className="bg-pip-boy-button-green hover:bg-pip-boy-button-hover-green text-black py-2 px-4 rounded w-full">
        Send
      </button>
    </form>
  );
};

export default SendTransaction;
