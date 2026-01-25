<script lang="ts">
    import Card from "./Card.svelte";
    import type { BatteryInfo } from "tauri-plugin-device-info-api";

    let { info }: { info: BatteryInfo | null } = $props();
</script>

<Card title="Battery">
    {#if info}
        <div class="item">
            <strong>Level:</strong>
            <p>
                {info.level ? info.level.toFixed(0) + "%" : "N/A"}
            </p>
        </div>
        <div class="item">
            <strong>Charging:</strong>
            <p>{info.isCharging ? "Yes" : "No"}</p>
        </div>
        <div class="item">
            <strong>Health:</strong>
            <p>{info.health}</p>
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
