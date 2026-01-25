<script lang="ts">
    import Card from "./Card.svelte";
    import type { StorageInfo } from "tauri-plugin-device-info-api";

    let { info }: { info: StorageInfo | null } = $props();

    // Dynamically set divisor based on OS (Mac uses decimal (1000), Windows uses binary (1024))
    const isMac = navigator.userAgent.includes("Mac");
    const DIVISOR = isMac ? 1000 : 1024;

    function formatSize(bytes: number | undefined) {
        if (bytes === undefined || bytes === null) return "N/A";
        return (bytes / DIVISOR / DIVISOR / DIVISOR).toFixed(2) + " GB";
    }
</script>

<Card title="Storage">
    {#if info}
        <div class="item">
            <strong>Total:</strong>
            <p>{formatSize(info.totalSpace)}</p>
        </div>
        <div class="item">
            <strong>Free:</strong>
            <p>{formatSize(info.freeSpace)}</p>
        </div>
        <div class="item">
            <strong>Type:</strong>
            <p>{info.storageType || "Unknown"}</p>
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
