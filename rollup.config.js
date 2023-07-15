import sucrase from '@rollup/plugin-sucrase';
import resolvePlugin from '@rollup/plugin-node-resolve';
import commonjs from '@rollup/plugin-commonjs';
import copy from 'rollup-plugin-copy'

import { realpathSync } from 'fs';
import { resolve,dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));

/**
 * @type {import('rollup').RollupOptions}
 */
const baseConfig = {
    external: ['electron'],

    plugins: [
        resolvePlugin({
            extensions: ['.js', '.ts']
        }),
        sucrase({
            exclude: ['node_modules/**'],
            transforms: ['typescript']
        }),
        commonjs()
    ]
}

export default [
    {
        input: 'src/lib/server/main.ts',
        output: {
            file: '.app/build/bundle.js',
            format: 'cjs'
        },
        ...baseConfig,
        plugins: [
            ...baseConfig.plugins,
            copy({
                targets: [
                    { src: 'prisma/schema.prisma', dest: '.app/build' },
                    { src: resolve(realpathSync(resolve(__dirname, 'node_modules/@prisma/client')), '../../.prisma/client/lib*'), dest: '.app/build' },
                ]
            })
        ]
    },
    {

        input: 'src/lib/server/preload.ts',
        output: {
            file: '.app/build/preload.js',
            format: 'cjs'
        },
        ...baseConfig
    },
]