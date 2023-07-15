import sucrase from '@rollup/plugin-sucrase';
import resolvePlugin from '@rollup/plugin-node-resolve';
import commonjs from '@rollup/plugin-commonjs';
import json from '@rollup/plugin-json';
import copy from 'rollup-plugin-copy'

import { realpathSync } from 'fs';
import { resolve, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));

/**
 * @type {import('rollup').RollupOptions}
 */
const baseConfig = {
    external: ['electron'],

    plugins: [
        resolvePlugin({
            extensions: ['.js', '.ts', '.json']
        }),
        sucrase({
            exclude: ['node_modules/**'],
            transforms: ['typescript']
        }),
        commonjs(),
        json(),
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
                    {
                        src: [
                            'node_modules/.pnpm/@prisma*@4*/**/*.wasm',
                        ], dest: '.app/build'
                    },
                    {
                        src: [
                            'node_modules/@prisma/engines/lib*',
                            'node_modules/@prisma/engines/migration*',
                        ], dest: '.app/'
                    },
                    {
                        src: [
                            'prisma/schema.prisma',
                            'prisma/migrations',
                        ], dest: '.app/prisma'
                    },

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