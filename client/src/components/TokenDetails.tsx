// ./components/TokenDetails.tsx
import React from "react";
import Balance from "./Balance";
import { isNum } from "react-toastify/dist/utils";
export interface TokenDetailsProps {
    tokenId: string;
    name: string;
    symbol: string;
    decimals: number;
    totalSupply: string;
    owner: string;
    website: string;
    balance: string;
}


// Price Chart api caller
const fetchPrice = async (tokenId: string) => {
    const response = await fetch(`/api/token/${tokenId}/price`);
    const data = await response.json();
    return data.price;
};

const TokenDetails: React.FC<TokenDetailsProps> = ({ tokenId, name, symbol, decimals, balance, totalSupply, owner, website }) => {
    const [price, setPrice] = React.useState<number | null>(null);

    React.useEffect(() => {
        const fetchPriceData = async () => {
            const priceData = await fetchPrice(tokenId);
            setPrice(priceData);
        };
        fetchPriceData();
    }, [tokenId]);

    return (
        <div className="token-details">
            <h2>{name}</h2>
            <p>Symbol: {symbol}</p>
            <p>Decimals: {decimals}</p>
            <p>Total Supply: {totalSupply}</p>
            <p>Owner: {owner}</p>
            <p>Website: {website}</p>
            <p>balance: {null}</p>
            {price !== null && <p>Price: {price}</p>}
        </div>
    );
};

export default TokenDetails;

