// src/pages/About.tsx

import React from 'react';

const About: React.FC = () => {
  return (
    <div className="home-container flex flex-col items-center p-8 min-h-screen relative">
      {/* Scanline Overlay */}
      <div className="scanline"></div>
      <h1 className="title text-4xl mb-4 text-center text-pip-boy-green font-vt323" style={{ textShadow: '0 0 5px var(--pip-boy-green)' }}>
        Welcome to Overpass Wallet  
      </h1>
      <p className="description text-lg max-w-2xl text-center mb-8 text-gray-300">
        Your gateway to managing TON transactions seamlessly. With Overpass Wallet, you can easily send, receive, and store your TON tokens securely. Our intuitive interface and powerful features make it simple to take control of your digital assets.
      </p>
      <div className="features flex justify-center gap-4 flex-wrap mb-8">
        <div className="feature bg-pip-boy-dark-green bg-opacity-20 p-4 rounded-lg w-64 text-center border border-pip-boy-border" style={{ boxShadow: '0 0 10px var(--pip-boy-green)' }}>
          <h3 className="text-pip-boy-green mb-2" style={{ textShadow: '0 0 5px var(--pip-boy-green)' }}>
            Secure Storage
          </h3>
          <p className="text-gray-300 text-sm">Keep your TON tokens safe with our state-of-the-art security measures.</p>
        </div>
        <div className="feature bg-pip-boy-dark-green bg-opacity-20 p-4 rounded-lg w-64 text-center border border-pip-boy-border" style={{ boxShadow: '0 0 10px var(--pip-boy-green)' }}>
          <h3 className="text-pip-boy-green mb-2" style={{ textShadow: '0 0 5px var(--pip-boy-green)' }}>
            Easy Transactions
          </h3>
          <p className="text-gray-300 text-sm">Send and receive TON tokens with just a few clicks.</p>
        </div>
        <div className="feature bg-pip-boy-dark-green bg-opacity-20 p-4 rounded-lg w-64 text-center border border-pip-boy-border" style={{ boxShadow: '0 0 10px var(--pip-boy-green)' }}>
          <h3 className="text-pip-boy-green mb-2" style={{ textShadow: '0 0 5px var(--pip-boy-green)' }}>
            Intuitive Interface
          </h3>
          <p className="text-gray-300 text-sm">Manage your digital assets effortlessly with our user-friendly interface.</p>
        </div>
      </div>
      <button className="cta bg-pip-boy-green text-black py-2 px-4 rounded-md cursor-pointer transition-colors duration-300 font-vt323" style={{ boxShadow: '0 0 5px var(--pip-boy-green)' }} aria-label="Get Started with Overpass Wallet">
        Get Started
      </button>
    </div>
  );
};

export default About;
  