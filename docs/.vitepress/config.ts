import { defineConfig } from 'vitepress'

export default defineConfig({
    title: 'tauri-plugin-device-info',
    description: 'Device info for Tauri 2 apps: battery, network, storage, display, power profile and device tier on macOS, iOS, Android, Windows and Linux.',

    base: '/tauri-plugin-device-info/',

    head: [
        // `head` links are not base-prefixed by VitePress.
        ['link', { rel: 'icon', type: 'image/svg+xml', href: '/tauri-plugin-device-info/logo.svg' }],
        ['link', { rel: 'preconnect', href: 'https://fonts.googleapis.com' }],
        ['link', { rel: 'preconnect', href: 'https://fonts.gstatic.com', crossorigin: '' }],
        ['link', { rel: 'stylesheet', href: 'https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700;800&family=JetBrains+Mono:wght@400;500;600&display=swap' }],
        ['meta', { name: 'theme-color', content: '#000000' }]
    ],

    themeConfig: {
        logo: '/logo.svg',

        nav: [
            { text: 'Guide', link: '/getting-started' },
            { text: 'API', link: '/api/' },
            { text: 'Platforms', link: '/platforms/' },
            { text: 'Examples', link: '/examples' },
            {
                text: 'v1.3.0',
                items: [
                    { text: 'Changelog', link: 'https://github.com/edisdev/tauri-plugin-device-info/blob/main/CHANGELOG.md' },
                    { text: 'npm', link: 'https://www.npmjs.com/package/tauri-plugin-device-info-api' },
                    { text: 'crates.io', link: 'https://crates.io/crates/tauri-plugin-device-info' },
                    { text: 'GitHub', link: 'https://github.com/edisdev/tauri-plugin-device-info' }
                ]
            }
        ],

        sidebar: {
            '/': [
                {
                    text: 'Introduction',
                    items: [
                        { text: 'What is this?', link: '/' },
                        { text: 'Getting Started', link: '/getting-started' },
                        { text: 'Examples', link: '/examples' }
                    ]
                },
                {
                    text: 'API Reference',
                    items: [
                        { text: 'Overview', link: '/api/' },
                        { text: 'Device Info', link: '/api/device-info' },
                        { text: 'Battery Info', link: '/api/battery-info' },
                        { text: 'Network Info', link: '/api/network-info' },
                        { text: 'Storage Info', link: '/api/storage-info' },
                        { text: 'Display Info', link: '/api/display-info' },
                        { text: 'Reactive Watch', link: '/api/watch' },
                        { text: 'Adaptive Apps', link: '/api/adaptive' }
                    ]
                },
                {
                    text: 'Platforms',
                    items: [
                        { text: 'Overview', link: '/platforms/' },
                        { text: 'Windows', link: '/platforms/windows' },
                        { text: 'macOS', link: '/platforms/macos' },
                        { text: 'Linux', link: '/platforms/linux' },
                        { text: 'iOS', link: '/platforms/ios' },
                        { text: 'Android', link: '/platforms/android' }
                    ]
                }
            ]
        },

        socialLinks: [
            { icon: 'github', link: 'https://github.com/edisdev/tauri-plugin-device-info' }
        ],

        footer: {
            message: 'Released under the MIT License.',
            copyright: 'Copyright © 2024–2026 edisdev'
        },

        search: {
            provider: 'local'
        }
    }
})
