import js from "@eslint/js";
import prettier from "eslint-config-prettier";
import globals from "globals";
import vue from "eslint-plugin-vue";

export default [
    {
        ignores: ["dist/**", "node_modules/**"],
    },
    js.configs.recommended,
    ...vue.configs["flat/recommended"],
    prettier,
    {
        files: ["src/**/*.{js,vue}"],
        languageOptions: {
            ecmaVersion: "latest",
            sourceType: "module",
            globals: {
                ...globals.browser,
                ...globals.es2024,
            },
        },
        rules: {
            "no-unused-vars": [
                "warn",
                {
                    argsIgnorePattern: "^_",
                    varsIgnorePattern: "^_",
                },
            ],
            "vue/attributes-order": "off",
            "vue/multi-word-component-names": "off",
            "vue/no-v-html": "off",
        },
    },
];
