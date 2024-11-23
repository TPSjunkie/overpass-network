// ./TokenList.tsx
import React from 'react'
import TokenDetails from './TokenDetails'
import type { TokenDetailsProps } from './TokenDetails'

interface Token {
    tokenId: string
    website: string
    name: string
    symbol: string
    balance: string
    circulatingsupply: string;
    totalsupply: string;
    maxsupply: string;
    timestamp: string;
    lastprice: string;
    volumeto: string;
    volumefrom: string;
    price: string;
    change24h: string;
    changepct24h: string;
}

interface TokenListProps {
    tokens: Token[]
}

const TokenList: React.FC<TokenListProps> = ({ tokens }) => {
    return (
        <div className="token-list">
            {tokens.map((token, index) => (
                <TokenDetails
                    key={index}
                    {...token as unknown as TokenDetailsProps}
                />
            ))}
        </div>
    );
};

export default TokenList;
