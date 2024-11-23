import React, { useEffect } from 'react';
import { TonConnectButton } from '@tonconnect/ui-react';

interface LoadScreenProps {
  scanline?: boolean;
  loadingGIF?: boolean;
  showConnectButton?: boolean;
  isPlaying?: boolean;
  volume?: number;
  loop?: boolean;
  URL?: string;
}

const LoadScreen: React.FC<LoadScreenProps> = ({ showConnectButton = false, isPlaying = false, volume = 1, loop = false, URL = '' }) => {
  useEffect(() => {
    if (URL) {
      const audio = new Audio(URL);
      audio.volume = volume;
      audio.loop = loop;
      
      if (isPlaying) {
        audio.play().catch(error => {
          console.error('Failed to play audio:', error);
        });
      }

      return () => {
        audio.pause();
        audio.currentTime = 0;
      };
    }
  }, [isPlaying, URL, volume, loop]);

  return (
    <>
      <div className="scanline absolute inset-0 pointer-events-none"></div>
      <div className="load-screen">
        <div className="load-screen-content">
          <div className="scanline"></div>
          <img
            src="/assets/9.png"
            alt="Overpass Logo"
            className="op-name"
          />
          {showConnectButton ? (
            <div className="scanline">
              <TonConnectButton />
            </div>
          ) : (
            <div className="loading-container">
              <img src="/assets/loadingOPlogo.GIF" alt="Loading..." className="loading-gif" />
              <p className="loading-text">LOADING...</p>
            </div>
          )}
        </div>
      </div>
      <style>{`
        .scanline {
          position: fixed;
          top: 0;
          left: 0;
          width: 100%;
          height: 100%;
          background: linear-gradient(
            to bottom,
            rgba(255, 255, 255, 0),
            rgba(255, 255, 255, 0) 20%,
            rgba(0, 0, 0, 0.2) 20%,
            rgba(0, 0, 0, 0.2)
          );
          background-size: 100% 6px;
          z-index: 2;
          pointer-events: none;
          opacity: 0.9;
          animation: scanlines 74s linear infinite;  
        }
        .glow-text {
          text-shadow: 0 0 5px var(--pip-boy-green);
        }
        @keyframes scanlines {
          0% {
            background-position: 0 0;
          }
          100% {
            background-position: 0 100%;
          }
        }
        
        .scanlines::before {
          content: "";
          position: absolute;
          top: 0;
          left: 0;
          right: 0;
          bottom: 0;
          background: linear-gradient(
            to bottom,
            transparent 50%,
            rgba(0, 0, 0, 0.1) 51%
          );
          background-size: 100% 4px;
          animation: scanlines 22s linear infinite;
          pointer-events: none;
        }
        @keyframes scanlines {
          0% {
            background-position: 0 0;
          }
          100% {
            background-position: 0 100%;
          }
        }
      
        .pip-boy-header {
          font-size: 2.5em;
          margin: 0;
          text-shadow: 0 0 5px #a7c957;
        }
      `}</style>
    </>
  );
};

export default LoadScreen;