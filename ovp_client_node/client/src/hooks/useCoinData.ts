// src/hooks/useCoinData.ts

import { useEffect, useState } from 'react';
import axios from 'axios';

interface CoinData {
  id: string;
  name: string;
  symbol: string;
  rank: number;
  is_new: boolean;
  is_active: boolean;
  type: string;
  description: string;
  // Add other relevant fields as needed
}

export const useCoinData = (coinId: string) => {
  const [coinData, setCoinData] = useState<CoinData | null>(null);
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);

  const fetchCoinData = async () => {
    try {
      const response = await axios.get(`https://api.coinpaprika.com/v1/coins/${coinId}`);
      setCoinData(response.data);
      setError(null);
    } catch (err) {
      setError(`Failed to fetch data for ${coinId}.`);
      console.error(err);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchCoinData();
  }, [coinId]);

  return { coinData, loading, error };
};
