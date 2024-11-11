// src/components/SevenSegmentDisplay.tsx

import React from 'react';
import '../styles/SevenSegmentDisplay.css';

interface SevenSegmentDisplayProps {
  value: string;
}

const SevenSegmentDisplay: React.FC<SevenSegmentDisplayProps> = ({ value }) => {
  return (
    <div className='sevenSegment'>
      {value}
    </div>
  );
};

export default SevenSegmentDisplay;
