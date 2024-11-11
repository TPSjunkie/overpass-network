// src/components/carousels/Carousel.tsx

import React, { useState, useEffect, useRef } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { useSwipeable } from 'react-swipeable';

interface CarouselProps {
    items: React.ReactNode[];
    direction?: 'left' | 'right';
    autoRotate?: boolean;
    rotationInterval?: number; // in milliseconds
    onSelect?: (index: number) => void; // Optional callback for item selection
    showIndicators?: boolean; // Whether to show rotation indicators
    showNavigation?: boolean; // Whether to show navigation buttons
    className?: string; // Additional classes for customization
}

const Carousel: React.FC<CarouselProps> = ({
    items,
    direction = 'left',
    autoRotate = true,
    rotationInterval = 3000,
    onSelect,
    showIndicators = true,
    showNavigation = true,
    className = '',
}) => {
    const [currentIndex, setCurrentIndex] = useState(0);
    const timeoutRef = useRef<NodeJS.Timeout | null>(null);

    // Reset the auto-rotation timeout
    const resetTimeout = () => {
        if (timeoutRef.current) {
            clearTimeout(timeoutRef.current);
        }
    };

    // Navigate to the next slide
    const handleNext = () => {
        setCurrentIndex((prevIndex) => (prevIndex === items.length - 1 ? 0 : prevIndex + 1));
    };

    // Navigate to the previous slide
    const handlePrev = () => {
        setCurrentIndex((prevIndex) => (prevIndex === 0 ? items.length - 1 : prevIndex - 1));
    };

    // Handle auto-rotation
    useEffect(() => {
        if (autoRotate) {
            resetTimeout();
            timeoutRef.current = setTimeout(() => {
                direction === 'left' ? handleNext() : handlePrev();
            }, rotationInterval);

            return () => {
                resetTimeout();
            };
        }
        return undefined;
    }, [currentIndex, autoRotate, rotationInterval, direction, items.length]);

    // Swipe handlers
    const swipeHandlers = useSwipeable({
        onSwipedLeft: () => handleNext(),
        onSwipedRight: () => handlePrev(),
        preventScrollOnSwipe: true,
        trackMouse: true,
    });

    // Keyboard navigation
    const handleKeyDown = (e: KeyboardEvent) => {
        if (e.key === 'ArrowLeft') {
            handlePrev();
        } else if (e.key === 'ArrowRight') {
            handleNext();
        }
    };

    useEffect(() => {
        window.addEventListener('keydown', handleKeyDown);
        return () => window.removeEventListener('keydown', handleKeyDown);
    }, []);

    // Animation variants
    const variants = {
        enter: (custom: any) => ({
            opacity: 0,
            x: custom.direction === 'left' ? 300 : -300,
        }),
        center: {
            zIndex: 1,
            opacity: 1,
            x: 0,
        },
        exit: (custom: any) => ({
            zIndex: 0,
            opacity: 0,
            x: custom.direction === 'left' ? -300 : 300,
        }),
    };

    return (
        <div
            className={`relative w-full h-full ${className}`}
            {...swipeHandlers}
            role="region"
            aria-roledescription="carousel"
            aria-label="Carousel"
        >
            <AnimatePresence initial={false} custom={{ direction }}>
                <motion.div
                    key={currentIndex}
                    custom={{ direction }}
                    variants={variants}
                    initial="enter"
                    animate="center"
                    exit="exit"
                    transition={{
                        type: 'tween',
                        ease: 'easeInOut',
                        duration: 0.5,
                    }}
                    className="absolute w-full h-full flex items-center justify-center"
                    role="group"
                    aria-roledescription="slide"
                    aria-label={`Slide ${currentIndex + 1} of ${items.length}`}
                    onClick={() => onSelect && onSelect(currentIndex)}
                >
                    {items[currentIndex]}
                </motion.div>
            </AnimatePresence>

            {/* Navigation Buttons */}
            {showNavigation && (
                <>
                    <button
                        className="absolute top-1/2 left-2 transform -translate-y-1/2 bg-gameboy-darkest bg-opacity-70 text-white p-2 rounded-full shadow-lg focus:outline-none"
                        onClick={handlePrev}
                        aria-label="Previous Slide"
                    >
                        ←
                    </button>
                    <button
                        className="absolute top-1/2 right-2 transform -translate-y-1/2 bg-gameboy-darkest bg-opacity-70 text-white p-2 rounded-full shadow-lg focus:outline-none"
                        onClick={handleNext}
                        aria-label="Next Slide"
                    >
                        →
                    </button>
                </>
            )}

            {/* Rotation Indicators */}
            {showIndicators && (
                <div
                    className="absolute bottom-4 left-1/2 transform -translate-x-1/2 flex space-x-2"
                    role="tablist"
                    aria-label="Slide Indicators"
                >
                    {items.map((_, index) => (
                        <button
                            key={index}
                            className={`w-3 h-3 rounded-full focus:outline-none ${
                                currentIndex === index ? 'bg-white' : 'bg-gray-400'
                            }`}
                            onClick={() => setCurrentIndex(index)}
                            aria-label={`Go to slide ${index + 1}`}
                            role="tab"
                            aria-selected={currentIndex === index}
                        ></button>
                    ))}
                </div>
            )}
        </div>
    );
};

export default Carousel;
