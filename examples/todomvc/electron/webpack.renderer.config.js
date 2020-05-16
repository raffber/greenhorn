const HtmlWebpackPlugin = require('html-webpack-plugin');
const webpack = require('webpack');
const TerserPlugin = require('terser-webpack-plugin');

module.exports = {
  output: {
    filename: "renderer.js"
  },
  entry: './js/renderer.js',
  target: "electron-renderer",
  module: {
    rules: [
      {
        test: /\.s?[ac]ss$/i,
        use: [
          'style-loader',
          'css-loader',
          'sass-loader',
        ],
      }
    ]
  },
  plugins: [
    new HtmlWebpackPlugin({
      template: './html/index.html',
    }),
    new webpack.DefinePlugin({})
  ],
  optimization: {
    minimize: true,
    minimizer: [ new TerserPlugin() ],
  }
};
