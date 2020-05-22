module.exports = {
    entry: './js/web_view.js',
    module: {
      rules: [
        {
          test: /\.(js)$/,
          exclude: /node_modules/,
          use: ['babel-loader']
        }
      ]
    },
    resolve: {
      extensions: ['*', '.js']
    },
    output: {
      path: __dirname + '/dist',
      publicPath: '/',
      filename: 'bundle.js',
      libraryTarget: 'var',
      library: 'greenhorn'
    },
    devServer: {
      contentBase: './dist'
    }
  };