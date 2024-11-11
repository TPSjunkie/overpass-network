// src/components/TonConnectProvider.tsx
import { TonConnectUIProvider } from '@tonconnect/ui-react';

interface TonConnectProviderProps {
  children: React.ReactNode;
}

const TonConnectProvider: React.FC<TonConnectProviderProps> = ({
  children,
}) => {
  return (    
    <TonConnectUIProvider manifestUrl="https://overpass.network/tonconnect-manifest.json"></TonConnectUIProvider>
    );
};

export default TonConnectProvider;
