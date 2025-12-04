/** @type {import('tailwindcss').Config} */
export default {
    content: [
        "./index.html",
        "./src/**/*.{svelte,js,ts,jsx,tsx}",
    ],
    theme: {
        extend: {
            colors: {
                'hpc-dark': '#0a0f1c',
                'hpc-blue': '#1a2332',
                'hpc-cyan': '#00f2ff',
                'hpc-green': '#00ff9d',
                'hpc-red': '#ff0055',
            },
            fontFamily: {
                mono: ['JetBrains Mono', 'monospace'],
                sans: ['Inter', 'sans-serif'],
            }
        },
    },
    plugins: [],
}
