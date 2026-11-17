<script lang="ts">
    import { store } from "../state/device.svelte";
    import {
        capitalize,
        formatGb,
        formatMemory,
        formatTime,
        gb,
        maskIp,
        maskMac,
        maskTail,
        networkLabel,
        platform,
        storageLabel,
        watchMode,
        type WatchKind,
    } from "../state/format";
    import Sparkline from "../ui/Sparkline.svelte";
    import CountUp from "../ui/CountUp.svelte";
    import { onMount } from "svelte";

    const DONUT_C = 2 * Math.PI * 50;
    const KINDS: WatchKind[] = ["battery", "network", "storage", "display", "power"];
    const nativeCount = KINDS.filter((k) => watchMode(k).native).length;
    const NETWORK_COLORS: Record<string, string> = {
        wifi: "var(--net-wifi)",
        ethernet: "var(--net-ethernet)",
        cellular: "var(--net-cellular)",
    };

    // Identifiers are masked so the demo is safe to screenshot.
    // Shared with every page (see store.reveal): masked until the person presses Show.
    let reveal = $derived(store.reveal);

    // Revealing the identifiers "decodes" them left to right over 0.9 s.
    let decode = $state(1);
    let decodeRaf = 0;
    function toggleReveal() {
        store.reveal = !store.reveal;
        cancelAnimationFrame(decodeRaf);
        const still =
            document.documentElement.dataset.power === "saver" ||
            window.matchMedia("(prefers-reduced-motion: reduce)").matches;
        if (!reveal || still) {
            decode = 1;
            return;
        }
        const start = performance.now();
        const tick = (now: number) => {
            decode = Math.min(1, (now - start) / 900);
            if (decode < 1) decodeRaf = requestAnimationFrame(tick);
        };
        decode = 0;
        decodeRaf = requestAnimationFrame(tick);
    }
    function scramble(real: string | null | undefined, t: number): string {
        if (!real) return "—";
        if (t >= 1) return real;
        const CH = "0123456789ABCDEF";
        return [...real]
            .map((ch, i) => (ch === "-" ? ch : i / real.length < t ? ch : CH[(i * 7 + Math.floor(t * 40)) % 16]))
            .join("");
    }

    // Rings start empty and sweep to their value once on screen.
    let ready = $state(false);
    onMount(() => {
        const id = requestAnimationFrame(() => (ready = true));
        return () => {
            cancelAnimationFrame(id);
            cancelAnimationFrame(decodeRaf);
        };
    });

    let device = $derived(store.device);
    let hw = $derived(store.tier?.hardware);
    let display = $derived(store.display);
    let storage = $derived(store.storage);
    let network = $derived(store.network);

    let specs = $derived(
        [
            { k: "CPU", v: hw?.cpuModel },
            { k: "Cores", v: hw?.cpuCores?.toString() },
            { k: "Memory", v: hw?.totalMemory ? formatMemory(hw.totalMemory) : undefined },
            { k: "OS", v: platform },
            { k: "Tier", v: store.tier ? `${capitalize(store.tier.tier)} · ${store.tier.score}` : undefined },
        ].filter((s) => s.v),
    );

    let isPhone = $derived(hw?.isMobile ?? (platform === "iOS" || platform === "Android"));

    // Display drawn at its real aspect ratio, fitted inside a 200 × 150 box —
    // so a portrait phone screen stays tall instead of being squashed.
    let screenBox = $derived.by(() => {
        const w = display?.width ?? 16;
        const h = display?.height ?? 10;
        const k = Math.min(200 / w, 150 / h);
        return { w: Math.round(w * k), h: Math.round(h * k), portrait: h > w };
    });
    let scale = $derived(display?.scaleFactor ?? 1);
    // "1 point = n×n pixels", drawn as an n×n grid (capped at 3×3).
    let pixelSide = $derived(Math.min(3, Math.max(1, Math.round(scale))));

    // What the plugin's two identifiers actually are on each platform.
    let idLabels = $derived(
        platform === "iOS"
            ? ["Vendor ID", "Keychain ID"]
            : platform === "Android"
              ? ["Android ID", "Serial"]
              : ["UUID", "Serial"],
    );
    let macIsReal = $derived(/^([0-9a-f]{2}:){5}[0-9a-f]{2}$/i.test(network?.macAddress ?? ""));
    let logical = $derived(
        display?.width ? `${Math.round(display.width / scale)} × ${Math.round((display.height ?? 0) / scale)}` : "—",
    );

    let usedPct = $derived(
        storage?.totalSpace ? Math.round((1 - (storage.freeSpace ?? 0) / storage.totalSpace) * 100) : 0,
    );
    let freeValues = $derived(store.storageHistory.map((s) => s.value));
    let freeMin = $derived(Math.min(...freeValues, storage?.freeSpace ?? 0));
    let freeMax = $derived(Math.max(...freeValues, storage?.freeSpace ?? 0));
    // Pad the range so a steady value draws mid-chart instead of on an edge.
    let pad = $derived(Math.max((freeMax - freeMin) * 0.2, 2e9));
    let freeDelta = $derived(
        store.storageHistory.length > 0 ? gb(storage?.freeSpace) - gb(store.storageHistory[0].value) : 0,
    );

    let segments = $derived.by(() => {
        const span = Math.max(1, store.now - store.startedAt);
        return store.networkHistory.map((seg, i) => {
            const end = store.networkHistory[i + 1]?.at ?? store.now;
            return {
                type: seg.type,
                at: seg.at,
                pct: Math.max(1, ((end - seg.at) / span) * 100),
                color: NETWORK_COLORS[seg.type] ?? "var(--muted)",
            };
        });
    });
    let legend = $derived([...new Map(segments.map((s) => [s.type, s.color])).entries()]);
</script>

{#snippet watchFooter(kind: WatchKind)}
    {@const mode = watchMode(kind)}
    {@const last = store.lastChange[kind]}
    <div class="footer">
        <span class="mode" class:native={mode.native}><span class="mode-dot"></span>{mode.label}</span>
        <span class="muted">
            {last ? `Last change ${formatTime(last.at)} — ${last.text}` : "No changes since the app started"}
        </span>
    </div>
{/snippet}

<div class="page">
    <header class="page-header">
        <div>
            <h1>System</h1>
            <p>Device identity, display, storage and network — each one a getter plus a watcher.</p>
        </div>
        <span class="muted small">{nativeCount} native watchers · {KINDS.length - nativeCount} polled</span>
    </header>

    <!-- Device hero -->
    <section class="card hero m-card" style:--i="0" aria-label="This device">
        <div class="glow" aria-hidden="true"></div>

        {#if isPhone}
            <div class="phone" aria-hidden="true">
                <div class="lcd m-poweron">
                    {#if store.tier}
                        <span class="lcd-score"><CountUp value={store.tier.score} /></span>
                        <span class="lcd-tier" data-tier={store.tier.tier}>{store.tier.tier.toUpperCase()}</span>
                    {/if}
                </div>
            </div>
        {:else}
            <div class="laptop" aria-hidden="true">
                <div class="lid">
                    <div class="lcd m-poweron">
                        {#if store.tier}
                            <span class="lcd-score"><CountUp value={store.tier.score} /></span>
                            <span class="lcd-tier" data-tier={store.tier.tier}>{store.tier.tier.toUpperCase()} TIER</span>
                        {/if}
                    </div>
                </div>
                <div class="base"></div>
            </div>
        {/if}

        <div class="hero-main">
            <span class="label">This device</span>
            <span class="hero-title">{device?.model ?? "Unknown model"}</span>
            <span class="soft">{[device?.manufacturer, reveal ? device?.device_name : device?.device_name && "Name hidden"].filter(Boolean).join(" · ")}</span>
            <div class="specs">
                {#each specs as s, i}
                    <span class="spec m-pop" style:--d="300ms" style:--i={i}><span class="muted">{s.k}</span><b>{s.v}</b></span>
                {/each}
            </div>
        </div>

        <div class="ids">
            <div class="ids-head">
                <span class="muted ids-title">Identifiers</span>
                <button class="reveal" onclick={toggleReveal} aria-pressed={reveal}>
                    {reveal ? "Hide" : "Show"}
                </button>
            </div>
            <div class="kv"><span class="muted">{idLabels[0]}</span><span class="mono">{reveal ? scramble(device?.uuid, decode) : maskTail(device?.uuid)}</span></div>
            <div class="kv"><span class="muted">{idLabels[1]}</span><span class="mono">{reveal ? scramble(device?.serial, decode) : maskTail(device?.serial, 2)}</span></div>
            <span class="api-tag">getDeviceInfo()</span>
        </div>
    </section>

    <div class="row-3">
        <!-- Display -->
        <section class="card detail m-card" style:--i="1" aria-label="Display">
            <div class="detail-head"><span class="label">Display</span><span class="api-tag">watchDisplay()</span></div>
            {#if screenBox.portrait}
                <!-- Portrait (phones): the screen is too narrow for its own label. -->
                <div class="monitor portrait">
                    <div class="panel m-reveal" style:width="{screenBox.w}px" style:height="{screenBox.h}px" aria-hidden="true">
                        {#if display?.refreshRate}<span class="mono hz-center">{Math.round(display.refreshRate)} Hz</span>{/if}
                    </div>
                    <div class="stack">
                        <span class="res">{display ? `${display.width} × ${display.height}` : "—"}</span>
                        <span class="muted small">physical pixels</span>
                    </div>
                </div>
            {:else}
                <div class="monitor" aria-hidden="true">
                    <div class="panel m-reveal" style:width="{screenBox.w}px" style:height="{screenBox.h}px">
                        <span class="res">{display ? `${display.width} × ${display.height}` : "—"}</span>
                        <span class="muted small">physical pixels</span>
                        {#if display?.refreshRate}<span class="hz mono">{Math.round(display.refreshRate)} Hz</span>{/if}
                    </div>
                    <div class="stand"></div>
                </div>
            {/if}
            <div class="explain">
                <div class="pixels" aria-hidden="true" style:grid-template-columns="repeat({pixelSide}, {pixelSide > 2 ? 9 : 14}px)">
                    {#each Array(pixelSide * pixelSide) as _, i}<span class="m-pop" style:--d="700ms" style:--i={i} style:height="{pixelSide > 2 ? 9 : 14}px"></span>{/each}
                </div>
                <div class="stack">
                    <span class="explain-title">Scale {scale.toFixed(1)}× — 1 point = {scale}×{scale} pixels</span>
                    <span class="soft small">Your UI is laid out at {logical} points</span>
                </div>
            </div>
            {@render watchFooter("display")}
        </section>

        <!-- Storage -->
        <section class="card detail m-card" style:--i="2" aria-label="Storage">
            <div class="detail-head"><span class="label">Storage</span><span class="api-tag">watchStorage()</span></div>
            <div class="storage-top">
                <div class="donut">
                    <svg width="120" height="120" viewBox="0 0 120 120" role="img" aria-label="{usedPct}% used">
                        <circle cx="60" cy="60" r="50" fill="none" stroke="var(--track)" stroke-width="14" />
                        <circle cx="60" cy="60" r="50" fill="none" stroke="var(--accent-line)" stroke-width="14" stroke-dasharray="{((ready ? usedPct : 0) / 100) * DONUT_C} {DONUT_C}" transform="rotate(-90 60 60)" class="sweep" />
                    </svg>
                    <div class="donut-center"><b><CountUp value={usedPct} />%</b><span class="muted small">used</span></div>
                </div>
                <div class="stack">
                    <div class="stack"><b class="value-xl">{#if gb(storage?.freeSpace) < 1000}<CountUp value={Math.round(gb(storage?.freeSpace))} /> GB{:else}{formatGb(storage?.freeSpace)}{/if}</b><span class="muted small">free of {formatGb(storage?.totalSpace)}</span></div>
                    <span class="type-pill">{storageLabel(storage?.storageType)}</span>
                </div>
            </div>
            <div class="explain column">
                <div class="row-between small">
                    <span class="soft">Free space · this session</span>
                    <span class="delta" class:down={freeDelta <= -1} class:up={freeDelta >= 1}>
                        {Math.abs(freeDelta) < 1 ? "steady" : `${freeDelta > 0 ? "+" : "−"}${Math.abs(Math.round(freeDelta))} GB`}
                    </span>
                </div>
                <div class="spark">
                    <Sparkline
                        samples={store.storageHistory}
                        start={store.startedAt}
                        end={store.now}
                        min={freeMin - pad}
                        max={freeMax + pad}
                        height={48}
                        color="var(--tile-storage-accent)"
                        label="Free disk space since the app started"
                    />
                </div>
            </div>
            {@render watchFooter("storage")}
        </section>

        <!-- Network -->
        <section class="card detail m-card" style:--i="3" aria-label="Network">
            <div class="detail-head"><span class="label">Network</span><span class="api-tag">watchNetwork()</span></div>
            <div class="net-top">
                <div class="net-icon" aria-hidden="true">
                    <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="var(--net-icon)" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path class="m-light" style:--i="0" d="M12 19.5h.01" /><path class="m-light" style:--i="1" d="M8.5 16a5 5 0 0 1 7 0" /><path class="m-light" style:--i="2" d="M5 12.5a10 10 0 0 1 14 0" /><path class="m-light" style:--i="3" d="M2 9a15 15 0 0 1 20 0" /></svg>
                </div>
                <div class="stack">
                    <b class="value-xl">{networkLabel(network?.networkType)}</b>
                    <span class="mono soft small">{reveal ? (network?.ipAddress ?? "—") : maskIp(network?.ipAddress)}</span>
                </div>
            </div>
            <div class="explain column">
                <span class="soft small">Connection · this session</span>
                <div class="timeline" role="img" aria-label="Connection types since the app started">
                    {#each segments as seg, i}
                        <div class="m-growx" style:--i={i} style:width="{seg.pct}%" style:background={seg.color}></div>
                    {/each}
                </div>
                <div class="row-between mono muted tiny">
                    <span>{formatTime(store.startedAt)}</span><span>now</span>
                </div>
                <div class="legend small">
                    {#each legend as [type, color]}
                        <span><i style:background={color}></i>{networkLabel(type)}</span>
                    {/each}
                </div>
            </div>
            <div class="kv">
                <span class="muted">MAC</span>
                {#if macIsReal}
                    <span class="mono">{reveal ? network?.macAddress : maskMac(network?.macAddress)}</span>
                {:else}
                    <span class="soft">{platform === "Android" ? "Restricted on Android" : platform === "iOS" ? "Unavailable on iOS" : "—"}</span>
                {/if}
            </div>
            {@render watchFooter("network")}
        </section>
    </div>
</div>

<style>
    .small {
        font-size: 0.8rem;
    }

    .tiny {
        font-size: 0.7rem;
    }

    .stack {
        display: flex;
        flex-direction: column;
        gap: 4px;
        min-width: 0;
    }

    .row-between {
        display: flex;
        justify-content: space-between;
        align-items: center;
    }

    .value-xl {
        font-size: 1.5rem;
        font-weight: 700;
    }

    /* ---- Hero ---- */
    .hero {
        position: relative;
        overflow: hidden;
        display: flex;
        align-items: center;
        gap: 32px;
        flex-wrap: wrap;
        padding: 22px 28px;
        border-radius: 16px;
        border-color: var(--track);
    }

    .glow {
        position: absolute;
        right: -80px;
        top: -120px;
        width: 360px;
        height: 360px;
        border-radius: 50%;
        background: var(--glow);
        pointer-events: none;
    }

    @media (max-width: 640px) {
        .glow {
            display: none;
        }

        .hero {
            gap: 16px;
            padding: 16px;
        }

        .hero-main {
            min-width: 0;
        }

        .hero-title {
            font-size: 1.5rem;
        }

        .ids {
            min-width: 0;
            flex: 1 1 100%;
        }
    }

    .laptop {
        position: relative;
        display: flex;
        flex-direction: column;
        align-items: center;
        flex-shrink: 0;
    }

    .phone {
        position: relative;
        width: 56px;
        height: 112px;
        flex-shrink: 0;
        padding: 4px;
        border-radius: 15px;
        background: var(--device-lid);
        border: 2px solid var(--device-edge);
    }

    .phone .lcd {
        border-radius: 11px;
        gap: 2px;
    }

    .phone .lcd-score {
        font-size: 0.875rem;
    }

    .phone .lcd-tier {
        font-size: 0.45rem;
        letter-spacing: 0.1em;
    }

    .lid {
        width: 196px;
        height: 124px;
        padding: 7px;
        border-radius: 10px 10px 4px 4px;
        background: var(--device-lid);
        border: 2px solid var(--device-edge);
    }

    .lcd {
        width: 100%;
        height: 100%;
        border-radius: 4px;
        background: var(--device-screen);
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        gap: 4px;
    }

    .lcd-score {
        font-size: 1.125rem;
        font-weight: 700;
        color: var(--accent-soft);
    }

    .lcd-tier {
        font-size: 0.55rem;
        font-weight: 700;
        letter-spacing: 0.14em;
        color: var(--good);
    }
    .lcd-tier[data-tier="mid"] {
        color: var(--warn);
    }
    .lcd-tier[data-tier="low"] {
        color: var(--bad);
    }

    .base {
        width: 236px;
        height: 10px;
        border-radius: 0 0 10px 10px;
        background: var(--device-base);
        border: 2px solid var(--device-edge);
        border-top: 0;
    }

    .hero-main {
        position: relative;
        flex: 1;
        min-width: 260px;
        display: flex;
        flex-direction: column;
        gap: 10px;
    }

    .hero-title {
        font-size: 1.875rem;
        font-weight: 700;
        line-height: 1.1;
    }

    .specs {
        display: flex;
        flex-wrap: wrap;
        gap: 8px;
        margin-top: 4px;
    }

    .spec {
        display: flex;
        gap: 6px;
        padding: 5px 11px;
        border-radius: 8px;
        background: var(--chip-bg);
        border: 1px solid var(--track);
        font-size: 0.8rem;
    }

    .ids {
        position: relative;
        display: flex;
        flex-direction: column;
        gap: 10px;
        min-width: 250px;
        padding: 14px 16px;
        border-radius: 12px;
        background: var(--sunken);
        border: 1px solid var(--divider);
    }

    .ids-head {
        display: flex;
        justify-content: space-between;
        align-items: center;
    }

    .ids-title {
        font-size: 0.75rem;
        font-weight: 600;
    }

    .reveal {
        all: unset;
        cursor: pointer;
        min-height: 28px;
        padding: 0 10px;
        border-radius: 6px;
        font-size: 0.75rem;
        font-weight: 600;
        color: var(--accent-soft);
        border: 1px solid var(--border); /* fallback: WebViews without color-mix() (Chrome < 111) */
        border: 1px solid color-mix(in srgb, var(--accent-soft) 45%, transparent);
    }

    .reveal:hover {
        background: var(--accent-bg);
    }

    .reveal:focus-visible {
        outline: 2px solid var(--accent);
    }

    .kv {
        display: flex;
        justify-content: space-between;
        gap: 12px;
        font-size: 0.8rem;
    }

    .kv .mono {
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
    }

    /* ---- Detail cards ---- */
    .row-3 {
        display: grid;
        grid-template-columns: repeat(3, minmax(0, 1fr));
        gap: 16px;
    }

    @media (max-width: 1100px) {
        .row-3 {
            grid-template-columns: 1fr;
        }
    }

    .detail {
        display: flex;
        flex-direction: column;
        gap: 14px;
    }

    .detail-head {
        display: flex;
        justify-content: space-between;
        align-items: center;
    }

    .explain {
        display: flex;
        align-items: center;
        gap: 14px;
        padding: 12px;
        border-radius: 10px;
        background: var(--sunken);
    }

    .explain.column {
        flex-direction: column;
        align-items: stretch;
        gap: 8px;
    }

    .explain-title {
        font-size: 0.875rem;
        font-weight: 600;
    }

    .footer {
        margin-top: auto;
        display: flex;
        flex-direction: column;
        gap: 6px;
        padding-top: 12px;
        border-top: 1px solid var(--divider);
        font-size: 0.75rem;
    }

    .mode {
        display: flex;
        align-items: center;
        gap: 6px;
        color: var(--warn-text);
    }

    .mode-dot {
        width: 7px;
        height: 7px;
        border-radius: 50%;
        background: var(--warn);
    }

    .mode.native {
        color: var(--good-text);
    }

    .mode.native .mode-dot {
        background: var(--good);
    }

    .sweep {
        transition: stroke-dasharray calc(1000ms * var(--m)) var(--ease-out) calc(300ms * var(--m));
    }

    /* Display */
    .monitor {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 5px;
        padding: 6px 0;
    }

    .monitor.portrait {
        flex-direction: row;
        justify-content: center;
        gap: 20px;
    }

    .monitor.portrait .panel {
        border-radius: 14px;
    }

    .hz-center {
        font-size: 0.7rem;
        color: var(--accent-tag);
    }

    .panel {
        position: relative;
        border: 3px solid var(--accent-line);
        border-radius: 8px;
        background: var(--accent-tint);
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        gap: 2px;
    }

    .res {
        font-size: 1.125rem;
        font-weight: 700;
    }

    .hz {
        position: absolute;
        right: 8px;
        bottom: 6px;
        font-size: 0.7rem;
        color: var(--accent-tag);
    }

    .stand {
        width: 64px;
        height: 6px;
        border-radius: 3px;
        background: var(--track); /* fallback: WebViews without color-mix() (Chrome < 111) */
        background: color-mix(in srgb, var(--accent-line) 40%, transparent);
    }

    .pixels {
        display: grid;
        gap: 2px;
        padding: 3px;
        border: 1px dashed var(--accent-soft); /* fallback: WebViews without color-mix() (Chrome < 111) */
        border: 1px dashed color-mix(in srgb, var(--accent-soft) 45%, transparent);
        border-radius: 4px;
        flex-shrink: 0;
    }

    .pixels span {
        border-radius: 2px;
        background: var(--accent-line);
    }

    /* Storage */
    .storage-top,
    .net-top {
        display: flex;
        align-items: center;
        gap: 18px;
    }

    .donut {
        position: relative;
        width: 120px;
        height: 120px;
        flex-shrink: 0;
    }

    .donut-center {
        position: absolute;
        inset: 0;
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
    }

    .donut-center b {
        font-size: 1.5rem;
    }

    .type-pill {
        align-self: flex-start;
        margin-top: 4px;
        padding: 2px 10px;
        border-radius: 6px;
        background: var(--chip-bg);
        font-size: 0.75rem;
        font-weight: 600;
    }

    .delta {
        font-weight: 600;
        color: var(--muted);
    }

    .delta.down {
        color: var(--bad-text);
    }

    .delta.up {
        color: var(--good-text);
    }

    .spark {
        position: relative;
    }

    /* Network */
    .net-icon {
        width: 56px;
        height: 56px;
        border-radius: 14px;
        background: var(--net-icon-bg);
        display: flex;
        align-items: center;
        justify-content: center;
        flex-shrink: 0;
    }

    .timeline {
        display: flex;
        gap: 2px;
        height: 12px;
        border-radius: 6px;
        overflow: hidden;
        background: var(--divider);
    }

    .legend {
        display: flex;
        flex-wrap: wrap;
        gap: 14px;
        color: var(--soft);
    }

    .legend span {
        display: flex;
        align-items: center;
        gap: 6px;
    }

    .legend i {
        width: 10px;
        height: 10px;
        border-radius: 3px;
    }
</style>
