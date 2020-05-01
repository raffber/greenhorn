const path = require('path')
const HtmlWebpackPlugin = require('html-webpack-plugin');

const browserConfig = {
  entry: './js/app.js',
  output: {
    path: path.resolve(__dirname, "pkg"),
    filename: "app.js",
  },
  plugins: [
    new HtmlWebpackPlugin({
        template: './html/index.html'
    }),
  ],
  mode: "development"
}

const workerConfig = {
  entry: "./js/worker.js",
  output: {
    path: path.resolve(__dirname, "pkg"),
    filename: "worker.js",
  },
  target: "webworker",
  mode: "development",
}

module.exports = [browserConfig, workerConfig]
