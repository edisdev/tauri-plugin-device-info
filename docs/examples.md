# Examples

## React Example

```tsx
import { useEffect, useState } from 'react';
import { 
  getDeviceInfo, 
  getBatteryInfo,
  type DeviceInfoResponse,
  type BatteryInfo
} from 'tauri-plugin-device-info-api';

function DeviceInfoCard() {
  const [device, setDevice] = useState<DeviceInfoResponse | null>(null);
  const [battery, setBattery] = useState<BatteryInfo | null>(null);

  useEffect(() => {
    async function loadInfo() {
      const deviceInfo = await getDeviceInfo();
      const batteryInfo = await getBatteryInfo();
      setDevice(deviceInfo);
      setBattery(batteryInfo);
    }
    loadInfo();
  }, []);

  if (!device) return <div>Loading...</div>;

  return (
    <div className="card">
      <h2>{device.deviceName}</h2>
      <p>Model: {device.model}</p>
      <p>Manufacturer: {device.manufacturer}</p>
      {battery && (
        <p>Battery: {battery.level}% {battery.isCharging ? '⚡' : ''}</p>
      )}
    </div>
  );
}
```

## Vue Example

```vue
<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { 
  getDeviceInfo, 
  getStorageInfo,
  type DeviceInfoResponse,
  type StorageInfo
} from 'tauri-plugin-device-info-api';

const device = ref<DeviceInfoResponse | null>(null);
const storage = ref<StorageInfo | null>(null);

onMounted(async () => {
  device.value = await getDeviceInfo();
  storage.value = await getStorageInfo();
});

function formatBytes(bytes: number): string {
  const gb = bytes / (1024 * 1024 * 1024);
  return `${gb.toFixed(1)} GB`;
}
</script>

<template>
  <div v-if="device" class="device-info">
    <h2>{{ device.deviceName }}</h2>
    <p>{{ device.manufacturer }} {{ device.model }}</p>
    
    <div v-if="storage" class="storage">
      <p>Storage: {{ formatBytes(storage.freeSpace) }} free of {{ formatBytes(storage.totalSpace) }}</p>
    </div>
  </div>
</template>
```

## Svelte Example

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import { 
    getDeviceInfo, 
    getBatteryInfo, 
    getNetworkInfo 
  } from 'tauri-plugin-device-info-api';

  let device = null;
  let battery = null;
  let network = null;

  onMount(async () => {
    device = await getDeviceInfo();
    battery = await getBatteryInfo();
    network = await getNetworkInfo();
  });
</script>

{#if device}
  <div class="info-card">
    <h1>{device.deviceName}</h1>
    <p>{device.manufacturer} {device.model}</p>
  </div>
{/if}

{#if battery}
  <div class="battery-card">
    <div class="level" style="width: {battery.level}%"></div>
    <span>{battery.level}%</span>
    {#if battery.isCharging}
      <span class="charging">⚡ Charging</span>
    {/if}
  </div>
{/if}

{#if network}
  <div class="network-card">
    <p>IP: {network.ipAddress}</p>
    <p>Type: {network.networkType}</p>
  </div>
{/if}
```

## Reactive Updates

Prefer the [`watch*` API](/api/watch) over manual polling — it pushes updates on
change (native event-driven on macOS, polling fallback elsewhere) and delivers
the current value immediately.

```typescript
import { watchBattery } from 'tauri-plugin-device-info-api';

// Called immediately with the current value, then on every change.
const stopMonitor = await watchBattery((battery) => {
  console.log(`Battery level: ${battery.level}%`);
});

// Later, when you no longer need updates:
await stopMonitor();
```

::: tip
Only reach for a manual `setInterval` + `getBatteryInfo()` loop if you need a
fixed cadence regardless of change. For "notify me when it changes", `watch*` is
both simpler and cheaper.
:::

## Error Handling

```typescript
import { getDeviceInfo } from 'tauri-plugin-device-info-api';

async function safeGetDeviceInfo() {
  try {
    const device = await getDeviceInfo();
    return device;
  } catch (error) {
    console.error('Failed to get device info:', error);
    return {
      deviceName: 'Unknown Device',
      manufacturer: null,
      model: null,
      uuid: null,
      serial: null,
      android_id: null
    };
  }
}
```
