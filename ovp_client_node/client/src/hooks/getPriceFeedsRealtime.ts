// src/hooks/getPriceFeedsRealtime.ts

import { useEffect, useState, useCallback, useRef } from "react";
import axios, { AxiosError } from "axios";

interface PriceFeed {
  name: string;
  price: number;
  change: number;
  changePercentage: number;
}

const COINGECKO_API_URL = "https://api.coingecko.com/api/v3/simple/price";
const COIN_IDS = "bitcoin,ethereum,tether,the-open-network"; // Verify if 'toncoin' is needed
const VS_CURRENCY = "usd";
const FETCH_INTERVAL = 75000;

const getPriceFeeds = async (previousFeeds: PriceFeed[]): Promise<PriceFeed[]> => {
  try {
    const response = await axios.get(
      `${COINGECKO_API_URL}?ids=${COIN_IDS}&vs_currencies=${VS_CURRENCY}`
    );
    const data = response.data;

    return [
      {
        name: "Bitcoin",
        price: data.bitcoin?.usd ?? 0,
        change: data.bitcoin?.usd - (previousFeeds.find(f => f.name === "Bitcoin")?.price || data.bitcoin?.usd || 0),
        changePercentage: previousFeeds.find(f => f.name === "Bitcoin")?.price
          ? ((data.bitcoin.usd - previousFeeds.find(f => f.name === "Bitcoin")!.price) / previousFeeds.find(f => f.name === "Bitcoin")!.price) * 100
          : 0,
      },
      {
        name: "Ethereum",
        price: data.ethereum?.usd ?? 0,
        change: data.ethereum?.usd - (previousFeeds.find(f => f.name === "Ethereum")?.price || data.ethereum?.usd || 0),
        changePercentage: previousFeeds.find(f => f.name === "Ethereum")?.price
          ? ((data.ethereum.usd - previousFeeds.find(f => f.name === "Ethereum")!.price) / previousFeeds.find(f => f.name === "Ethereum")!.price) * 100
          : 0,
      },
      {
        name: "USDT",
        price: data.tether?.usd ?? 0,
        change: data.tether?.usd - (previousFeeds.find(f => f.name === "USDT")?.price || data.tether?.usd || 0),
        changePercentage: previousFeeds.find(f => f.name === "USDT")?.price
          ? ((data.tether.usd - previousFeeds.find(f => f.name === "USDT")!.price) / previousFeeds.find(f => f.name === "USDT")!.price) * 100
          : 0,
      },
      {
        name: "TON",
        price: data["the-open-network"]?.usd ?? 0,
        change: data["the-open-network"]?.usd - (previousFeeds.find(f => f.name === "TON")?.price || data["the-open-network"]?.usd || 0),
        changePercentage: previousFeeds.find(f => f.name === "TON")?.price
          ? ((data["the-open-network"].usd - previousFeeds.find(f => f.name === "TON")!.price) / previousFeeds.find(f => f.name === "TON")!.price) * 100
          : 0,
      },
    ];
  } catch (error) {
    console.error("Error in getPriceFeeds:", error);
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