// ./src/components/TransactionForm.tsx
import React, { useState } from "react";
interface TransactionFormProps {
    onSubmit: (amount: string, recipient: string) => void;
}
const TransactionForm: React.FC<TransactionFormProps> = ({ onSubmit }) => {
    const [amount, setAmount] = useState("");
    const [recipient, setRecipient] = useState("");
    const handleSubmit = (e: React.FormEvent) => {
        e.preventDefault();
        onSubmit(amount, recipient);
    };
    return (
        <form onSubmit={handleSubmit}>
            <input
                type="text"
                placeholder="Amount"
                value={amount}
                onChange={(e) => setAmount(e.target.value)}
            />
            <input
                type="text"
                placeholder="Recipient Address"
                value={recipient}
                onChange={(e) => setRecipient(e.target.value)}
            />
            <button type="submit">Send Transaction</button>
        </form>
    );
};
export default TransactionForm;
