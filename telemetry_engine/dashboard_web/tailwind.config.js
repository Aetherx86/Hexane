// Copyright (c) 2026 jk9576 (Sushi Kim / Jeong Hoon Kim). All rights reserved.
/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./src/pages/**/*.{js,ts,jsx,tsx,mdx}",
    "./src/components/**/*.{js,ts,jsx,tsx,mdx}",
    "./src/app/**/*.{js,ts,jsx,tsx,mdx}",
  ],
  theme: {
    extend: {
      colors: {
        background: "#0b0f19",
        card: "#111827",
        border: "#1f2937",
        accent: "#3b82f6",
      },
    },
  },
  plugins: [],
};
