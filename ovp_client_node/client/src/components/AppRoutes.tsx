// src/components/AppRoutes.tsx
import { lazy, Suspense } from 'react';
import { Routes, Route } from 'react-router-dom';
import LoadingSpinner from '../components/ui/loading-spinner';

// Defining a type that includes a preload method for lazy-loaded components
type PreloadableComponent = React.FC & { preload?: () => void };

// Lazy-loaded pages with preloading
const Home = lazy(() => import('../pages/Home'));
const About = lazy(() => import('../pages/About'));
const Dashboard = lazy(() => import('../pages/Dashboard').then(module => ({ default: module.Dashboard })));

const AppRoutes: React.FC = () => {
  return (
    <Suspense fallback={<LoadingSpinner />}> {/* Replaced the basic loading div with LoadingSpinner component */}
      <Routes>
        <Route path="/" element={<Home />} />
        <Route path="/about" element={<About />} />
        <Route path="/dashboard" element={<Dashboard />} />
      </Routes>
    </Suspense>
  );
};
export default AppRoutes;
// Example pages/Home.tsx with preload method
// This part should be in a separate file: src/pages/Home.tsx
/*
import React from 'react';

const Home: React.FC & { preload?: () => void } = () => {
  return <div>Home Page</div>;
};

// Optionally preload data or perform setup here
Home.preload = () => {
  console.log('Preloading Home page');
};

export default Home;
*/