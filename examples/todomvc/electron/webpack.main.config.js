const webpack = require('webpack');
const TerserPlugin = require('terser-webpack-plugin');

module.exports = {
  entry: './js/main.js',
  target: 'electron-main',
  output: {
    filename: 'main.js'
  },
  node: {
    // tell webpack that we actually want a working __dirname value
    // (ref: https://webpack.js.org/configuration/node/#node-__dirname)
    __dirname: false
  },
  module: {
    rules: [
      {
        test: /\.node$/,
        use: 'node-loader'
      }
    ],
  },
  plugins: [new webpack.DefinePlugin({})],
  optimization: {
    minimize: true,
    minimizer: [ new TerserPlugin() ],
  }
};
