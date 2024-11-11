export const content = [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
];
export const darkMode = 'class';
export const theme = {
    extend: {
        colors: {
            'gameboy-light': '#9bbc0f',
            'gameboy-dark': '#306230',
            'gameboy-darkest': '#0f380f',
            'gameboy-bg': '#8bac0f',
            'military-brown': '#4a3c28',
        },
        fontFamily: {
            vt323: ['"VT323"', 'monospace'],
        },
        boxShadow: {
            'gameboy': '0 0 20px rgba(0, 0, 0, 0.5), inset 0 0 10px rgba(0, 0, 0, 0.5)',
            'gameboy-screen': 'inset 0 0 10px rgba(0, 0, 0, 0.3)',
        },
        backgroundImage: {
            "gameboy-light": "linear-gradient(to bottom, #9bbc0f, #8bac0f)",
            "gameboy-dark": "linear-gradient(to bottom, #306230, #0f380f)",
            "gameboy-darkest": "linear-gradient(to bottom, #0f380f, #000000)",
            "gameboy-bg": "linear-gradient(to bottom, #8bac0f, #9bbc0f)",
            "military-brown": "linear-gradient(to bottom, #4a3c28, #302418)",
            "military-brown-dark": "linear-gradient(to bottom, #4a3c28, #302418)",
        },
        animation: {
            scanlines: 'scanlines 10s linear infinite',
        },
        animation: {
            scanlines: 'scanlines 6.0s linear infinite',
        },
        keyframes: {
            scanlines: {
                'from': { transform: 'translateY(0)' },
                'to': { transform: 'translateY(4px)' },
            },
        },
    },
};
export const plugins = [];
  

  
 