<script lang="ts">
    import Card from "./Card.svelte";
    import { maskStart, maskString } from "./utils";
    import type { DeviceInfo } from "tauri-plugin-device-info-api";

    let { info }: { info: DeviceInfo | null } = $props();
</script>

<Card title="Device">
    {#if info}
        <div class="item">
            <strong>Name:</strong>
            <p>{info.device_name}</p>
        </div>
        <div class="item">
            <strong>Model:</strong>
            <p>{info.model}</p>
        </div>
        <div class="item">
            <strong>Manufacturer:</strong>
            <p>{info.manufacturer}</p>
        </div>
        <div class="item">
            <strong>Android ID:</strong>
            <p>{info.android_id || "-"}</p>
        </div>
        <div class="item">
            <strong>Serial:</strong>
            <p>{maskString(info.serial)}</p>
        </div>
        <div class="item">
            <strong>UUID:</strong>
            <p>{maskString(info.uuid)}</p>
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
