// src/hooks/useGlobalData.ts

import { useEffect, useState } from 'react';
import axios from 'axios';

interface GlobalData {
  market_cap_usd: number;
  volume_24h_usd: number;
  bitcoin_dominance_percentage: number;
  cryptocurrencies_number: number;
  market_cap_ath_value: number;
  market_cap_ath_date: string;
  volume_24h_ath_value: number;
  volume_24h_ath_date: string;
  market_cap_change_24h: number;
  volume_24h_change_24h: number;
  last_updated: number;
}

export const useGlobalData = () => {
  const [globalData, setGlobalData] = useState<GlobalData | null>(null);
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);

  const fetchGlobalData = async () => {
    try {
      const response = await axios.get('https://api.coinpaprika.com/v1/global');
      setGlobalData(response.data);
      setError(null);
    } catch (err) {
      setError('Failed to fetch global market data.');
      console.error(err);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchGlobalData();

    const interval = setInterval(fetchGlobalData, 60000); // Refresh every 60 seconds

    return () => clearInterval(interval);
  }, []);

  return { globalData, loading, error };
};
