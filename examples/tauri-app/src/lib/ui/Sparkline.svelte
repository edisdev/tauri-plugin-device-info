<!--
  Line chart of session samples over [start, end]. The last value is carried
  forward to `end`, so a value that hasn't changed still draws as a flat line.
-->
<script lang="ts">
    import type { Sample } from "../state/device.svelte";

    let {
        samples,
        start,
        end,
        min,
        max,
        color = "var(--good)",
        height = 48,
        area = false,
        label,
    }: {
        samples: Sample[];
        start: number;
        end: number;
        min: number;
        max: number;
        color?: string;
        height?: number;
        area?: boolean;
        label: string;
    } = $props();

    const W = 1000; // viewBox width; the SVG stretches to its container

    let points = $derived.by(() => {
        if (samples.length === 0) return [];
        const until = Math.max(end, samples[samples.length - 1].at);
        const span = Math.max(1, until - start);
        const range = Math.max(1e-9, max - min);
        const toXY = (at: number, value: number) => {
            const x = ((Math.min(Math.max(at, start), until) - start) / span) * W;
            const y = height - 2 - ((value - min) / range) * (height - 4);
            return [x, Math.min(height, Math.max(0, y))] as const;
        };
        const pts = samples.map((s) => toXY(s.at, s.value));
        pts.push(toXY(until, samples[samples.length - 1].value));
        return pts;
    });

    let line = $derived(points.map(([x, y]) => `${x.toFixed(1)},${y.toFixed(1)}`).join(" "));
    let last = $derived(points.at(-1));
</script>

<svg viewBox="0 0 {W} {height}" preserveAspectRatio="none" style:height="{height}px" role="img" aria-label={label}>
    {#if points.length > 0}
        {#if area}
            <polygon class="area" points="0,{height} {line} {W},{height}" fill={color} fill-opacity="0.08" />
        {/if}
        <polyline
            class="m-draw"
            pathLength="1"
            points={line}
            fill="none"
            stroke={color}
            stroke-width="2.5"
            stroke-linejoin="round"
            vector-effect="non-scaling-stroke"
        />
    {/if}
</svg>
{#if last}
    <span class="now-dot" style:top="{last[1]}px" style:background={color}>
        <span class="m-pulse m-loop" style:background={color}></span>
    </span>
{/if}

<style>
    svg {
        display: block;
        width: 100%;
        overflow: visible;
    }

    /*
     * The dash only exists during the draw-in. Kept afterwards (fill "both"),
     * a line that grows with new samples got cut off where the first draw
     * ended — Chromium/WebView2 doesn't re-normalize pathLength with a
     * non-scaling stroke.
     */
    polyline.m-draw {
        animation-fill-mode: backwards;
    }

    .area {
        animation: v-fadeup calc(500ms * var(--m)) var(--ease-out) calc(1100ms * var(--m)) both;
    }

    .now-dot > span {
        position: absolute;
        inset: 0;
        border-radius: 50%;
    }

    .now-dot {
        animation: v-pop calc(420ms * var(--m)) var(--spring) calc(1200ms * var(--m)) both;
        position: absolute;
        right: -4px;
        width: 9px;
        height: 9px;
        margin-top: -4.5px;
        border-radius: 50%;
    }
</style>
