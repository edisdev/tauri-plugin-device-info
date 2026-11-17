<!-- The Vitals mark: a heartbeat line on a rounded square. The line draws itself once on load. -->
<script lang="ts">
    let { size = 34, animate = true }: { size?: number; animate?: boolean } = $props();
</script>

<svg width={size} height={size} viewBox="0 0 120 120" role="img" aria-label="Vitals">
    <rect width="120" height="120" rx="27" fill="#30d158" />
    <polyline
        class:m-draw={animate}
        style:--d="0ms"
        points="18,62 40,62 48,44 58,82 68,36 76,62 102,62"
        fill="none"
        stroke="#000000"
        stroke-width="10"
        stroke-linecap="round"
        stroke-linejoin="round"
        pathLength="1"
        stroke-dasharray="1"
    />
</svg>
