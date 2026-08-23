export interface DeviceInfoResponse {
  uuid?: string;
  manufacturer?: string;
  model?: string;
  serial?: string;
  android_id?: string;
  device_name?: string;
}

export interface BatteryInfo {
  level?: number;
  isCharging?: boolean;
  health?: string;
}

export interface NetworkInfo {
  ipAddress?: string;
  networkType?: string;
  macAddress?: string;
}

export interface StorageInfo {
  totalSpace?: number;
  freeSpace?: number;
  storageType?: string;
}

export interface DisplayInfo {
  width?: number;
  height?: number;
  scaleFactor?: number;
  refreshRate?: number;
}

/** Options for the reactive `watch*` APIs. */
export interface WatchOptions {
  /**
   * Polling interval in milliseconds used to detect changes (only applies to
   * kinds that fall back to polling; native event-driven kinds ignore it).
   *
   * Defaults and minimums are per kind, because some getters are expensive and
   * some values rarely change:
   * - `device`: defaults to 60000ms, floored at 10000ms (reads are costly and it barely changes)
   * - `storage`: defaults to 10000ms, floored at 1000ms
   * - others: default 2000ms, floored at 250ms
   *
   * Only the first subscriber for a given kind sets the interval.
   */
  intervalMs?: number;
}

/** Stops a watcher and removes its listener. Returned by every `watch*` function. */
export type UnwatchFn = () => Promise<void>;