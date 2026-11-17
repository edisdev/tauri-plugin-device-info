<!-- 270° ring gauge for the device tier score, optionally with the low/mid/high zones marked. -->
<script lang="ts">
    import type { DeviceTierKind } from "tauri-plugin-device-info-api";
    import CountUp from "../ui/CountUp.svelte";

    let {
        score,
        tier,
        size = 168,
        zones = true,
    }: { score: number; tier: DeviceTierKind; size?: number; zones?: boolean } = $props();

    let stroke = $derived(Math.max(6, Math.round(size / 14)));
    let r = $derived((size - stroke) / 2);
    let c = $derived(2 * Math.PI * r);
    let arc = $derived(c * 0.75); // 270° of the circle

    // Zone boundaries match the plugin's thresholds: low < 30 ≤ mid < 65 ≤ high.
    const ZONES = [
        { from: 0, to: 30, color: "var(--bad)" },
        { from: 30, to: 65, color: "var(--warn)" },
        { from: 65, to: 100, color: "var(--good)" },
    ];
    const TIER_COLOR: Record<DeviceTierKind, string> = {
        low: "var(--bad)",
        mid: "var(--warn)",
        high: "var(--good)",
    };

    // Start at 0 and grow to the score once mounted, so the ring animates in.
    let shown = $state(0);
    $effect(() => {
        const target = score;
        const id = requestAnimationFrame(() => (shown = target));
        return () => cancelAnimationFrame(id);
    });
</script>

<div class="gauge" style:width="{size}px" style:height="{size}px" style:--tier-color={TIER_COLOR[tier]}>
    <svg width={size} height={size} viewBox="0 0 {size} {size}" role="img" aria-label="Device tier {tier}, score {score} of 100">
        <g transform="rotate(135 {size / 2} {size / 2})">
            {#if zones}
                {#each ZONES as zone}
                    <circle
                        cx={size / 2}
                        cy={size / 2}
                        {r}
                        fill="none"
                        stroke={zone.color}
                        stroke-opacity="0.18"
                        stroke-width={stroke}
                        stroke-dasharray="{((zone.to - zone.from) / 100) * arc} {c}"
                        stroke-dashoffset={-(zone.from / 100) * arc}
                    />
                {/each}
            {:else}
                <circle
                    cx={size / 2}
                    cy={size / 2}
                    {r}
                    fill="none"
                    stroke="var(--track)"
                    stroke-width={stroke}
                    stroke-dasharray="{arc} {c}"
                />
            {/if}
            <circle
                class="progress"
                cx={size / 2}
                cy={size / 2}
                {r}
                fill="none"
                stroke="var(--tier-color)"
                stroke-width={stroke}
                stroke-linecap="round"
                stroke-dasharray="{(shown / 100) * arc} {c}"
            />
        </g>
    </svg>
    <div class="center">
        <span class="score" style:font-size="{size * 0.26}px"><CountUp value={score} /></span>
        {#if size >= 120}<span class="tier m-pop" style:--d="1100ms">{tier}</span>{/if}
    </div>
</div>

<style>
    .gauge {
        position: relative;
        flex-shrink: 0;
    }

    .progress {
        transition: stroke-dasharray calc(1200ms * var(--m)) var(--ease-out) calc(150ms * var(--m));
    }

    .center {
        position: absolute;
        inset: 0;
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        gap: 4px;
    }

    .score {
        font-weight: 700;
        line-height: 1;
        font-variant-numeric: tabular-nums;
    }

    .tier {
        font-size: 0.75rem;
        font-weight: 600;
        letter-spacing: 0.12em;
        text-transform: uppercase;
        color: var(--tier-color);
    }
</style>
