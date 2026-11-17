<!--
  getPowerProfile / watchPowerProfile / getDeviceTier, explained — and applied:
  the background scene and page-wide effects follow the (possibly simulated) profile.
-->
<script lang="ts">
    import type { PowerProfileKind, ThermalState } from "tauri-plugin-device-info-api";
    import { store } from "../state/device.svelte";
    import { capitalize, formatMemory } from "../state/format";
    import { REASON_LABELS, type SceneStats } from "../adaptive/types";
    import AdaptiveScene from "../adaptive/AdaptiveScene.svelte";
    import TierGauge from "../adaptive/TierGauge.svelte";
    import { theme } from "../state/theme.svelte";
    import { onMount } from "svelte";

    type Mode = "auto" | PowerProfileKind;

    const PROFILES: PowerProfileKind[] = ["full", "balanced", "saver"];
    const THERMAL_LEVELS: ThermalState[] = ["nominal", "fair", "serious", "critical"];
    const PROFILE_HINT: Record<PowerProfileKind, string> = {
        full: "No constraints — running at full quality.",
        balanced: "Some pressure — trimming non-essential work.",
        saver: "Heavy pressure — effects off, doing the minimum.",
    };
    const OPTIONS: { mode: Mode; title: string; sub: string; tint: string }[] = [
        { mode: "auto", title: "My device", sub: "Live signals", tint: "var(--accent-line)" },
        { mode: "full", title: "Full", sub: "All effects on", tint: "var(--good)" },
        { mode: "balanced", title: "Balanced", sub: "Fewer effects · 30 fps", tint: "var(--warn)" },
        { mode: "saver", title: "Saver", sub: "Effects off · frozen", tint: "var(--bad)" },
    ];
    // What *this demo* does per profile — the plugin only recommends; apps decide.
    const CHANGES: [string, string, string, string][] = [
        ["Animations", "On", "Reduced", "Off"],
        ["Background scene", "60 fps + links", "30 fps", "Frozen"],
        ["Blur effects", "On", "On", "Off"],
        ["Background sync", "Every 30 s", "Every 5 min", "Paused"],
    ];

    let stats = $state<SceneStats | null>(null);

    let power = $derived(store.power);
    let tier = $derived(store.tier);
    let effective = $derived(store.effectiveProfile);
    let profileIndex = $derived(PROFILES.indexOf(effective));
    let thermalIndex = $derived(power?.thermalState ? THERMAL_LEVELS.indexOf(power.thermalState) : -1);

    // A profile change sends a color ripple out from the scale — but not on page load.
    let mounted = $state(false);
    onMount(() => {
        const id = setTimeout(() => (mounted = true), 50);
        return () => clearTimeout(id);
    });
</script>

<div class="page adaptive" data-profile={effective}>
    {#if tier}
        <AdaptiveScene profile={effective} tier={tier.tier} theme={theme.current} onstats={(s) => (stats = s)} />
    {/if}

    <header class="page-header">
        <div>
            <h1>Adaptive</h1>
            <p>One signal for how much work to do right now, one for what the hardware can handle.</p>
        </div>
        {#if stats}
            <span class="mono muted hud">
                {stats.particles} particles · {stats.fps > 0 ? `${stats.links ? "links on" : "links off"} · ${stats.fps} fps` : "frozen"}
            </span>
        {/if}
    </header>

    <!-- Try it -->
    <div class="try-it m-card" style:--i="0">
        <div class="try-it-copy">
            <span class="try-it-title">Try it</span>
            <span>Pretend the device is under pressure and watch this app adapt.</span>
        </div>
        <div class="options" role="radiogroup" aria-label="Power profile to preview">
            {#each OPTIONS as option, i}
                <button
                    class="m-pop"
                    style:--d="200ms"
                    style:--i={i}
                    role="radio"
                    aria-checked={store.simulate === option.mode}
                    class:active={store.simulate === option.mode}
                    style:--tint={option.tint}
                    onclick={() => (store.simulate = option.mode)}
                >
                    <span class="option-title"><span class="option-dot"></span>{option.title}</span>
                    <span class="option-sub">{option.sub}</span>
                </button>
            {/each}
        </div>
    </div>

    {#if !power || !tier}
        <p class="soft">Reading power and hardware signals…</p>
    {:else}
        <div class="row">
            <!-- Power profile -->
            <section class="card glass power m-card" style:--i="1" aria-label="Recommended power profile">
                {#key effective}
                    {#if mounted}
                        <span class="ripple" aria-hidden="true" style:left="{16.67 + profileIndex * 33.33}%"></span>
                    {/if}
                {/key}
                <div class="label-row">
                    <span class="label">Recommended power profile</span>
                    {#if store.simulate !== "auto"}<span class="sim-tag">Simulated</span>{/if}
                </div>

                <div class="scale" style:--index={profileIndex}>
                    <div class="rail" aria-hidden="true"><div class="rail-fill"></div></div>
                    <!-- One knob that springs to the active stop, colour crossfading on the way. -->
                    <span class="knob" aria-hidden="true" style:left="{16.67 + profileIndex * 33.33}%"></span>
                    {#each PROFILES as p, i}
                        <div class="stop" class:on={p === effective} class:passed={i < profileIndex}>
                            <span class="stop-dot"></span>
                            <span class="stop-label">{capitalize(p)}</span>
                        </div>
                    {/each}
                </div>
                <div class="scale-ends"><span>Full quality</span><span>Minimum work</span></div>

                <p class="hint">{PROFILE_HINT[effective]}</p>

                <div class="reasons">
                    {#key effective}
                        {#if store.simulate === "auto" && power.reasons.length > 0}
                            {#each power.reasons as reason, i}
                                <span class="chip m-pop" style:--d="150ms" style:--i={i}>{REASON_LABELS[reason]}</span>
                            {/each}
                        {:else if store.simulate === "auto"}
                            <span class="chip ok m-pop" style:--d="150ms">All clear</span>
                        {/if}
                    {/key}
                </div>

                <p class="based-on">Based on these signals from the device</p>
                <div class="signals">
                    <div class="signal">
                        <span class="muted small">Battery</span>
                        <span class="value">
                            {power.batteryLevel !== undefined && power.batteryLevel !== null
                                ? `${Math.round(power.batteryLevel)}% · ${power.isCharging ? "charging" : "discharging"}`
                                : "Mains power"}
                        </span>
                    </div>
                    <div class="signal">
                        <span class="muted small">Thermal</span>
                        <div class="thermal-row">
                            <div class="thermal" aria-hidden="true">
                                {#each THERMAL_LEVELS as _, i}
                                    <span class="bar m-rise" style:--d="500ms" style:--i={i} class:lit={i <= thermalIndex} data-step={i}></span>
                                {/each}
                            </div>
                            <span class="value">{power.thermalState ? capitalize(power.thermalState) : "N/A"}</span>
                        </div>
                    </div>
                    <div class="signal">
                        <span class="muted small">Low Power Mode</span>
                        <span class="value">
                            {power.lowPowerMode === undefined || power.lowPowerMode === null ? "N/A" : power.lowPowerMode ? "On" : "Off"}
                        </span>
                    </div>
                </div>
            </section>

            <!-- Device tier -->
            <section class="card glass m-card" style:--i="2" aria-label="Device tier">
                <span class="label">Device tier</span>
                <p class="caption">
                    Scored 0–100 against other {tier.hardware.isMobile ? "phones" : "desktops"}. Low &lt; 30 · Mid &lt; 65 · High
                </p>
                <div class="tier-body">
                    <TierGauge score={tier.score} tier={tier.tier} size={140} />
                    <dl>
                        <div><dt>CPU</dt><dd>{tier.hardware.cpuModel ?? "—"}</dd></div>
                        <div><dt>Cores</dt><dd>{tier.hardware.cpuCores ?? "—"}</dd></div>
                        <div><dt>Memory</dt><dd>{formatMemory(tier.hardware.totalMemory)}</dd></div>
                        <div><dt>Scale</dt><dd>{tier.hardware.isMobile ? "Mobile" : "Desktop"}</dd></div>
                    </dl>
                </div>
            </section>
        </div>

        <div class="row">
            <!-- What changes -->
            <section class="card glass m-card" style:--i="3" aria-label="What this app changes in each profile">
                <span class="label">What this app changes in each profile</span>
                <div class="table-wrap">
                <!-- The highlight is one block that slides between columns. -->
                <span class="col-highlight" aria-hidden="true" style:left="{34 + profileIndex * 22}%"></span>
                <table>
                    <colgroup><col style="width: 34%" /><col style="width: 22%" /><col style="width: 22%" /><col style="width: 22%" /></colgroup>
                    <thead>
                        <tr>
                            <th scope="col"><span class="visually-hidden">Behavior</span></th>
                            {#each PROFILES as p, i}
                                <th scope="col" class:on={i === profileIndex} data-profile={p}>{capitalize(p)}</th>
                            {/each}
                        </tr>
                    </thead>
                    <tbody>
                        {#each CHANGES as [label, ...cells]}
                            <tr>
                                <th scope="row">{label}</th>
                                {#each cells as cell, i}
                                    <td class:on={i === profileIndex}>{cell}</td>
                                {/each}
                            </tr>
                        {/each}
                    </tbody>
                </table>
                </div>
            </section>

            <!-- Code -->
            <section class="card code m-card" style:--i="4" aria-label="Use it in your app">
                <span class="label">Use it in your app</span>
                <pre class="mono">{`import { watchPowerProfile } from
  'tauri-plugin-device-info-api'

await watchPowerProfile(({ profile }) => {
  document.documentElement
    .dataset.power = profile
})

/* CSS */
[data-power="saver"] * {
  animation: none;
}`}</pre>
            </section>
        </div>
    {/if}
</div>

<style>
    .adaptive {
        --profile-color: var(--good);
        position: relative;
        isolation: isolate;
    }

    .adaptive[data-profile="balanced"] {
        --profile-color: var(--warn);
    }
    .adaptive[data-profile="saver"] {
        --profile-color: var(--bad);
    }

    /* The scene sits behind everything on the page. */
    .adaptive > :global(canvas) {
        position: fixed;
        z-index: -1;
    }

    .hud {
        font-size: 0.75rem;
    }

    .small {
        font-size: 0.75rem;
    }

    .row {
        display: grid;
        grid-template-columns: 1.3fr 1fr;
        gap: 16px;
    }

    @media (max-width: 980px) {
        .row {
            grid-template-columns: 1fr;
        }
    }

    .glass {
        background: var(--surface-glass);
        backdrop-filter: blur(12px);
        -webkit-backdrop-filter: blur(12px);
        display: flex;
        flex-direction: column;
    }

    /* ---- Try it ---- */
    .try-it {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 20px;
        flex-wrap: wrap;
        padding: 12px 12px 12px 18px;
        border-radius: 14px;
        background: var(--surface);
        border: 1px solid var(--track);
    }

    .try-it-copy {
        display: flex;
        flex-direction: column;
        gap: 4px;
        max-width: 280px;
        font-size: 0.82rem;
        line-height: 1.4;
        color: var(--body);
    }

    .try-it-title {
        font-size: 0.75rem;
        font-weight: 700;
        letter-spacing: 0.08em;
        text-transform: uppercase;
        color: var(--accent-soft);
    }

    .options {
        display: flex;
        gap: 8px;
        flex-wrap: wrap;
    }

    .options button {
        all: unset;
        box-sizing: border-box;
        display: flex;
        flex-direction: column;
        align-items: flex-start;
        gap: 3px;
        min-width: 132px;
        min-height: 54px;
        padding: 8px 14px;
        border-radius: 10px;
        cursor: pointer;
        border: 1px solid var(--track);
        background: var(--chip-bg);
        transition: border-color 0.2s, background 0.2s;
    }

    .options button:hover {
        border-color: var(--track-strong);
    }

    .options button.active {
        border-color: var(--tint);
        background: var(--chip-bg); /* fallback: WebViews without color-mix() (Chrome < 111) */
        background: color-mix(in srgb, var(--tint) 14%, transparent);
    }

    .options button:focus-visible {
        outline: 2px solid var(--tint);
        outline-offset: 2px;
    }

    .option-title {
        display: flex;
        align-items: center;
        gap: 7px;
        font-size: 0.88rem;
        font-weight: 600;
        color: var(--text);
    }

    .option-dot {
        width: 8px;
        height: 8px;
        border-radius: 50%;
        background: var(--tint);
    }

    .active .option-dot {
        box-shadow: 0 0 8px var(--tint);
    }

    .option-sub {
        font-size: 0.75rem;
        color: var(--soft);
    }

    @media (max-width: 640px) {
        .try-it {
            flex-direction: column;
            align-items: stretch;
            gap: 12px;
            padding: 14px;
            border-radius: 18px;
        }

        .options {
            display: grid;
            grid-template-columns: repeat(2, minmax(0, 1fr));
        }

        .options button {
            min-width: 0;
            border-radius: 12px;
        }

        /* The snippet is for developers at a desk; on a phone it only scrolls sideways. */
        .card.code {
            display: none;
        }

        /* Signals as label / value rows: three columns don't fit "Nominal" and "Off". */
        .card .signals {
            grid-template-columns: 1fr;
            gap: 8px;
        }

        .card .signal {
            flex-direction: row;
            justify-content: space-between;
            align-items: center;
        }

        .hud {
            display: none;
        }

        th,
        td {
            padding: 8px 6px;
        }
    }

    /* ---- Power profile ---- */
    .label-row {
        display: flex;
        justify-content: space-between;
        align-items: center;
    }

    .sim-tag {
        font-size: 0.7rem;
        font-weight: 700;
        padding: 2px 8px;
        border-radius: 6px;
        color: var(--accent-tag);
        background: var(--accent-bg);
    }

    .scale {
        position: relative;
        display: grid;
        grid-template-columns: repeat(3, 1fr);
        margin: 1.1rem 0 0.4rem;
    }

    .rail {
        position: absolute;
        top: 9px;
        left: 16.67%;
        right: 16.67%;
        height: 4px;
        border-radius: 2px;
        background: var(--track);
    }

    .rail-fill {
        height: 100%;
        width: calc(var(--index) * 50%);
        border-radius: 2px;
        background: var(--profile-color);
        transition: width 0.45s cubic-bezier(0.22, 1, 0.36, 1), background 0.45s;
    }

    .stop {
        position: relative;
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 8px;
    }

    .power {
        position: relative;
        overflow: hidden;
    }

    .power > :not(.ripple) {
        position: relative;
        z-index: 1;
    }

    .ripple {
        position: absolute;
        z-index: 0;
        top: 74px;
        width: 1400px;
        height: 1400px;
        border-radius: 50%;
        background: var(--profile-color);
        pointer-events: none;
        animation: v-ripple calc(900ms * var(--m)) var(--ease-out) both;
    }

    .knob {
        position: absolute;
        top: 0;
        z-index: 1;
        width: 22px;
        height: 22px;
        margin-left: -11px;
        box-sizing: border-box;
        border-radius: 50%;
        border: 4px solid var(--surface);
        background: var(--profile-color);
        box-shadow: 0 0 0 2px var(--profile-color); /* fallback: no color-mix() */
        box-shadow:
            0 0 0 2px var(--profile-color),
            0 0 16px color-mix(in srgb, var(--profile-color) 50%, transparent);
        transition:
            left calc(560ms * var(--m)) var(--spring),
            background calc(400ms * var(--m)),
            box-shadow calc(400ms * var(--m));
    }

    .stop-dot {
        width: 22px;
        height: 22px;
        box-sizing: border-box;
        border-radius: 50%;
        background: var(--surface);
        border: 2px solid var(--track-strong);
        transition: all 0.3s;
    }

    .stop.passed .stop-dot,
    .stop.on .stop-dot {
        background: var(--profile-color);
        border: 4px solid var(--surface);
    }

    .stop.on .stop-dot {
        box-shadow: 0 0 0 2px var(--profile-color); /* fallback: no color-mix() */
        box-shadow:
            0 0 0 2px var(--profile-color),
            0 0 16px color-mix(in srgb, var(--profile-color) 50%, transparent);
    }

    .stop-label {
        font-size: 0.95rem;
        font-weight: 600;
        color: var(--faint);
    }

    .stop.on .stop-label {
        color: var(--profile-color);
    }

    .scale-ends {
        display: flex;
        justify-content: space-between;
        margin-bottom: 0.9rem;
        font-size: 0.7rem;
        color: var(--faint);
    }

    .hint {
        margin: 0 0 0.75rem;
        font-size: 0.88rem;
        color: var(--body);
    }

    .reasons {
        display: flex;
        flex-wrap: wrap;
        gap: 6px;
        min-height: 24px;
        margin-bottom: 1.1rem;
    }

    .chip {
        font-size: 0.75rem;
        font-weight: 600;
        padding: 3px 10px;
        border-radius: 999px;
        color: var(--bad-text);
        background: var(--bad-bg);
        border: 1px solid var(--bad-border);
    }

    .chip.ok {
        color: var(--good-text);
        background: var(--good-bg);
        border-color: var(--good-border);
    }

    .based-on {
        margin: auto 0 0.75rem;
        padding-top: 0.9rem;
        border-top: 1px solid var(--track);
        font-size: 0.75rem;
        color: var(--soft);
    }

    .signals {
        display: grid;
        grid-template-columns: repeat(3, 1fr);
        gap: 0.75rem;
    }

    .signal {
        display: flex;
        flex-direction: column;
        gap: 6px;
        min-width: 0;
    }

    .value {
        font-size: 0.95rem;
        font-weight: 600;
    }

    .thermal-row {
        display: flex;
        align-items: center;
        gap: 10px;
    }

    .thermal {
        display: flex;
        gap: 3px;
        align-items: flex-end;
        height: 16px;
    }

    .thermal .bar {
        width: 7px;
        border-radius: 2px;
        background: var(--track);
    }
    .thermal .bar[data-step="0"] { height: 40%; }
    .thermal .bar[data-step="1"] { height: 60%; }
    .thermal .bar[data-step="2"] { height: 80%; }
    .thermal .bar[data-step="3"] { height: 100%; }
    .thermal .bar.lit[data-step="0"] { background: var(--good); }
    .thermal .bar.lit[data-step="1"] { background: var(--warn); }
    .thermal .bar.lit[data-step="2"] { background: var(--orange); }
    .thermal .bar.lit[data-step="3"] { background: var(--bad); }

    /* ---- Tier ---- */
    .caption {
        margin: 6px 0 0;
        font-size: 0.75rem;
        line-height: 1.45;
        color: var(--soft);
    }

    .tier-body {
        display: flex;
        align-items: center;
        gap: 22px;
        margin-top: 14px;
        flex: 1;
        flex-wrap: wrap;
    }

    dl {
        margin: 0;
        flex: 1;
        min-width: 150px;
        display: grid;
        gap: 10px;
    }

    dl div {
        display: flex;
        justify-content: space-between;
        gap: 12px;
        font-size: 0.875rem;
    }

    dt {
        color: var(--muted);
    }

    dd {
        margin: 0;
        font-weight: 600;
        text-align: right;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
    }

    /* ---- Changes table ---- */
    .table-wrap {
        position: relative;
        margin-top: 12px;
    }

    .col-highlight {
        position: absolute;
        top: 0;
        bottom: 0;
        width: 22%;
        border-radius: 8px;
        background: var(--chip-bg); /* fallback: WebViews without color-mix() (Chrome < 111) */
        background: color-mix(in srgb, var(--profile-color) 12%, transparent);
        transition:
            left calc(450ms * var(--m)) var(--spring),
            background calc(450ms * var(--m));
    }

    table {
        position: relative;
        width: 100%;
        table-layout: fixed;
        border-collapse: collapse;
        font-size: 0.82rem;
    }

    th,
    td {
        padding: 8px 10px;
        text-align: left;
    }

    thead th {
        font-weight: 700;
        color: var(--faint);
        border-radius: 8px 8px 0 0;
    }

    thead th.on {
        color: var(--profile-color);
    }

    tbody th {
        padding-left: 0;
        font-weight: 400;
        color: var(--soft);
        border-top: 1px solid var(--divider);
    }

    td {
        color: var(--faint);
        border-top: 1px solid var(--divider);
    }

    td.on {
        color: var(--text);
        font-weight: 600;
    }

    .visually-hidden {
        position: absolute;
        width: 1px;
        height: 1px;
        overflow: hidden;
        clip: rect(0 0 0 0);
        white-space: nowrap;
    }

    /* ---- Code ---- */
    .code {
        display: flex;
        flex-direction: column;
        gap: 10px;
        color: var(--code-text);
        background: var(--code-bg);
        border-color: var(--track);
    }

    .code .label {
        color: var(--muted); /* fallback: WebViews without color-mix() (Chrome < 111) */
        color: color-mix(in srgb, var(--code-text) 65%, transparent);
    }

    pre {
        margin: 0;
        font-size: 0.75rem;
        line-height: 1.7;
        color: var(--code-text);
        overflow-x: auto;
    }
</style>
