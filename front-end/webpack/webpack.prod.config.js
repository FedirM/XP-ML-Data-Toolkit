const common = require('./webpack.common.config.js');
const { merge } = require('webpack-merge');
const MiniCssExtractPlugin = require('mini-css-extract-plugin');
const CssMinimizerPlugin = require('css-minimizer-webpack-plugin');
const path = require('path');

module.exports = merge(common, {
    mode: 'production',
    output: {
        path: path.resolve(__dirname, '../dist'),
        filename: 'js/[name].[contenthash:12].js'
    },
    // devtool: 'source-map',
    optimization: {
        minimize: true,
        minimizer: [
            `...`,
            new CssMinimizerPlugin(),
        ],
        splitChunks: {
            chunks: 'all',
            maxSize: Infinity,
            minSize: 0,
            cacheGroups: {
                node_modules: {
                    test: /[\\/]node_modules[\\/]/,
                    name(module) {
                        const packageName = module.context.match(/[\\/]node_modules[\\/](.*?)([\\/]|$)/)[1];
                        return packageName;
                    },
                },
                shared: {
                    test: /[\\/]src[\\/]shared[\\/]js[\\/]/,
                    name(module) {
                        const packageName = module.resource.match(/[\\/]src[\\/]shared[\\/]js[\\/](.*?)([\\/]|$)/)[1].split('.')[0];
                        return packageName;
                    },
                },
                shared_workers: {
                    test: /[\\/]src[\\/]shared[\\/]workers[\\/]/,
                    name(module) {
                        const packageName = module.resource.match(/[\\/]src[\\/]shared[\\/]workers[\\/](.*?)([\\/]|$)/)[1].split('.')[0];
                        return packageName;
                    },
                },
                shared_styles: {
                    test: /[\\/]src[\\/]shared[\\/]styles[\\/]/,
                    type: 'css/mini-extract',
                    name(module) {
                        const packageName = module._identifier.match(/[\\/]src[\\/]shared[\\/]styles[\\/](.*?)([\\/]|$)/)[1].split('.')[0];
                        return packageName;
                    },
                    // chunks: 'all',
                    enforce: true,
                },
            }
        }
    },
    plugins: [
        new MiniCssExtractPlugin( ),
    ],
    module: {
        rules: [
            {
                test: /\.css$/,
                exclude: /\.raw.css$/,
                use: [ MiniCssExtractPlugin.loader, 'css-loader', 'postcss-loader' ]
            },
            // {
            //     test: /\.(png|jpg|svg)$/,
            //     type: 'asset',
            //     parser: {
            //         dataUrlCondition: {
            //             maxSize: 10 * 1024 // 10 kb
            //         }
            //     },
            //     generator: {
            //         filename: './images/[name].[contenthash:12][ext]'
            //     }
            // },
            {
                test: /\.(png|jpg|svg)$/,
                type: 'asset',
                parser: {
                    dataUrlCondition: {
                        maxSize: 10 * 1024 // 10 kb
                    }
                },
                generator: {
                    filename: './images/[name].[contenthash:12][ext]'
                },
                use: [
                    {
                        loader: 'image-webpack-loader',
                        options: {
                            mozjpeg: {
                                quality: 40,
                            },
                            pngquant: {
                                quality: [0.65, 0.90],
                                speed: 4
                            }
                        }
                    }
                ]
            }
        ]
    }
});