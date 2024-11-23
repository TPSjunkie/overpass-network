// src/components/TokenomicsOverview.tsx

import React from 'react';

const TokenomicsOverview: React.FC = () => {
  return (
    <div className="bg-gray-800 p-6 rounded-lg shadow-lg">
      <h2 className="text-2xl font-bold text-green-400 mb-4">Overpass Channels Tokenomics</h2>
      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        <div>
          <h3 className="text-xl font-semibold text-green-300 mb-2">Total Supply</h3>
          <p className="text-white">100 billion tokens</p>
        </div>
        <div>
          <h3 className="text-xl font-semibold text-green-300 mb-2">Distribution</h3>
          <ul className="list-disc list-inside text-white">
            <li>70% (70 billion) - Community Airdrop</li>
            <li>20% (20 billion) - Treasury/Governance</li>
            <li>10% (10 billion) - Team, Investors, Advisors</li>
          </ul>
        </div>
      </div>
      <div className="mt-4">
        <h3 className="text-xl font-semibold text-green-300 mb-2">Key Features</h3>
        <ul className="list-disc list-inside text-white">
          <li>Fixed supply model</li>
          <li>Decentralized governance</li>
          <li>Sustainable ecosystem development</li>
          <li>Balanced fee structure</li>
        </ul>
      </div>
    </div>
  );
};

export default TokenomicsOverview;