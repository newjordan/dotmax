/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{ts,tsx}"],
  theme: {
    extend: {
      colors: {
        ink: "#050709",
        panel: "#0b1010",
        panel2: "#101616",
        line: "#24312d",
        paper: "#eef7ed",
        muted: "#9fb0a8",
        terminal: "#68f28a",
        cyan: "#5ed8d4",
        amber: "#f2c36b",
        coral: "#f06f5f",
      },
      fontFamily: {
        sans: ["Inter", "ui-sans-serif", "system-ui", "sans-serif"],
        mono: ["JetBrains Mono", "SFMono-Regular", "Consolas", "monospace"],
      },
      boxShadow: {
        terminal: "0 0 0 1px rgba(104, 242, 138, 0.16), 0 18px 80px rgba(0, 0, 0, 0.35)",
      },
    },
  },
  plugins: [],
};
