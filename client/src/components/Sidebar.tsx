// src/components/Sidebar.tsx

import React from 'react';
import Header from './Header';
import Footer from './Footer';
import AppRoutes from './AppRoutes';

const Sidebar: React.FC = () => {
  return (
    <aside className="pipboy-sidebar w-64 bg-pip-boy-dark-green p-4 flex flex-col justify-between">
      <div>
        <Header />
        <nav className="mt-8">
          <AppRoutes />
        </nav>
      </div>
      <Footer />
    </aside>
  );
};

export default Sidebar;