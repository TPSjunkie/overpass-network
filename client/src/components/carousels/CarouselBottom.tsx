import React from 'react';
import Carousel from './Carousel';

interface CarouselBottomProps {
    items: React.ReactNode[];
    direction?: 'left' | 'right';
    autoRotate?: boolean;
    rotationInterval?: number;
}

const CarouselBottom: React.FC<CarouselBottomProps> = ({
    items,
    direction = 'right',
    autoRotate = true,
    rotationInterval = 5000,
}) => {
    return (
        <Carousel
            items={items.map((item, idx) => (
                <div key={idx} className="grid place-items-center" style={{ width: '100px', height: '100px' }}>
                    {item}
                </div>
            ))}
            direction={direction}
            autoRotate={autoRotate}
            rotationInterval={rotationInterval}
            showIndicators={true}
            showNavigation={true}
            className="mt-4"
        />
    );
};

export default CarouselBottom;
