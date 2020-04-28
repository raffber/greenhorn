const path = require('path');
const HtmlWebpackPlugin = require('html-webpack-plugin');
const webpack = require('webpack');
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");

module.exports = {
    entry: {
        app: './js/app.js',
        worker: './js/worker.js'
    },
    output: {
        filename: '[name].js',
        path: __dirname + '/pkg'
    },
    mode: 'development'
};
