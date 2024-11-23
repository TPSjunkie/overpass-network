// src/components/AudioControl.tsx
import React from 'react';

interface AudioControlProps {
  isAudioOn: boolean;
  onToggle: () => void;
}

const AudioControl: React.FC<AudioControlProps> = ({ isAudioOn, onToggle }) => {
  return (
    <div className="absolute top-2 right-2 z-10 cursor-pointer">
      <button
        onClick={onToggle}
        className="w-8 h-8 flex items-center justify-center bg-transparent rounded-full transition-transform duration-200 hover:scale-110 active:scale-95"
        aria-label={isAudioOn ? "Turn audio off" : "Turn audio on"}
      >
        <img
          src={isAudioOn ? '/on.png' : '/off.png'}
          alt={isAudioOn ? "Sound On" : "Sound Off"}
          className="w-5 h-5 object-contain"
        />
      </button>
    </div>
  );
};
export default AudioControl;
