// src/context/GlobalStateContext.tsx

import React, { createContext, useContext, useState, ReactNode } from 'react';

interface GlobalState {
  isAudioPlaying: boolean;
  toggleAudio: () => void;
}

const GlobalStateContext = createContext<GlobalState | undefined>(undefined);

export const GlobalStateProvider: React.FC<{ children: ReactNode }> = ({ children }) => {
  const [isAudioPlaying, setIsAudioPlaying] = useState(true);

  const toggleAudio = () => {
    setIsAudioPlaying((prev) => !prev);
  };

  return (
    <GlobalStateContext.Provider value={{ isAudioPlaying, toggleAudio }}>
      {children}
    </GlobalStateContext.Provider>
  );
};

export const useGlobalState = (): GlobalState => {
  const context = useContext(GlobalStateContext);
  if (context === undefined) {
    throw new Error('useGlobalState must be used within a GlobalStateProvider');
  }
  return context;
};
