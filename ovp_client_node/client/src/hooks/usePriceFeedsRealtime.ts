// src/hooks/usePriceFeedsRealtime.ts

import { useEffect, useState, useCallback, useRef } from "react";
import axios, { AxiosError } from "axios";

interface PriceFeed {
  name: string;
  price: number;
  change: number;
  changePercentage: number;
}

const COINPAPRIKA_API_URL = "https://api.coinpaprika.com/v1/tickers";
const COIN_IDS = ["btc-bitcoin", "eth-ethereum", "usdt-tether", "ton-toncoin"];
const FETCH_INTERVAL = 5000; // 5 seconds

const getPriceFeeds = async (previousFeeds: PriceFeed[]): Promise<PriceFeed[]> => {
  try {
    const requests = COIN_IDS.map((coinId) =>
      axios.get(`${COINPAPRIKA_API_URL}/${coinId}`)
    );

    const responses = await Promise.all(requests);
    const data = responses.map((response) => response.data);

    return data.map((coinData) => {
      const previousFeed = previousFeeds.find((f) => f.name === coinData.name);
      const previousPrice = previousFeed ? previousFeed.price : coinData.quotes.USD.price;

      const change = coinData.quotes.USD.price - previousPrice;
      const changePercentage = previousPrice !== 0 ? (change / previousPrice) * 100 : 0;

      return {
        name: coinData.name,
        price: coinData.quotes.USD.price,
        change: change,
        changePercentage: changePercentage,
      };
    });
  } catch (error) {
    console.error("Error fetching price feeds:", error);
    throw error;
  }
};

export const usePriceFeedsRealtime = () => {
  const [priceFeeds, setPriceFeeds] = useState<PriceFeed[]>([]);
  const [error, setError] = useState<Error | null>(null);
  const previousFeedsRef = useRef<PriceFeed[]>([]);

  const fetchPriceFeeds = useCallback(async () => {
    try {
      const feeds = await getPriceFeeds(previousFeedsRef.current);
      previousFeedsRef.current = feeds;
      setPriceFeeds(feeds);
      setError(null);
    } catch (err) {
      if (axios.isAxiosError(err)) {
        const axiosError = err as AxiosError;
        console.error("AxiosError Details:", {
          message: axiosError.message,
          code: axiosError.code,
          response: axiosError.response?.data,
        });
      } else {
        console.error("Unknown Error:", err);
      }
      setError(err instanceof Error ? err : new Error("An unknown error occurred"));
    }
  }, []);

  useEffect(() => {
    fetchPriceFeeds();

    const interval = setInterval(fetchPriceFeeds, FETCH_INTERVAL);

    return () => clearInterval(interval);
  }, [fetchPriceFeeds]);

  return { priceFeeds, error };
};
