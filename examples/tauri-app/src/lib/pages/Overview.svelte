<script lang="ts">
    import { onMount } from "svelte";
    import { store } from "../state/device.svelte";
    import { capitalize, formatGb, formatHealth, formatMemory, formatTime, gb, maskIp, networkLabel, platform, storageLabel } from "../state/format";
    import { REASON_LABELS } from "../adaptive/types";
    import TierGauge from "../adaptive/TierGauge.svelte";
    import CountUp from "../ui/CountUp.svelte";

    // Rings start empty and sweep to their value once the page is on screen.
    let ready = $state(false);
    // Events older than this page visit fade in with the page; newer ones slide in.
    const openedAt = Date.now();
    onMount(() => {
        const id = requestAnimationFrame(() => (ready = true));
        return () => cancelAnimationFrame(id);
    });

    const PROFILE_HINT = {
        full: "No constraints — running at full quality.",
        balanced: "Some pressure — trimming non-essential work.",
        saver: "Heavy pressure — effects off, doing the minimum.",
    };
    const PROFILES = ["full", "balanced", "saver"] as const;

    let profile = $derived(store.effectiveProfile);
    let profileIndex = $derived(PROFILES.indexOf(profile));
    let usedPct = $derived(
        store.storage?.totalSpace
            ? Math.round((1 - (store.storage.freeSpace ?? 0) / store.storage.totalSpace) * 100)
            : 0,
    );
    // Draw the display at its real aspect ratio, fitted inside a 72 × 48 box
    // (a portrait phone screen stays small instead of stretching the tile).
    let screen = $derived.by(() => {
        const w = store.display?.width ?? 3;
        const h = store.display?.height ?? 2;
        const k = Math.min(72 / w, 48 / h);
        return { w: Math.round(w * k), h: Math.round(h * k) };
    });
    // On phones the model and the CPU name are the same identifier ("iPhone16,1").
    let subtitle = $derived(
        [...new Set([store.device?.model, store.tier?.hardware.cpuModel, platform].filter(Boolean))].join(" · "),
    );
</script>

<div class="page">
    <header class="page-header">
        <div>
            <h1>Overview</h1>
            <p>
                {subtitle}
            </p>
        </div>
        <span class="live-pill"><span class="live-dot"><span class="m-pulse m-loop"></span></span>Live — updates on change</span>
    </header>

    <!-- Adaptive summary -->
    <div class="row-2">
        <a class="card summary m-card" style:--i="0" href="#/adaptive" data-profile={profile} style:background="var(--hero-power-bg)">
            <div class="summary-main">
                <span class="label">Power profile</span>
                <span class="big profile-color">{capitalize(profile)}</span>
                <span class="soft small">{PROFILE_HINT[profile]}</span>
            </div>
            <div class="summary-side">
                {#key profile}
                    {#if store.simulate !== "auto"}
                        <span class="chip sim m-pop" style:--d="380ms">Simulated</span>
                    {:else if store.power && store.power.reasons.length > 0}
                        <span class="chip bad m-pop" style:--d="380ms">{REASON_LABELS[store.power.reasons[0]]}</span>
                    {:else}
                        <span class="chip ok m-pop" style:--d="380ms">All clear</span>
                    {/if}
                {/key}
                <div class="mini-scale" aria-hidden="true">
                    {#each PROFILES as p, i}
                        {#if i > 0}<span class="mini-rail" class:filled={i <= profileIndex}></span>{/if}
                        <span class="mini-dot m-pop" style:--d="{300 + i * 80}ms" class:on={i === profileIndex} class:passed={i < profileIndex}></span>
                    {/each}
                </div>
                <span class="link">Open Adaptive →</span>
            </div>
        </a>

        <a class="card summary tier m-card" style:--i="1" href="#/adaptive" style:background="var(--hero-tier-bg)">
            {#if store.tier}
                <TierGauge score={store.tier.score} tier={store.tier.tier} size={96} zones={false} />
                <div class="summary-main">
                    <span class="label">Device tier</span>
                    <span class="big tier-color" data-tier={store.tier.tier}>{capitalize(store.tier.tier)}</span>
                    <span class="soft small">
                        {[
                            store.tier.hardware.cpuModel,
                            store.tier.hardware.cpuCores && `${store.tier.hardware.cpuCores} cores`,
                            store.tier.hardware.totalMemory && formatMemory(store.tier.hardware.totalMemory),
                        ]
                            .filter(Boolean)
                            .join(" · ")}
                    </span>
                </div>
            {:else}
                <span class="soft">Scoring hardware…</span>
            {/if}
        </a>
    </div>

    <!-- Readings -->
    <div class="row-4">
        <a class="card tile m-card" style:--i="2" href="#/power" data-tile="battery">
            <span class="label">Battery</span>
            {#if store.battery?.level !== undefined && store.battery?.level !== null}
                <!-- Counts up on arrival; each later change rolls in. -->
                <span class="big"><CountUp value={Math.round(store.battery.level)} />%</span>
                <div class="bar"><div class="bar-fill m-growx" style:--d="250ms" style:width="{store.battery.level}%"></div></div>
                <span class="soft small">
                    {store.battery.isCharging ? "Charging" : "Discharging"}{store.battery.health ? ` · Health ${formatHealth(store.battery.health)}` : ""}
                </span>
            {:else}
                <span class="big">Mains</span>
                <span class="soft small">No battery reported</span>
            {/if}
        </a>

        <a class="card tile m-card" style:--i="3" href="#/system" data-tile="network">
            <span class="label">Network</span>
            <div class="inline">
                <!-- Arcs light up from the center out, like a signal arriving. -->
                <svg width="26" height="26" viewBox="0 0 24 24" fill="none" stroke="var(--tile-accent)" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path class="m-light" style:--i="0" d="M12 19.5h.01" /><path class="m-light" style:--i="1" d="M8.5 16a5 5 0 0 1 7 0" /><path class="m-light" style:--i="2" d="M5 12.5a10 10 0 0 1 14 0" /><path class="m-light" style:--i="3" d="M2 9a15 15 0 0 1 20 0" /></svg>
                <span class="big">{networkLabel(store.network?.networkType)}</span>
            </div>
            <span class="mono soft small">{store.reveal ? (store.network?.ipAddress ?? "—") : maskIp(store.network?.ipAddress)}</span>
        </a>

        <a class="card tile m-card" style:--i="4" href="#/system" data-tile="storage">
            <span class="label">Storage</span>
            <div class="inline">
                <svg width="56" height="56" viewBox="0 0 80 80" role="img" aria-label="{usedPct}% used">
                    <circle cx="40" cy="40" r="34" fill="none" stroke="var(--track)" stroke-width="10" />
                    <circle class="sweep" cx="40" cy="40" r="34" fill="none" stroke="var(--tile-accent)" stroke-width="10" stroke-dasharray="{((ready ? usedPct : 0) / 100) * 213.6} 213.6" transform="rotate(-90 40 40)" />
                </svg>
                <div class="stack">
                    <span class="medium">
                        {#if gb(store.storage?.freeSpace) < 1000}<CountUp value={Math.round(gb(store.storage?.freeSpace))} /> GB{:else}{formatGb(store.storage?.freeSpace)}{/if}
                    </span>
                    <span class="muted small">free of {formatGb(store.storage?.totalSpace)}</span>
                </div>
            </div>
            <span class="soft small">{storageLabel(store.storage?.storageType)} · {usedPct}% used</span>
        </a>

        <a class="card tile m-card" style:--i="5" href="#/system" data-tile="display">
            <span class="label">Display</span>
            <div class="inline">
                <div class="screen m-reveal" style:width="{screen.w}px" style:height="{screen.h}px" aria-hidden="true"></div>
                <div class="stack">
                    <span class="medium">{#if store.display?.refreshRate}<CountUp value={Math.round(store.display.refreshRate)} /> Hz{:else}—{/if}</span>
                    <span class="muted small">@{store.display?.scaleFactor ?? 1}x scale</span>
                </div>
            </div>
            <span class="soft small">{store.display ? `${store.display.width} × ${store.display.height} px` : "—"}</span>
        </a>
    </div>

    <!-- Live events -->
    <section class="card events m-card" style:--i="6" aria-label="Live events">
        <div class="events-head">
            <span class="label">Live events</span>
            <span class="muted small">Pushed by <span class="mono">watch*()</span> — no polling in your code</span>
        </div>
        {#if store.events.length === 0}
            <p class="empty">
                No changes yet. Unplug the charger, switch networks or connect a display — each change shows up here
                the moment the plugin reports it.
            </p>
        {:else}
            <ul>
                {#each store.events.slice(0, 6) as event, i (event.id)}
                    <li class={event.at > openedAt ? "m-enter" : "m-fadeup"} style:--d="500ms" style:--i={i}>
                        <span class="time mono muted small">{formatTime(event.at)}</span>
                        <span class="api mono" data-kind={event.kind}>{event.api}</span>
                        <span class="text">{event.text}</span>
                    </li>
                {/each}
            </ul>
        {/if}
    </section>
</div>

<style>
    .live-pill {
        display: flex;
        align-items: center;
        gap: 8px;
        padding: 6px 12px;
        border-radius: 999px;
        background: var(--good-bg);
        border: 1px solid var(--good-border);
        font-size: 0.75rem;
        font-weight: 600;
        color: var(--good-text);
    }

    .row-2 {
        display: grid;
        grid-template-columns: repeat(2, minmax(0, 1fr));
        gap: 16px;
    }

    .row-4 {
        display: grid;
        grid-template-columns: repeat(4, minmax(0, 1fr));
        gap: 16px;
    }

    @media (max-width: 1100px) {
        .row-4 {
            grid-template-columns: repeat(2, minmax(0, 1fr));
        }
    }

    @media (max-width: 640px) {
        .row-2 {
            grid-template-columns: 1fr;
        }

        /* Phone: profile text left, chip + scale right, on one row. */
        .card.summary {
            flex-wrap: nowrap;
            gap: 12px;
        }

        .summary .link {
            display: none;
        }

        .big {
            font-size: 1.625rem;
        }

        .row-4 {
            gap: 12px;
        }

        /* Two-up tiles are ~165 px wide: drop the drawings, keep the numbers. */
        .row-4 .inline > svg,
        .row-4 .screen {
            display: none;
        }
    }

    a.card {
        text-decoration: none;
        color: inherit;
        transition: border-color 0.2s;
    }

    a.card:hover {
        border-color: var(--accent-line); /* fallback: WebViews without color-mix() (Chrome < 111) */
        border-color: color-mix(in srgb, var(--accent-line) 40%, transparent);
    }

    a.card:focus-visible {
        outline: 2px solid var(--accent);
        outline-offset: 2px;
    }

    li > :first-child {
        white-space: nowrap;
    }

    .summary {
        display: flex;
        flex-wrap: wrap;
        justify-content: space-between;
        align-items: center;
        gap: 20px;
    }

    .summary.tier {
        justify-content: flex-start;
    }

    .summary-main {
        display: flex;
        flex-direction: column;
        gap: 8px;
        min-width: 0;
    }

    .summary-side {
        display: flex;
        flex-direction: column;
        align-items: flex-end;
        gap: 14px;
    }

    .big {
        font-size: 1.875rem;
        font-weight: 700;
        line-height: 1.1;
    }

    .medium {
        font-size: 1.375rem;
        font-weight: 700;
    }

    .small {
        font-size: 0.8rem;
    }

    .summary[data-profile="full"] {
        --profile-color: var(--good);
    }
    .summary[data-profile="balanced"] {
        --profile-color: var(--warn);
    }
    .summary[data-profile="saver"] {
        --profile-color: var(--bad);
    }

    .profile-color {
        color: var(--profile-color);
    }

    .tier-color[data-tier="high"] {
        color: var(--good);
    }
    .tier-color[data-tier="mid"] {
        color: var(--warn);
    }
    .tier-color[data-tier="low"] {
        color: var(--bad);
    }

    .chip {
        font-size: 0.75rem;
        font-weight: 600;
        padding: 3px 10px;
        border-radius: 999px;
    }

    .chip.ok {
        color: var(--good-text);
        background: var(--good-bg);
        border: 1px solid var(--good-border);
    }

    .chip.bad {
        color: var(--bad-text);
        background: var(--bad-bg);
        border: 1px solid var(--bad-border);
    }

    .chip.sim {
        color: var(--accent-tag);
        background: var(--accent-bg);
    }

    .mini-scale {
        display: flex;
        align-items: center;
    }

    .mini-dot {
        width: 14px;
        height: 14px;
        border-radius: 50%;
        border: 2px solid var(--track-strong);
    }

    .mini-dot.passed,
    .mini-dot.on {
        border-color: var(--profile-color);
        background: var(--profile-color);
    }

    .mini-dot.on {
        box-shadow: 0 0 10px var(--profile-color);
    }

    .mini-rail {
        width: 34px;
        height: 3px;
        background: var(--track);
    }

    .mini-rail.filled {
        background: var(--profile-color);
    }

    .link {
        font-size: 0.8rem;
        font-weight: 600;
        color: var(--accent-soft);
    }

    .tile[data-tile="battery"] {
        --tile-accent: var(--tile-battery-accent);
        --tile-label: var(--tile-battery-label);
        background: var(--tile-battery-bg);
        border-color: var(--tile-battery-border);
    }
    .tile[data-tile="network"] {
        --tile-accent: var(--tile-network-accent);
        --tile-label: var(--tile-network-label);
        background: var(--tile-network-bg);
        border-color: var(--tile-network-border);
    }
    .tile[data-tile="storage"] {
        --tile-accent: var(--tile-storage-accent);
        --tile-label: var(--tile-storage-label);
        background: var(--tile-storage-bg);
        border-color: var(--tile-storage-border);
    }
    .tile[data-tile="display"] {
        --tile-accent: var(--tile-display-accent);
        --tile-label: var(--tile-display-label);
        background: var(--tile-display-bg);
        border-color: var(--tile-display-border);
    }

    .tile .label {
        color: var(--tile-label);
    }

    .tile {
        display: flex;
        flex-direction: column;
        gap: 12px;
    }

    .inline {
        display: flex;
        align-items: center;
        gap: 12px;
    }

    .stack {
        display: flex;
        flex-direction: column;
    }

    .bar {
        height: 8px;
        border-radius: 4px;
        background: var(--track);
        overflow: hidden;
    }

    .sweep {
        transition: stroke-dasharray calc(1000ms * var(--m)) var(--ease-out) calc(300ms * var(--m));
    }

    .bar-fill {
        height: 100%;
        background: var(--tile-accent);
    }

    .screen {
        border: 2px solid var(--tile-accent);
        border-radius: 5px;
        background: var(--accent-tint); /* fallback: WebViews without color-mix() (Chrome < 111) */
        background: color-mix(in srgb, var(--tile-accent) 15%, transparent);
    }

    .events {
        display: flex;
        flex-direction: column;
    }

    .events-head {
        display: flex;
        justify-content: space-between;
        align-items: baseline;
        gap: 12px;
        flex-wrap: wrap;
        margin-bottom: 10px;
    }

    .empty {
        margin: 6px 0 0;
        font-size: 0.875rem;
        line-height: 1.5;
        color: var(--soft);
    }

    ul {
        list-style: none;
        margin: 0;
        padding: 0;
    }

    li {
        border-radius: 8px;
        display: grid;
        grid-template-columns: 72px 160px minmax(0, 1fr);
        align-items: center;
        gap: 12px;
        padding: 9px 0;
        border-top: 1px solid var(--divider);
    }

    @media (max-width: 640px) {
        li {
            grid-template-columns: minmax(0, 1fr) auto;
            grid-template-areas:
                "api time"
                "text text";
            row-gap: 4px;
            padding: 10px 0;
        }
        .api {
            grid-area: api;
            font-size: 0.7rem;
        }
        .time {
            grid-area: time;
        }
        .text {
            grid-area: text;
        }
    }

    .api {
        justify-self: start;
        font-size: 0.75rem;
        padding: 2px 8px;
        border-radius: 6px;
        color: var(--badge-display);
        background: var(--badge-display-bg);
    }

    .api[data-kind="power"] {
        color: var(--badge-power);
        background: var(--badge-power-bg);
    }
    .api[data-kind="battery"] {
        color: var(--badge-battery);
        background: var(--badge-battery-bg);
    }
    .api[data-kind="network"] {
        color: var(--badge-network);
        background: var(--badge-network-bg);
    }
    .api[data-kind="storage"] {
        color: var(--badge-storage);
        background: var(--badge-storage-bg);
    }

    .text {
        font-size: 0.875rem;
        color: var(--text-2);
    }
</style>
