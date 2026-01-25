<script lang="ts">
    import Card from "./Card.svelte";
    import type { DisplayInfo } from "tauri-plugin-device-info-api";

    let { info }: { info: DisplayInfo | null } = $props();
</script>

<Card title="Display">
    {#if info}
        <div class="item">
            <strong>Resolution:</strong>
            <p>{info.width} x {info.height}</p>
        </div>
        <div class="item">
            <strong>Scale Factor:</strong>
            <p>{info.scaleFactor}</p>
        </div>
        <div class="item">
            <strong>Refresh Rate:</strong>
            <p>{info.refreshRate?.toFixed(2)} Hz</p>
        </div>
    {:else}
        <p>Loading...</p>
    {/if}
</Card>

<style>
    .item {
        margin-bottom: 0.8rem;
        font-size: 0.95rem;
        line-height: 1.5;
        color: #e0e0e0;
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 16px;
    }

    .item:last-child {
        margin-bottom: 0;
    }

    .item strong {
        color: #9ca3af;
        font-weight: 500;
    }

    .item p {
        text-align: right;
        margin: 0;
        font-weight: 500;
        color: #ffffff;
    }
</style>
