// src/components/AudioPlayer.tsx
import React, { useEffect, useRef } from 'react';

interface AudioPlayerProps {
  src: string;
  isPlaying: boolean;
  volume: number;
  loop: boolean;
}

const AudioPlayer: React.FC<AudioPlayerProps> = ({ src, isPlaying, volume, loop }) => {
  const audioRef = useRef<HTMLAudioElement>(null);

  useEffect(() => {
    const audio = audioRef.current;
    if (audio) {
      audio.volume = volume;
      audio.loop = loop;
      if (isPlaying) {
        audio.play().catch(error => {
          console.error('Audio playback failed:', error);
        });
      } else {
        audio.pause();
      }
    }
  }, [isPlaying, volume, loop, src]);

  return (
    <audio ref={audioRef} src={src} />
  );
};

export default AudioPlayer;
