import React, { useState, useEffect } from 'react';
import { usePriceFeedsRealtime } from '../hooks/getPriceFeedsRealtime';
import { useGlobalData } from '../hooks/useGlobalData';
import { ArrowUpRight, ArrowDownRight, Radio } from 'lucide-react';

type MarketViewSection = 'prices' | 'global' | 'feed';

const MarketView = () => {
  const { priceFeeds, error: priceError } = usePriceFeedsRealtime();
  const { globalData, loading: globalLoading, error: globalError } = useGlobalData();
  const [selectedSection, setSelectedSection] = useState<MarketViewSection>('prices');
  const [selectedIndex, setSelectedIndex] = useState(0);
  const [priceHistories, setPriceHistories] = useState<{ [key: string]: number[] }>({});

  useEffect(() => {
    if (priceFeeds.length > 0) {
      const updatedHistories = { ...priceHistories };
      priceFeeds.forEach(feed => {
        updatedHistories[feed.name] = [
          ...(priceHistories[feed.name] || []).slice(-19),
          feed.price
        ];
      });
      setPriceHistories(updatedHistories);
    }
  }, [priceFeeds]);

  const formatPrice = (price: number) => price.toLocaleString('en-US', {
    style: 'currency',
    currency: 'USD',
    minimumFractionDigits: 2,
    maximumFractionDigits: 2
  });

  const formatPercent = (value: number) => {
    const formatted = value.toFixed(2);
    return `${value >= 0 ? '+' : ''}${formatted}%`;
  };

  // Custom GameBoy-style sparkline component
  const RetroSparkline = ({ data, positive }: { data: number[], positive: boolean }) => {
    if (!data.length) return null;

    const min = Math.min(...data);
    const max = Math.max(...data);
    const range = max - min;
    const height = 16; // Height in pixels

    return (
      <div className="h-4 flex items-end gap-px">
        {data.map((value, i) => {
          const normalizedHeight = range === 0 
            ? height / 2 
            : ((value - min) / range) * height;
          
          return (
            <div
              key={i}
              className={`w-0.5 transition-all duration-300 ${
                positive ? 'bg-[#0f380f]' : 'bg-[#2f5817]'
              }`}
              style={{ height: `${normalizedHeight}px` }}
            />
          );
        })}
      </div>
    );
  };

  const PriceItem = ({ feed, index, selected }: { feed: any; index: number; selected: boolean }) => (
    <div 
      className={`p-2 ${selected ? 'bg-[#0f380f] text-[#9bbc0f]' : 'border border-[#0f380f]'}`}
    >
      <div className="flex justify-between items-center">
        <span className="font-bold text-xs">{feed.name}</span>
        <div className="text-right">
          <div className="text-xs">{formatPrice(feed.price)}</div>
          <div className={`text-xs flex items-center ${feed.changePercentage >= 0 ? 'text-[#0f380f]' : ''}`}>
            {feed.changePercentage >= 0 ? <ArrowUpRight className="w-3 h-3" /> : <ArrowDownRight className="w-3 h-3" />}
            {formatPercent(feed.changePercentage)}
          </div>
        </div>
      </div>
      <div className="mt-2">
        <RetroSparkline 
          data={priceHistories[feed.name] || [feed.price]} 
          positive={feed.changePercentage >= 0}
        />
      </div>
    </div>
  );

  const GlobalStats = () => (
    <div className="animate-fadeIn space-y-3">
      <div className="text-sm font-bold mb-4">GLOBAL MARKET DATA</div>
      {globalLoading ? (
        <div className="text-center text-xs animate-pulse">Loading...</div>
      ) : globalError ? (
        <div className="text-xs text-center">Failed to load market data</div>
      ) : (
        <div className="space-y-2 text-xs">
          <div className="border border-[#0f380f] p-2">
            <div className="opacity-75">Market Cap</div>
            <div className="font-bold">{formatPrice(globalData?.market_cap_usd || 0)}</div>
          </div>
          <div className="border border-[#0f380f] p-2">
            <div className="opacity-75">24h Volume</div>
            <div className="font-bold">{formatPrice(globalData?.volume_24h_usd || 0)}</div>
          </div>
          <div className="border border-[#0f380f] p-2">
            <div className="opacity-75">BTC Dominance</div>
            <div className="font-bold">{globalData?.bitcoin_dominance_percentage}%</div>
          </div>
          <div className="border border-[#0f380f] p-2">
            <div className="opacity-75">Active Cryptocurrencies</div>
            <div className="font-bold">{globalData?.cryptocurrencies_number}</div>
          </div>
          <div className="border border-[#0f380f] p-2">
            <div className="opacity-75">Market Change 24h</div>
            <div className={`font-bold ${(globalData?.market_cap_change_24h || 0) >= 0 ? 'text-[#0f380f]' : ''}`}>
              {formatPercent(globalData?.market_cap_change_24h || 0)}
            </div>
          </div>
          <div className="text-[0.6rem] mt-4">
            Last Updated: {new Date((globalData?.last_updated || 0) * 1000).toLocaleString()}
          </div>
        </div>
      )}
    </div>
  );

  const LiveFeed = () => (
    <div className="animate-fadeIn">
      <div className="flex justify-between items-center mb-4">
        <div className="text-sm font-bold">LIVE FEED</div>
        <Radio className="w-3 h-3 animate-pulse text-[#0f380f]" />
      </div>
      <div className="space-y-1 text-xs">
        {priceFeeds.map((feed, index) => (
          <div 
            key={index} 
            className={`py-1 border-b border-[#0f380f] last:border-0 ${
              index === selectedIndex ? 'bg-[#0f380f] text-[#9bbc0f]' : ''
            }`}
          >
            <div className="flex justify-between">
              <span>{feed.name}</span>
              <span>{formatPrice(feed.price)}</span>
            </div>
            <div className="flex justify-between text-[0.6rem] opacity-75">
              <span>Change</span>
              <span className={feed.changePercentage >= 0 ? 'text-[#0f380f]' : ''}>
                {formatPercent(feed.changePercentage)}
              </span>
            </div>
          </div>
        ))}
      </div>
    </div>
  );

  if (priceError) {
    return (
      <div className="flex flex-col items-center justify-center h-full">
        <div className="text-xs mb-4">ERROR LOADING MARKET DATA</div>
        <button 
          onClick={() => window.location.reload()}
          className="bg-[#0f380f] text-[#9bbc0f] px-4 py-2 text-xs"
        >
          RETRY
        </button>
      </div>
    );
  }

  return (
    <div className="h-full flex flex-col">
      {/* Header with section selector */}
      <div className="flex justify-between items-center mb-4">
        <div className="text-sm font-bold">MARKET DATA</div>
        <div className="flex gap-2 text-xs">
          {(['prices', 'global', 'feed'] as MarketViewSection[]).map(section => (
            <button
              key={section}
              className={`px-2 py-1 ${
                selectedSection === section ? 'bg-[#0f380f] text-[#9bbc0f]' : ''
              }`}
              onClick={() => {
                setSelectedSection(section);
                setSelectedIndex(0);
              }}
            >
              {section.toUpperCase()}
            </button>
          ))}
        </div>
      </div>

      {/* Main content area */}
      <div className="flex-1 overflow-y-auto">
        {selectedSection === 'prices' && (
          <div className="space-y-2">
            {priceFeeds.map((feed, index) => (
              <PriceItem
                key={feed.name}
                feed={feed}
                index={index}
                selected={index === selectedIndex}
              />
            ))}
          </div>
        )}
        {selectedSection === 'global' && <GlobalStats />}
        {selectedSection === 'feed' && <LiveFeed />}
      </div>

      {/* Navigation help */}
      <div className="text-[0.6rem] mt-4 space-y-1 border-t border-[#0f380f] pt-2">
        <div className="flex justify-between">
          <span>↑↓: NAVIGATE</span>
          <span>A: SELECT</span>
        </div>
        <div className="flex justify-between">
          <span>SELECT: CHANGE VIEW</span>
          <span>B: BACK</span>
        </div>
      </div>
    </div>
  );
};

export default MarketView;