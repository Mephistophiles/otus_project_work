module.exports = {
  purge: ["./index.html", "./src/**/*.{vue,js,ts,jsx,tsx}"],
  darkMode: false, // or 'media' or 'class'
  theme: {
    extend: {
      colors: {
        "dlink-main": "#0087A9",
        "dlink-hover": "#007B9A",
      },
    },
  },
  variants: {
    extend: {},
  },
  plugins: [],
};
