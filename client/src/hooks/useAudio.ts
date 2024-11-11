// src/hooks/useAudio.ts
import { useState, useCallback } from 'react';

export const useAudio = () => {
  const [isAudioOn, setIsAudioOn] = useState(false);
  const [volume] = useState(0.5); // You can extend this to allow volume control

  const toggleAudio = useCallback(() => {
    setIsAudioOn(prev => !prev);
  }, []);

  return {
    isAudioOn,
    volume,
    toggleAudio
  };
};
