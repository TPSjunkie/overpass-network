// src/components/Livepricefeeds.tsx

import React, { useEffect, useState } from "react";
import { usePriceFeedsRealtime } from "../hooks/getPriceFeedsRealtime";
import HeartbeatChart from "./HeartbeatChart";
import SevenSegmentDisplay from "./SevenSegmentDisplay";


const Livepricefeeds: React.FC = () => {
  const { priceFeeds, error } = usePriceFeedsRealtime();
  
  // Initialize state for multiple coins
  const [priceHistories, setPriceHistories] = useState<{ [key: string]: number[] }>({});
  const [currentPrices, setCurrentPrices] = useState<{ [key: string]: string }>({});
  const [isLoading, setIsLoading] = useState<boolean>(true);

  useEffect(() => {
    if (priceFeeds.length > 0) {
      const updatedPriceHistories: { [key: string]: number[] } = {};
      const updatedCurrentPrices: { [key: string]: string } = {};

      priceFeeds.forEach((feed) => {
        // Update price history (keep last 50 data points)
        updatedPriceHistories[feed.name] = [
          ...(priceHistories[feed.name] || []).slice(-49),
          feed.price,
        ];

        // Update current price
        updatedCurrentPrices[feed.name] = `${feed.price.toFixed(2)}`;
      });

      setPriceHistories(updatedPriceHistories);
      setCurrentPrices(updatedCurrentPrices);
      setIsLoading(false);

      // Optional: Console logs for debugging
      console.log("Updated Price Histories:", updatedPriceHistories);
      console.log("Updated Current Prices:", updatedCurrentPrices);
    }
  }, [priceFeeds]);

  if (error) {
    return (
      <div className='error'>
        <p>Oops! Something went wrong while fetching the price feeds.</p>
        <p>Error: {error.message}</p>
        <button className='retryButton' onClick={() => window.location.reload()}>
          Retry
        </button>
      </div>
    );
  }

  if (isLoading) {
    return <div className='loading'>Loading live price feeds...</div>;
  }

  return (
    <div className='livePriceFeeds'>
      <h2>Live Price Feeds</h2>
      <div className='priceFeedList'>
        {priceFeeds.map((feed) => (
          <div key={feed.name} className='priceFeedItem'>
            <h3>{feed.name}</h3>
            <HeartbeatChart priceFeed={priceHistories[feed.name] || []} />
            <SevenSegmentDisplay value={currentPrices[feed.name] || "$0.00"} />
          </div>
        ))}
      </div>
    </div>
  );
};

export default Livepricefeeds;

const LivePriceFeedsUI: React.FC = () => (
  <div className="livePriceFeeds flex flex-col items-center gap-5">
    <h2 className="text-pip-boy-green font-vt323 mb-2 text-2xl" style={{ textShadow: "0 0 5px var(--pip-boy-green)" }}>
      Live Price Feeds
    </h2>
    <div className="priceFeedList flex gap-5 flex-wrap justify-center">
      <div className="priceFeedItem bg-pip-boy-dark-green bg-opacity-20 p-4 rounded-lg border border-pip-boy-border w-72 text-center" style={{ boxShadow: "0 0 10px var(--pip-boy-green)" }}>
        <h3 className="text-pip-boy-green mb-2 text-xl" style={{ textShadow: "0 0 5px var(--pip-boy-green)" }}>
          Price Feed 1
        </h3>
        <p className="text-gray-300 text-lg">Current Price: $1234</p>
      </div>
      {/* More price feed items */}
    </div>
    <div className="error text-red-500 font-vt323 bg-pip-boy-dark-green bg-opacity-20 p-5 rounded-lg" style={{ textShadow: "0 0 5px red" }}>
      Error loading price feed!
    </div>
    <div className="loading text-pip-boy-green font-vt323 text-lg" style={{ textShadow: "0 0 5px var(--pip-boy-green)" }}>
      Loading...
    </div>
    <button className="retryButton bg-pip-boy-dark-green text-pip-boy-text border border-pip-boy-border px-4 py-2 rounded cursor-pointer transition-colors duration-300 font-vt323 text-sm mt-2 hover:bg-pip-boy-green hover:text-pip-boy-bg">
      Retry
    </button>
  </div>
);
