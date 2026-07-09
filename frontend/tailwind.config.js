/** @type {import('tailwindcss').Config} */
export default {
  content: ['./index.html', './src/**/*.{js,jsx}'],
  theme: {
    extend: {
      colors: {
        asphalt: {
          DEFAULT: '#15181C',
          soft: '#1D2126',
          line: '#2B3036',
        },
        chalk: {
          DEFAULT: '#EDEFF2',
          dim: '#B7BDC6',
        },
        amber: {
          DEFAULT: '#E8A23D',
          bright: '#F4B860',
          dim: '#8F651F',
        },
        signal: {
          go: '#37B475',
          hold: '#E8A23D',
          stop: '#D6503F',
        },
      },
      fontFamily: {
        display: ['"Space Grotesk"', 'ui-sans-serif', 'system-ui', 'sans-serif'],
        sans: ['"Inter"', 'ui-sans-serif', 'system-ui', 'sans-serif'],
        mono: ['"IBM Plex Mono"', 'ui-monospace', 'SFMono-Regular', 'monospace'],
      },
      letterSpacing: {
        widest2: '0.2em',
      },
      boxShadow: {
        gate: '0 0 0 1px rgba(232,162,61,0.3), 0 10px 28px -10px rgba(0,0,0,0.65)',
      },
    },
  },
  plugins: [],
}
