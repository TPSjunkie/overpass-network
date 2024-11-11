// src/components/GlobalMarketData.tsx

import React from 'react';
import { useGlobalData } from '../hooks/useGlobalData';
import '../styles/GlobalMarketData.css';

const GlobalMarketData: React.FC = () => {
  const { globalData, loading, error } = useGlobalData();

  if (loading) {
    return <div className="loading text-pip-boy-green font-vt323 text-lg" style={{ textShadow: '0 0 5px var(--pip-boy-green)' }}>Loading global market data...</div>;
  }

  if (error) {
    return <div className="error text-red-500 font-vt323" style={{ textShadow: '0 0 5px red' }}>Error: {error}</div>;
  }

  return (
    <div className="globalDataContainer bg-pip-boy-dark-green bg-opacity-20 text-pip-boy-text p-5 rounded-lg max-w-2xl mx-auto my-5" style={{ boxShadow: '0 0 10px var(--pip-boy-green)' }}>
      <h2 className="text-center mb-4 font-vt323 text-2xl">Global Market Data</h2>
      <ul className="list-none p-0">
        <li className="py-1 text-base">Market Cap (USD): ${globalData?.market_cap_usd?.toLocaleString()}</li>
        <li className="py-1 text-base">24h Volume (USD): ${globalData?.volume_24h_usd?.toLocaleString()}</li>
        <li className="py-1 text-base">Bitcoin Dominance: {globalData?.bitcoin_dominance_percentage}%</li>
        <li className="py-1 text-base">Total Cryptocurrencies: {globalData?.cryptocurrencies_number}</li>
        <li className="py-1 text-base">Market Cap All-Time High: ${globalData?.market_cap_ath_value?.toLocaleString()} on {globalData?.market_cap_ath_date ? new Date(globalData.market_cap_ath_date).toLocaleDateString() : 'N/A'}</li>
        <li className="py-1 text-base">24h Change Market Cap: {globalData?.market_cap_change_24h}%</li>
        <li className="py-1 text-base">24h Change Volume: {globalData?.volume_24h_change_24h}%</li>
        <li className="py-1 text-base">Last Updated: {globalData?.last_updated ? new Date(globalData.last_updated * 1000).toLocaleString() : 'N/A'}</li>
      </ul>
    </div>
  );
};

export default GlobalMarketData;
