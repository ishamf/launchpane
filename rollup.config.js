import sucrase from '@rollup/plugin-sucrase';
import resolve from '@rollup/plugin-node-resolve';
import commonjs from '@rollup/plugin-commonjs';

/**
 * @type {import('rollup').RollupOptions}
 */
const baseConfig = {
    external: ['electron', '@prisma/client'],

    plugins: [
        resolve({
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
        ...baseConfig
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