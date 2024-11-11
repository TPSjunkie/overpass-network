// src/components/CoinDetail.tsx

import React from 'react';
import { useCoinData } from '../hooks/useCoinData';
import '../styles/CoinDetail.css';

interface CoinDetailProps {
  coinId: string;
}

const CoinDetail: React.FC<CoinDetailProps> = ({ coinId }) => {
  const { coinData, loading, error } = useCoinData(coinId);

  if (loading) {
    return <div className="loading">Loading {coinId} data...</div>;
  }

  if (error) {
    return <div className="error">Error: {error}</div>;
  }

  return (
    <div className="coinDetailContainer">
      <h2>{coinData?.name} ({coinData?.symbol})</h2>
      <p>Rank: {coinData?.rank}</p>
      <p>Type: {coinData?.type}</p>
      <p>{coinData?.description}</p>
      {/* Add more details as needed */}
    </div>
  );
};
export default CoinDetail;

<div className="coinDetailContainer bg-pip-boy-dark-green bg-opacity-20 text-pip-boy-text p-5 rounded-lg max-w-md mx-auto my-5" style={{ boxShadow: '0 0 10px var(--pip-boy-green)' }}>
  <h2 className="text-center mb-4 font-vt323 text-2xl">Coin Details</h2>
  
  <p className="text-base py-1 text-gray-300">
    The latest price for this coin is $50,000.
  </p>
  
  <div className="error text-red-500 font-vt323" style={{ textShadow: '0 0 5px red' }}>
    Error loading coin details.
  </div>

  <div className="loading text-pip-boy-green font-vt323 text-lg" style={{ textShadow: '0 0 5px var(--pip-boy-green)' }}>
    Loading coin details...
  </div>
</div>
