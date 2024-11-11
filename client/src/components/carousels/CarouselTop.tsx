import React from 'react';
import Carousel from './Carousel';
import type { MainNavItem } from '../../types/wasm-types';

interface CarouselTopProps {
    items: MainNavItem[];
    selectedItem: MainNavItem | null;
    onSelect: (item: MainNavItem) => void;
}

const CarouselTop: React.FC<CarouselTopProps> = ({ items, selectedItem, onSelect }) => {
    const carouselItems = items.map((item) => (
        <div
            key={item.id}
            className={`gameboy-panel space-y-2 grid grid-cols-1 place-items-center cursor-pointer transition-transform transform ${
                selectedItem?.id === item.id ? 'scale-105 border-4 border-yellow-500' : ''
            }`}
            onClick={() => onSelect(item)}
            style={{ transition: 'all 0.3s ease-in-out', width: '100px', height: '100px' }} // Set fixed width and height for consistency
        >
            {item.iconUrl && (
                <img
                    src={item.iconUrl}
                    alt={item.title}
                    className="w-40 h-40 mb-4 rounded-full object-cover"
                    loading="lazy"
                />
            )}
            <h2 className="glow-text text-sm text-center">{item.title}</h2> {/* Adjust text size */}
            <p className="text-white text-xs text-center">{item.description}</p> {/* Ensure consistency in description */}
        </div>
    ));

    return (
        <Carousel
            items={carouselItems}
            direction="left"
            autoRotate={false}
            rotationInterval={4000}
            showIndicators={false}
            showNavigation={false}
        />
    );
};

export default CarouselTop;
