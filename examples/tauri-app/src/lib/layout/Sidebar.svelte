<!-- App navigation: a sidebar on desktop, a bottom tab bar on phones. -->
<script lang="ts">
    import { platform, type Page } from "../state/format";
    import { store } from "../state/device.svelte";
    import { theme, THEMES, type Theme } from "../state/theme.svelte";
    import VitalsLogo from "../ui/VitalsLogo.svelte";

    let { page }: { page: Page } = $props();

    const ITEMS: { id: Page; label: string }[] = [
        { id: "overview", label: "Overview" },
        { id: "adaptive", label: "Adaptive" },
        { id: "power", label: "Power" },
        { id: "system", label: "System" },
    ];

    let activeIndex = $derived(ITEMS.findIndex((i) => i.id === page));

    // Crossfade the whole app between themes where the browser supports it.
    function switchTheme(apply: () => void) {
        const still =
            document.documentElement.dataset.power === "saver" ||
            window.matchMedia("(prefers-reduced-motion: reduce)").matches;
        const doc = document as Document & { startViewTransition?: (cb: () => void) => unknown };
        if (still || !doc.startViewTransition) apply();
        else doc.startViewTransition(apply);
    }
    const pick = (id: Theme) => switchTheme(() => theme.set(id));
</script>

<nav aria-label="Main">
    <div class="brand">
        <VitalsLogo />
        <div class="brand-text">
            <span class="name">Vitals</span>
            <span class="sub">Tauri device-info demo</span>
        </div>
    </div>

    <div class="items">
        <!-- One highlight that springs between items instead of each item lighting up. -->
        <span class="indicator" aria-hidden="true" style:--index={activeIndex}></span>
        {#each ITEMS as item}
            <a href="#/{item.id}" aria-current={page === item.id ? "page" : undefined} class:active={page === item.id}>
                <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                    {#if item.id === "overview"}
                        <rect x="3" y="3" width="7" height="7" rx="1.5" /><rect x="14" y="3" width="7" height="7" rx="1.5" /><rect x="3" y="14" width="7" height="7" rx="1.5" /><rect x="14" y="14" width="7" height="7" rx="1.5" />
                    {:else if item.id === "adaptive"}
                        <path d="M4 18a8 8 0 1 1 16 0" /><path d="m12 18 4-6" />
                    {:else if item.id === "power"}
                        <rect x="2" y="7" width="17" height="10" rx="2" /><path d="M22 11v2" /><path d="m11 9-2 3h3l-2 3" />
                    {:else}
                        <rect x="6" y="6" width="12" height="12" rx="2" /><path d="M9 2v4M15 2v4M9 18v4M15 18v4M2 9h4M2 15h4M18 9h4M18 15h4" />
                    {/if}
                </svg>
                <span>{item.label}</span>
            </a>
        {/each}
    </div>

    <div class="theme">
        <span class="theme-title" id="theme-label">Theme</span>
        <div class="theme-options" role="radiogroup" aria-labelledby="theme-label">
            {#each THEMES as t}
                <button
                    role="radio"
                    aria-checked={theme.current === t.id}
                    class:on={theme.current === t.id}
                    onclick={() => pick(t.id)}
                >
                    <span class="swatch" aria-hidden="true">
                        <span style:background={t.swatch[0]}></span><span style:background={t.swatch[1]}></span>
                    </span>
                    {t.label}
                </button>
            {/each}
        </div>
    </div>

    <!-- Phones: one button that cycles the theme. -->
    <button class="theme-cycle" onclick={() => switchTheme(() => theme.cycle())} aria-label="Theme: {theme.current}. Switch theme">
        <span class="swatch" aria-hidden="true">
            {#each THEMES.find((t) => t.id === theme.current)?.swatch ?? [] as color}
                <span style:background={color}></span>
            {/each}
        </span>
        <span>Theme</span>
    </button>

    <div class="status">
        <div class="status-row">
            {#if store.error}
                <span class="dot off"></span>
            {:else}
                <span class="live-dot"><span class="m-pulse m-loop"></span></span>
            {/if}
            <span class="status-title">{store.error ? "Watchers stopped" : "Watching 5 signals"}</span>
        </div>
        <span class="status-sub">
            {platform === "macOS"
                ? "Native events for battery, display and network; the rest is polled."
                : "Changes are detected by polling and pushed as events."}
        </span>
        <span class="status-platform">{platform}{store.tier?.hardware.cpuModel ? ` · ${store.tier.hardware.cpuModel}` : ""}</span>
    </div>
</nav>

<style>
    nav {
        position: sticky;
        top: 0;
        width: 232px;
        height: 100vh;
        flex-shrink: 0;
        padding: 24px 16px;
        padding-top: calc(env(safe-area-inset-top) + 24px);
        display: flex;
        flex-direction: column;
        gap: 28px;
        background: var(--surface-2);
        border-right: 1px solid var(--border);
        z-index: 10;
    }

    .brand {
        display: flex;
        align-items: center;
        gap: 10px;
        padding: 0 8px;
    }

    .brand-text {
        display: flex;
        flex-direction: column;
    }

    .name {
        font-size: 1.05rem;
        font-weight: 800;
        letter-spacing: -0.01em;
    }

    .sub {
        font-size: 0.7rem;
        color: var(--muted);
    }

    .items {
        position: relative;
        display: flex;
        flex-direction: column;
        gap: 4px;
    }

    .indicator {
        position: absolute;
        left: 0;
        right: 0;
        top: 0;
        height: 44px;
        border-radius: 10px;
        background: var(--nav-active);
        transform: translateY(calc(var(--index) * 48px));
        transition: transform calc(420ms * var(--m)) var(--spring);
    }

    a {
        position: relative;
        display: flex;
        align-items: center;
        gap: 12px;
        min-height: 44px;
        padding: 0 12px;
        border-radius: 10px;
        text-decoration: none;
        font-size: 0.875rem;
        font-weight: 600;
        color: var(--soft);
        transition: background 0.2s, color 0.2s;
    }

    a:hover {
        color: var(--text);
        background: var(--hover);
    }

    a.active {
        color: var(--nav-active-text);
    }

    a.active:hover {
        background: transparent;
    }

    a:focus-visible {
        outline: 2px solid var(--accent);
        outline-offset: 2px;
    }

    .theme {
        margin-top: auto;
        display: flex;
        flex-direction: column;
        gap: 8px;
    }

    .theme-title {
        padding: 0 4px;
        font-size: 0.7rem;
        font-weight: 600;
        letter-spacing: 0.08em;
        text-transform: uppercase;
        color: var(--muted);
    }

    .theme-options {
        display: grid;
        grid-template-columns: repeat(3, minmax(0, 1fr));
        gap: 4px;
        padding: 4px;
        border-radius: 12px;
        background: var(--sunken);
        border: 1px solid var(--border);
    }

    .theme-options button,
    .theme-cycle {
        all: unset;
        box-sizing: border-box;
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 5px;
        min-height: 52px;
        padding: 7px 0;
        border-radius: 9px;
        cursor: pointer;
        font-size: 0.7rem;
        font-weight: 600;
        color: var(--muted);
        transition: background 0.2s, color 0.2s;
    }

    .theme-options button:hover {
        color: var(--text);
    }

    .theme-options button.on {
        color: var(--text);
        background: var(--surface);
        box-shadow: 0 0 0 1.5px var(--accent);
    }

    .theme-options button:focus-visible,
    .theme-cycle:focus-visible {
        outline: 2px solid var(--accent);
        outline-offset: 2px;
    }

    .swatch {
        display: flex;
        width: 22px;
        height: 16px;
        border-radius: 5px;
        overflow: hidden;
        border: 1px solid var(--border);
    }

    .swatch span {
        flex: 1;
    }

    .theme-cycle {
        display: none;
    }

    .status {
        display: flex;
        flex-direction: column;
        gap: 10px;
        padding: 14px;
        border-radius: 12px;
        background: var(--surface);
        border: 1px solid var(--border);
    }

    .status-row {
        display: flex;
        align-items: center;
        gap: 8px;
    }

    .dot.off {
        width: 8px;
        height: 8px;
        border-radius: 50%;
        background: var(--bad);
        box-shadow: none;
    }

    .status-title {
        font-size: 0.8rem;
        font-weight: 600;
    }

    .status-sub {
        font-size: 0.75rem;
        line-height: 1.45;
        color: var(--soft);
    }

    .status-platform {
        font-size: 0.75rem;
        color: var(--muted);
    }

    /* Phones: bottom tab bar. */
    @media (max-width: 820px) {
        nav {
            position: fixed;
            top: auto;
            bottom: 0;
            left: 0;
            right: 0;
            width: 100%;
            height: auto;
            padding: 6px 6px;
            padding-bottom: calc(env(safe-area-inset-bottom) + 6px);
            flex-direction: row;
            gap: 0;
            background: var(--surface-2); /* fallback: WebViews without color-mix() (Chrome < 111) */
            background: color-mix(in srgb, var(--surface-2) 94%, transparent);
            backdrop-filter: blur(16px);
            -webkit-backdrop-filter: blur(16px);
            border-right: 0;
            border-top: 1px solid var(--border);
        }

        .brand,
        .theme,
        .status,
        .indicator {
            display: none;
        }

        .theme-cycle {
            display: flex;
            flex: 1;
            justify-content: center;
            min-height: 48px;
            padding: 4px 0;
            font-size: 0.625rem;
            color: var(--muted);
        }

        .items {
            flex: 4;
            flex-direction: row;
            gap: 0;
        }

        a {
            flex: 1;
            flex-direction: column;
            gap: 3px;
            min-height: 48px;
            padding: 4px 0;
            justify-content: center;
            font-size: 0.625rem;
            color: var(--muted);
        }

        a svg {
            width: 22px;
            height: 22px;
        }

        a.active,
        a:hover {
            color: var(--text);
            background: transparent;
        }
    }
</style>
