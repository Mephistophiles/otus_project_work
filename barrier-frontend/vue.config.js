const { defineConfig } = require("@vue/cli-service");
module.exports = defineConfig({
  transpileDependencies: true,
  configureWebpack: {
    optimization: {
      splitChunks: {
        minSize: 10000,
        maxSize: 250000,
      },
    },
  },
  devServer: {
    proxy: {
      "^/(auth|gates|ping)": {
        target: "http://localhost:7000/",
        ws: true,
        changeOrigin: true,
        logLevel: "debug",
      },
    },
  },
});
