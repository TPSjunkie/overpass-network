// src/pages/Home.tsx

import React from 'react';
import Livepricefeeds from '../components/Livepricefeeds';
import '../styles/Home.css';

const Home: React.FC = () => {
  return (
    <div className="home-container flex flex-col items-center p-8 min-h-screen relative">
      {/* Scanline Overlay */}
      <div className="scanline absolute inset-0 pointer-events-none"></div>

      <h1 className="title text-5xl mb-6 text-center text-pip-boy-green font-vt323 shadow-pip-boy">
        Welcome to Overpass Wallet
      </h1>
      <p className="description text-lg max-w-2xl text-center mb-8 text-pip-boy-text shadow-pip-boy">
        Your gateway to managing TON transactions seamlessly. With Overpass Wallet, you can easily send, receive, and store your TON tokens securely. Our intuitive interface and powerful features make it simple to take control of your digital assets.
      </p>
      <div className="priceCharts w-full max-w-3xl mb-8">
        <Livepricefeeds />
      </div>
      <div className="features flex justify-center gap-6 flex-wrap mb-8">
        <div className="feature bg-pip-boy-panel p-6 rounded-lg w-72 text-center border border-pip-boy-border shadow-pip-boy">
          <h3 className="text-pip-boy-green mb-3 text-2xl shadow-pip-boy">Secure Storage</h3>
          <p className="text-pip-boy-text text-sm">
            Keep your TON tokens safe with our state-of-the-art security measures.
          </p>
        </div>
        <div className="feature bg-pip-boy-panel p-6 rounded-lg w-72 text-center border border-pip-boy-border shadow-pip-boy">
          <h3 className="text-pip-boy-green mb-3 text-2xl shadow-pip-boy">Easy Transactions</h3>
          <p className="text-pip-boy-text text-sm">
            Send and receive TON tokens with just a few clicks.
          </p>
        </div>
        <div className="feature bg-pip-boy-panel p-6 rounded-lg w-72 text-center border border-pip-boy-border shadow-pip-boy">
          <h3 className="text-pip-boy-green mb-3 text-2xl shadow-pip-boy">Intuitive Interface</h3>
          <p className="text-pip-boy-text text-sm">
            Manage your digital assets effortlessly with our user-friendly interface.
          </p>
        </div>
      </div>
      <button
        className="cta bg-pip-boy-green text-black py-3 px-6 rounded-md cursor-pointer transition-colors duration-300 font-vt323 shadow-pip-boy hover:bg-pip-boy-hover-green"
        aria-label="Get Started with Overpass Wallet"
      >
        Get Started
      </button>
    </div>
  );
};

export default Home;
