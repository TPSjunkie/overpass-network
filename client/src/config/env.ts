// src/config/env.ts
export const getEnvConfig = () => ({
    appTitle: import.meta.env.VITE_APP_TITLE || 'TON Game Boy',
    tonNetwork: import.meta.env.VITE_TON_NETWORK || 'mainnet',
    apiEndpoint: import.meta.env.VITE_API_ENDPOINT || 'https://ton.access.orbs.network',
    isDevelopment: import.meta.env.DEV,
    isProduction: import.meta.env.PROD,
    mode: import.meta.env.MODE,
  });
  
  // Type guard for runtime environment checking
  export const isValidNetwork = (network: unknown): network is 'mainnet' | 'testnet' => {
    return typeof network === 'string' && ['mainnet', 'testnet'].includes(network);
  };