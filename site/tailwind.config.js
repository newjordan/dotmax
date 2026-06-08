/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{ts,tsx}"],
  theme: {
    extend: {
      colors: {
        page: "#070807",
        ink: "#0b0d0c",
        surface: "#0e110f",
        popover: "#0c0f0d",
        panel: "#101312",
        panel2: "#161a18",
        panel3: "#1c211e",
        line: "#262b28",
        line2: "#333a36",
        paper: "#f4f0e8",
        muted: "#9b988f",
        subtle: "#6f6d66",
        terminal: "#6ee7a0",
        "terminal-bright": "#8af7b6",
        cyan: "#7adbd6",
        amber: "#e2b965",
        coral: "#e8806c",
      },
      fontFamily: {
        sans: ["Inter", "ui-sans-serif", "system-ui", "sans-serif"],
        mono: ["JetBrains Mono", "ui-monospace", "SFMono-Regular", "Consolas", "monospace"],
      },
      borderRadius: {
        lg: "0.625rem",
        xl: "0.875rem",
        "2xl": "1.125rem",
      },
      boxShadow: {
        terminal: "0 0 0 1px rgba(110, 231, 160, 0.12), 0 24px 80px -20px rgba(0, 0, 0, 0.6)",
        card: "0 1px 0 0 rgba(255,255,255,0.03) inset, 0 16px 50px -28px rgba(0,0,0,0.7)",
        glow: "0 0 0 1px rgba(110,231,160,0.35), 0 0 40px -8px rgba(110,231,160,0.35)",
        pop: "0 24px 70px -24px rgba(0,0,0,0.75)",
      },
      keyframes: {
        "fade-in": {
          from: { opacity: "0", transform: "translateY(8px)" },
          to: { opacity: "1", transform: "translateY(0)" },
        },
        "scale-in": {
          from: { opacity: "0", transform: "scale(0.97)" },
          to: { opacity: "1", transform: "scale(1)" },
        },
        shimmer: {
          "100%": { transform: "translateX(100%)" },
        },
      },
      animation: {
        "fade-in": "fade-in 0.5s ease-out both",
        "scale-in": "scale-in 0.16s ease-out both",
      },
    },
  },
  plugins: [],
};
