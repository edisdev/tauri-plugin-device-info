<script lang="ts">
    import type { ThermalState } from "tauri-plugin-device-info-api";
    import { store } from "../state/device.svelte";
    import { capitalize, formatHealth, formatTime, platform, watchMode } from "../state/format";
    import Sparkline from "../ui/Sparkline.svelte";
    import CountUp from "../ui/CountUp.svelte";
    import { onMount } from "svelte";

    // The battery ring starts empty and sweeps to its level once on screen.
    let ready = $state(false);
    onMount(() => {
        const id = requestAnimationFrame(() => (ready = true));
        return () => cancelAnimationFrame(id);
    });

    const THERMAL_LEVELS: ThermalState[] = ["nominal", "fair", "serious", "critical"];
    const RING_C = 2 * Math.PI * 52;

    let battery = $derived(store.battery);
    let power = $derived(store.power);
    let hasBattery = $derived(battery?.level !== undefined && battery?.level !== null);
    let level = $derived(Math.round(battery?.level ?? 0));
    let levelColor = $derived(level <= 20 ? "var(--bad)" : level <= 40 ? "var(--warn)" : "var(--good)");
    let thermalIndex = $derived(power?.thermalState ? THERMAL_LEVELS.indexOf(power.thermalState) : -1);
    let lpmSource = $derived(
        {
            macOS: "NSProcessInfo (macOS 12+)",
            iOS: "ProcessInfo",
            Android: "PowerManager",
            Windows: "Battery Saver · GetSystemPowerStatus",
            Linux: "ACPI platform profile",
        }[platform],
    );
    let batteryWatch = watchMode("battery");
</script>

<div class="page">
    <header class="page-header">
        <div>
            <h1>Power</h1>
            <p>Battery, power-saving mode and thermal state — the inputs to the power profile.</p>
        </div>
    </header>

    <!-- Three signals -->
    <div class="row-3">
        <section class="card battery m-card" style:--i="0" aria-label="Battery">
            {#if hasBattery}
                <!-- Charging: the ring glows in a slow loop (live state). -->
                <div class="ring" class:m-glow={battery?.isCharging} class:m-loop={battery?.isCharging}>
                    <svg width="116" height="116" viewBox="0 0 116 116" role="img" aria-label="Battery {level}%">
                        <circle cx="58" cy="58" r="52" fill="none" stroke="var(--track)" stroke-width="10" />
                        <circle
                            class="ring-fill"
                            cx="58"
                            cy="58"
                            r="52"
                            fill="none"
                            stroke={levelColor}
                            stroke-width="10"
                            stroke-linecap="round"
                            stroke-dasharray="{((ready ? level : 0) / 100) * RING_C} {RING_C}"
                            transform="rotate(-90 58 58)"
                        />
                    </svg>
                    <span class="ring-value"><CountUp value={level} />%</span>
                </div>
            {/if}
            <div class="stack">
                <span class="label">Battery</span>
                {#if hasBattery}
                    <span class="value-lg charge-row">
                        {#if battery?.isCharging}
                            <svg class="bolt m-pop" width="18" height="18" viewBox="0 0 24 24" role="img" aria-label="Charging"><path d="M13 2 4 14h7l-1 8 9-12h-7z" fill="var(--warn)" /></svg>
                        {/if}
                        {battery?.isCharging ? "Charging" : "Discharging"}
                    </span>
                    {#if battery?.health}<span class="soft small">Health: {formatHealth(battery.health)}</span>{/if}
                {:else}
                    <span class="value-lg">Mains power</span>
                    <span class="soft small">No battery — it never affects the profile.</span>
                {/if}
                <span class="api-tag">watchBattery()</span>
            </div>
        </section>

        <section class="card thermal-card m-card" style:--i="1" aria-label="Thermal state">
            <span class="label">Thermal state</span>
            {#if thermalIndex >= 0}
                <div class="thermal-bars" aria-hidden="true">
                    {#each THERMAL_LEVELS as _, i}
                        <!-- Bars rise in order; the current level breathes (live state). -->
                        <div class="tbar m-rise" class:m-loop={i === thermalIndex} style:--i={i} class:lit={i <= thermalIndex} class:now={i === thermalIndex} data-step={i}></div>
                    {/each}
                </div>
                <div class="thermal-labels">
                    {#each THERMAL_LEVELS as t, i}
                        <span class:current={i === thermalIndex} data-step={i}>{capitalize(t)}</span>
                    {/each}
                </div>
                <!-- Phones: four labels don't fit under the bars; name the current state instead. -->
                <span class="thermal-now" data-step={thermalIndex}>{capitalize(power?.thermalState)}</span>
            {:else}
                <span class="value-lg">Not available</span>
                <span class="soft small">{platform} exposes no thermal-pressure API.</span>
            {/if}
            <span class="soft small push">Fair → Balanced · Serious or Critical → Saver</span>
        </section>

        <section class="card lpm m-card" style:--i="2" aria-label="Low Power Mode">
            <span class="label">Low Power Mode</span>
            <span class="lpm-pill m-pop" style:--d="380ms" class:on={power?.lowPowerMode}>
                {power?.lowPowerMode === undefined || power?.lowPowerMode === null ? "N/A" : power.lowPowerMode ? "On" : "Off"}
            </span>
            <span class="soft small">When it's on, the profile drops straight to Saver — whatever the battery says.</span>
            <span class="muted small push">Source: {lpmSource}</span>
        </section>
    </div>

    <!-- Session history -->
    {#if hasBattery}
        <section class="card m-card" style:--i="3" aria-label="Battery level this session">
            <div class="head">
                <span class="label">Battery level — this session</span>
                <span class="muted small">Built from watchBattery() events · {batteryWatch.label}</span>
            </div>
            <div class="chart">
                <div class="threshold warn m-growx" style:--d="200ms" style:top="{200 - 0.4 * 200}px"><span>Balanced below 40%</span></div>
                <div class="threshold bad m-growx" style:--d="280ms" style:top="{200 - 0.2 * 200}px"><span>Saver below 20%</span></div>
                <Sparkline
                    samples={store.batteryHistory}
                    start={store.startedAt}
                    end={store.now}
                    min={0}
                    max={100}
                    height={200}
                    color={levelColor}
                    area
                    label="Battery level since the app started"
                />
            </div>
            <div class="axis mono">
                <span>{formatTime(store.startedAt)}</span>
                <span>now</span>
            </div>
        </section>

        <!-- Threshold bar -->
        <section class="card m-card" style:--i="4" aria-label="How battery affects the profile">
            <div class="head">
                <span class="label">How battery affects the profile</span>
                <span class="soft small">Only while discharging — charging never lowers the profile.</span>
            </div>
            <div class="zones-wrap">
                <div class="zones">
                    <div class="zone bad m-growx" style:--i="0" style:width="20%"></div>
                    <div class="zone warn m-growx" style:--i="1" style:width="20%"></div>
                    <div class="zone good m-growx" style:--i="2" style:width="60%"></div>
                </div>
                <!-- Drops in and settles with a small bounce; slides when the level changes. -->
                <div class="marker m-drop" style:left="{level}%">
                    <span>Now · {level}%</span>
                    <i></i>
                </div>
            </div>
            <div class="zone-labels">
                <span style:width="20%" class="t-bad"><span class="long">0–20% ·&nbsp;</span>Saver</span>
                <span style:width="20%" class="t-warn"><span class="long">21–40% ·&nbsp;</span><span class="short">Bal.</span><span class="long">Balanced</span></span>
                <span style:width="60%" class="t-good"><span class="long">41–100% ·&nbsp;</span>Full</span>
            </div>
        </section>
    {/if}
</div>

<style>
    .row-3 {
        display: grid;
        grid-template-columns: repeat(3, minmax(0, 1fr));
        gap: 16px;
    }

    @media (max-width: 1100px) {
        .row-3 {
            grid-template-columns: repeat(2, minmax(0, 1fr));
        }

        /* Battery gets the full row; thermal and Low Power Mode sit side by side. */
        .battery {
            grid-column: 1 / -1;
        }
    }

    .thermal-now {
        display: none;
        font-size: 1.125rem;
        font-weight: 600;
    }
    .thermal-now[data-step="0"] { color: var(--good); }
    .thermal-now[data-step="1"] { color: var(--warn); }
    .thermal-now[data-step="2"] { color: var(--orange); }
    .thermal-now[data-step="3"] { color: var(--bad); }

    @media (max-width: 640px) {
        .row-3 {
            gap: 12px;
        }

        .thermal-card .thermal-labels {
            display: none;
        }

        .thermal-card .thermal-now {
            display: block;
        }

        .thermal-card .thermal-bars {
            height: 40px;
        }

        .long {
            display: none;
        }
    }

    @media (min-width: 641px) {
        .short {
            display: none;
        }
    }

    .small {
        font-size: 0.8rem;
    }

    .stack {
        display: flex;
        flex-direction: column;
        gap: 8px;
    }

    .push {
        margin-top: auto;
    }

    .value-lg {
        font-size: 1.125rem;
        font-weight: 600;
    }

    .battery {
        display: flex;
        align-items: center;
        gap: 18px;
    }

    .ring {
        position: relative;
        width: 116px;
        height: 116px;
        flex-shrink: 0;
    }

    .ring-fill {
        transition: stroke-dasharray calc(1200ms * var(--m)) var(--ease-out) calc(150ms * var(--m));
    }

    .charge-row {
        display: flex;
        align-items: center;
        gap: 6px;
    }

    /* The current thermal level rises, then keeps breathing. */
    .tbar.now {
        animation:
            v-rise calc(420ms * var(--m)) var(--spring) calc((250ms + var(--i) * 60ms) * var(--m)) both,
            v-breathe 2.4s ease-in-out 900ms infinite;
    }

    .tbar:not(.now) {
        animation-iteration-count: 1;
    }

    .ring-value {
        position: absolute;
        inset: 0;
        display: flex;
        align-items: center;
        justify-content: center;
        font-size: 1.75rem;
        font-weight: 700;
    }

    .thermal-card,
    .lpm {
        display: flex;
        flex-direction: column;
        gap: 12px;
    }

    .thermal-bars {
        display: grid;
        grid-template-columns: repeat(4, minmax(0, 1fr));
        gap: 6px;
        align-items: end;
        height: 56px;
    }

    .tbar {
        border-radius: 4px;
        background: var(--track);
    }
    .tbar[data-step="0"] { height: 40%; }
    .tbar[data-step="1"] { height: 60%; }
    .tbar[data-step="2"] { height: 80%; }
    .tbar[data-step="3"] { height: 100%; }
    .tbar.lit[data-step="0"] { background: var(--good); }
    .tbar.lit[data-step="1"] { background: var(--warn); }
    .tbar.lit[data-step="2"] { background: var(--orange); }
    .tbar.lit[data-step="3"] { background: var(--bad); }

    .thermal-labels {
        display: grid;
        grid-template-columns: repeat(4, minmax(0, 1fr));
        gap: 6px;
        font-size: 0.7rem;
        text-align: center;
        color: var(--muted);
    }

    .thermal-labels .current {
        font-weight: 700;
    }
    .thermal-labels .current[data-step="0"] { color: var(--good); }
    .thermal-labels .current[data-step="1"] { color: var(--warn); }
    .thermal-labels .current[data-step="2"] { color: var(--orange); }
    .thermal-labels .current[data-step="3"] { color: var(--bad); }

    .lpm-pill {
        align-self: flex-start;
        padding: 4px 16px;
        border-radius: 999px;
        background: var(--track);
        font-size: 1.125rem;
        font-weight: 600;
    }

    .lpm-pill.on {
        background: var(--warn);
        color: var(--on-warn);
    }

    .head {
        display: flex;
        justify-content: space-between;
        align-items: baseline;
        gap: 12px;
        flex-wrap: wrap;
        margin-bottom: 12px;
    }

    .chart {
        position: relative;
        height: 200px;
    }

    .threshold {
        position: absolute;
        left: 0;
        right: 0;
        border-top: 1px dashed;
        opacity: 0.8;
    }

    .threshold span {
        position: absolute;
        left: 8px;
        top: -18px;
        font-size: 0.7rem;
    }

    .threshold.warn {
        border-color: var(--warn); /* fallback: WebViews without color-mix() (Chrome < 111) */
        border-color: color-mix(in srgb, var(--warn) 55%, transparent);
        color: var(--warn-text);
    }

    .threshold.bad {
        border-color: var(--bad); /* fallback: WebViews without color-mix() (Chrome < 111) */
        border-color: color-mix(in srgb, var(--bad) 55%, transparent);
        color: var(--bad-text);
    }

    .axis {
        display: flex;
        justify-content: space-between;
        margin-top: 6px;
        font-size: 0.7rem;
        color: var(--muted);
    }

    .zones-wrap {
        position: relative;
        padding-top: 20px;
    }

    .zones {
        display: flex;
        height: 14px;
        border-radius: 7px;
        overflow: hidden;
    }

    .zone.bad {
        background: var(--bad); /* fallback: WebViews without color-mix() (Chrome < 111) */
        background: color-mix(in srgb, var(--bad) 60%, transparent);
    }
    .zone.warn {
        background: var(--warn); /* fallback: WebViews without color-mix() (Chrome < 111) */
        background: color-mix(in srgb, var(--warn) 60%, transparent);
    }
    .zone.good {
        background: var(--good); /* fallback: WebViews without color-mix() (Chrome < 111) */
        background: color-mix(in srgb, var(--good) 45%, transparent);
    }

    .marker {
        position: absolute;
        top: 0;
        transform: translateX(-50%);
        display: flex;
        flex-direction: column;
        align-items: center;
        transition: left calc(600ms * var(--m)) var(--spring);
    }

    .marker span {
        font-size: 0.7rem;
        font-weight: 700;
        white-space: nowrap;
    }

    .marker i {
        width: 3px;
        height: 22px;
        border-radius: 2px;
        background: var(--text);
    }

    .zone-labels {
        display: flex;
        margin-top: 12px;
        font-size: 0.75rem;
        font-weight: 600;
    }

    .t-bad {
        color: var(--bad-text);
    }
    .t-warn {
        color: var(--warn-text);
    }
    .t-good {
        color: var(--good-text);
    }
</style>
