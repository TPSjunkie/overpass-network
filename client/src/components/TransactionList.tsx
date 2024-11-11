// ./src/components/ChannelList.tsx
import * as React from 'react';
import { FixedSizeList as List } from 'react-window';

interface Token {
  name: string;
  symbol: string;
  balance: string;
}

interface TokenListProps {
  tokens: Token[];
}
const ChannelList: React.FC<{ channels: string[] }> = ({ channels }) => {
  const Row = ({ index, style }) => (
    <div style={style}>
      Channel {channels[index]}
    </div>
  );

  return (
    <List
      height={400}
      itemCount={channels.length}
      itemSize={35}
      width={300}
    >
      {Row}
    </List>
  );
};
const TokenList: React.FC<TokenListProps> = ({ tokens }) => {
  return (
    <div className="bg-gray-700 p-4 rounded-lg shadow-md">
      <h2 className="text-xl font-semibold mb-4">Tokens</h2>
      <ul className="space-y-2">
        {tokens.map((token, index) => (
          <li key={index} className="flex justify-between items-center">
            <span className="text-lg font-semibold">{token.name}</span>
            <span className="text-sm text-gray-400">{token.symbol}</span>
            <span className="text-lg font-semibold">{token.balance}</span>
          </li>
        ))}
      </ul>
    </div>
  );
};

export default TokenList; // Default export
