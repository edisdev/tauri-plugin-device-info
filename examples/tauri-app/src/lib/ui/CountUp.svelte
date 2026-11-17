<!--
  A number that counts up to its value on first render (1.2 s) and glides to
  each new value (0.5 s). In Saver / reduced motion it just shows the value.
-->
<script lang="ts">
    import { untrack } from "svelte";

    let { value, decimals = 0 }: { value: number | null | undefined; decimals?: number } = $props();

    let shown = $state(0);
    let first = true;

    const still = () =>
        document.documentElement.dataset.power === "saver" ||
        window.matchMedia("(prefers-reduced-motion: reduce)").matches;

    $effect(() => {
        const target = value ?? 0;
        const from = first ? 0 : untrack(() => shown);
        const balanced = document.documentElement.dataset.power === "balanced";
        const duration = (first ? 1200 : 500) * (balanced ? 0.5 : 1);
        first = false;

        if (still() || from === target) {
            shown = target;
            return;
        }

        const start = performance.now();
        let raf = 0;
        const tick = (now: number) => {
            const t = Math.min(1, (now - start) / duration);
            shown = from + (target - from) * (1 - Math.pow(1 - t, 3));
            if (t < 1) raf = requestAnimationFrame(tick);
        };
        raf = requestAnimationFrame(tick);
        return () => cancelAnimationFrame(raf);
    });
</script>

<span class="count">{shown.toFixed(decimals)}</span>

<style>
    .count {
        font-variant-numeric: tabular-nums;
    }
</style>
