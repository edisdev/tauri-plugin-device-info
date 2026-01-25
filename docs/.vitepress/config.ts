import { defineConfig } from 'vitepress'

export default defineConfig({
    title: 'Tauri Plugin Device Info',
    description: 'Cross-platform device information for Tauri apps',

    base: '/tauri-plugin-device-info/',

    head: [
        ['link', { rel: 'icon', href: '/favicon.ico' }]
    ],

    themeConfig: {
        logo: '/logo.svg',

        nav: [
            { text: 'Guide', link: '/getting-started' },
            { text: 'API', link: '/api/' },
            { text: 'Platforms', link: '/platforms/' },
            {
                text: 'Links',
                items: [
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
                        { text: 'Display Info', link: '/api/display-info' }
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
            copyright: 'Copyright © 2024 edisdev'
        },

        search: {
            provider: 'local'
        }
    }
})
