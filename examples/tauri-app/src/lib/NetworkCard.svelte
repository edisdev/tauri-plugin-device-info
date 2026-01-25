<script lang="ts">
    import Card from "./Card.svelte";
    import { maskIp } from "./utils";
    import type { NetworkInfo } from "tauri-plugin-device-info-api";

    let { info }: { info: NetworkInfo | null } = $props();
</script>

<Card title="Network">
    {#if info}
        <div class="item">
            <strong>IP Address:</strong>
            <p>{maskIp(info.ipAddress)}</p>
        </div>
        <div class="item">
            <strong>Type:</strong>
            <p>{info.networkType}</p>
        </div>
        <div class="item">
            <strong>MAC:</strong>
            <p>{info.macAddress}</p>
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
