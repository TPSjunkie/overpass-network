import React from 'react';

const Header = () => {
  return (
    <div className="flex items-center justify-between">
      <img 
        src={`${process.env.PUBLIC_URL}/assets/4.png`} 
        alt="Overpass Logo" 
        className="h-auto max-h-5 w-20% max-w-full" 
      />
      <h1 className="pipboy-header">Overpass Wallet</h1>
    </div>
  );
};

export default Header;
