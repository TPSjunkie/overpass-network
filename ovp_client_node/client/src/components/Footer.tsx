// src/components/Footer.tsx

import React from 'react';

const Footer: React.FC = () => {
  return (
    <div className="footer text-center mt-4">
      <p className="text-pip-boy-text text-sm">&copy; {new Date().getFullYear()} Overpass Wallet. All rights reserved.</p>
    </div>
  );
};

export default Footer;
