const common = require('./webpack.common.config.js');
const { merge } = require('webpack-merge');
const MiniCssExtractPlugin = require('mini-css-extract-plugin');
const path = require('path');

module.exports = merge(common, {
    mode: 'development',
    output: {
        filename: '[name].bundle.js',
        path: path.resolve(__dirname, '../dist_dev')
    },
    devtool: 'eval-source-map',
    devServer: {
        port: 9000,
        open: ['/'],
        static: {
            directory: path.resolve(__dirname, '../dist_dev'),
        },
        devMiddleware: {
            index: 'index.html',
            mimeTypes: { html: 'text/html' },
            // writeToDisk: true,
        },
        client: {
            overlay: true
        },
        liveReload: false,
    },
    plugins: [
        new MiniCssExtractPlugin(),
    ],
    module: {
        rules: [
            {
                test: /\.css$/,
                exclude: /\.raw.css$/,
                use: [ MiniCssExtractPlugin.loader, 'css-loader', 'postcss-loader' ]
            },
            {
                test: /\.(png|jpg|svg)$/,
                type: 'asset',
                parser: {
                    dataUrlCondition: {
                        maxSize: 10 * 1024 // 10 kb
                    }
                },
                generator: {
                    filename: './images/[name][ext]'
                }
            }
        ]
    }
});