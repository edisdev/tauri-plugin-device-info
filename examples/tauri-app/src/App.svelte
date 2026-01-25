<script lang="ts">
  import {
    getDeviceInfo,
    getBatteryInfo,
    getNetworkInfo,
    getStorageInfo,
    getDisplayInfo,
  } from "tauri-plugin-device-info-api";

  import type {
    DeviceInfo,
    BatteryInfo,
    NetworkInfo,
    StorageInfo,
    DisplayInfo,
  } from "tauri-plugin-device-info-api";
  import { onMount, onDestroy } from "svelte";

  import Header from "./lib/Header.svelte";
  import ChargingOverlay from "./lib/ChargingOverlay.svelte";
  import DeviceCard from "./lib/DeviceCard.svelte";
  import BatteryCard from "./lib/BatteryCard.svelte";
  import NetworkCard from "./lib/NetworkCard.svelte";
  import StorageCard from "./lib/StorageCard.svelte";
  import DisplayCard from "./lib/DisplayCard.svelte";

  let deviceInfo = $state<DeviceInfo | null>(null);
  let batteryInfo = $state<BatteryInfo | null>(null);
  let networkInfo = $state<NetworkInfo | null>(null);
  let storageInfo = $state<StorageInfo | null>(null);
  let displayInfo = $state<DisplayInfo | null>(null);
  let error = $state<string | null>(null);
  let loading = $state(false);

  let showChargingIcon = $state(false);
  let lastChargingState = false;
  let pollingInterval: number | undefined;

  async function fetchData() {
    loading = true;
    error = null;
    try {
      deviceInfo = await getDeviceInfo();
      batteryInfo = await getBatteryInfo();
      networkInfo = await getNetworkInfo();
      storageInfo = await getStorageInfo();
      displayInfo = await getDisplayInfo();

      // Initialize tracking state
      if (batteryInfo) {
        lastChargingState = batteryInfo.isCharging || false;
      }
    } catch (e: any) {
      error = e.toString();
    } finally {
      loading = false;
    }
  }

  function startBatteryPolling() {
    pollingInterval = setInterval(async () => {
      try {
        const info = await getBatteryInfo();

        // Check if charging started (transition from false to true)
        if (info.isCharging && !lastChargingState) {
          showChargingIcon = true;
          setTimeout(() => {
            showChargingIcon = false;
          }, 5000);
        }

        lastChargingState = info.isCharging || false;
        batteryInfo = info; // Update UI with latest info
      } catch (e) {
        console.error("Battery polling error:", e);
      }
    }, 1000); // Check every 1 second
  }

  onMount(() => {
    fetchData();
    startBatteryPolling();
  });

  onDestroy(() => {
    if (pollingInterval) clearInterval(pollingInterval);
  });
</script>

<ChargingOverlay show={showChargingIcon} />
<Header {loading} {fetchData} />

<div class="container">
  {#if error}
    <div class="error">
      <strong>Error:</strong>
      {error}
    </div>
  {/if}

  <div class="grid">
    <DeviceCard info={deviceInfo} />
    <BatteryCard info={batteryInfo} />
    <NetworkCard info={networkInfo} />
    <StorageCard info={storageInfo} />
    <DisplayCard info={displayInfo} />
  </div>
</div>

<style>
  :global(body) {
    margin: 0;
    font-family:
      "Inter",
      -apple-system,
      BlinkMacSystemFont,
      "Segoe UI",
      Roboto,
      Helvetica,
      Arial,
      sans-serif;
    background: linear-gradient(135deg, #121212 0%, #1e1e2e 100%);
    color: rgba(255, 255, 255, 0.92);
    min-height: 100vh;
  }

  .container {
    padding: 2rem;
    min-height: 100vh;
    box-sizing: border-box;
    display: flex;
    flex-direction: column;
  }

  @media (min-width: 768px) {
    .container {
      padding: 4rem;
    }
  }

  .error {
    background-color: rgba(69, 26, 26, 0.5);
    color: #ff8080;
    padding: 1rem;
    border-radius: 8px;
    margin-bottom: 2rem;
    border: 1px solid rgba(125, 42, 42, 0.5);
    backdrop-filter: blur(10px);
  }

  .grid {
    display: flex;
    flex-wrap: wrap;
    gap: 2rem;
    width: 100%;
    justify-content: flex-start;
  }

  @media (min-width: 1200px) {
    .grid {
      justify-content: center;
      max-width: 1400px;
      margin: 0 auto;
    }
  }
</style>
