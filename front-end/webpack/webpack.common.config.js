const HtmlWebpackPlugin = require('html-webpack-plugin');
const FaviconsWebpackPlugin = require('favicons-webpack-plugin');

const config = {
    entry: './src/index.js',
    output: {
        clean: true
    },
    module: {
        rules: [
            {
                test: /\.html$/,
                use: [
                    {
                        loader: 'html-loader'
                    }
                ]
            },
            {
                test: /\.raw.css$/,
                use: [ 'style-loader', 'raw-loader', 'css-loader' ]
            },
            {
                test: /\.(woff|woff2|eot|ttf|otf)$/i,
                type: 'asset/resource',
            },
        ]
    },
    plugins: [
        // new FaviconsWebpackPlugin('src/assets/images/red-black-logo.svg'),
        new HtmlWebpackPlugin({
            filename: 'index.html',
            chunks: ['main'],
            template: 'src/index.html',
        })
    ]
};

module.exports = config;