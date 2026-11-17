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

/** Thermal pressure on Apple's four-level scale (other platforms are mapped onto it). */
export type ThermalState = 'nominal' | 'fair' | 'serious' | 'critical';

/**
 * Recommended amount of work for the app right now:
 * - `full` — no constraints, run at full quality
 * - `balanced` — some pressure; skip non-essential work, lower animation/refresh rates
 * - `saver` — heavy pressure; do the minimum, pause background work and effects
 */
export type PowerProfileKind = 'full' | 'balanced' | 'saver';

/**
 * Why the profile is not `full`:
 * - `lowPowerMode` — OS power saving is on (→ saver)
 * - `thermalCritical` / `thermalSerious` — device is hot (→ saver)
 * - `thermalFair` — device is warm (→ balanced)
 * - `batteryCritical` — on battery at ≤ 20% (→ saver)
 * - `batteryLow` — on battery at ≤ 40% (→ balanced)
 *
 * "On battery" means not charging and not on external power; an unknown or
 * implausible battery level never triggers a battery reason.
 */
export type PowerProfileReason =
  | 'lowPowerMode'
  | 'thermalCritical'
  | 'thermalSerious'
  | 'thermalFair'
  | 'batteryCritical'
  | 'batteryLow';

export interface PowerProfile {
  /** The recommended profile — the most restrictive one implied by any reason. */
  profile: PowerProfileKind;
  /** Every condition pushing the profile below `full` (empty when `full`). */
  reasons: PowerProfileReason[];
  /** Battery level 0-100; absent on devices without a battery or when the level is unknown. */
  batteryLevel?: number;
  isCharging?: boolean;
  /** Low Power Mode / Battery Saver / `low-power` profile; absent if the OS doesn't expose it. */
  lowPowerMode?: boolean;
  /** Absent where the OS has no thermal API (Windows). */
  thermalState?: ThermalState;
  /**
   * Running on external power (AC adapter, USB-C charger), even when the
   * battery isn't charging — e.g. full or held at a charge limit. Battery
   * reasons never apply while this is `true`. Absent when the OS doesn't say.
   */
  onExternalPower?: boolean;
}

export interface HardwareInfo {
  /** Logical CPU cores. */
  cpuCores?: number;
  /** Physical CPU cores (desktop only). */
  physicalCores?: number;
  /** CPU / SoC name, e.g. "Apple M2" or "iPhone15,2". */
  cpuModel?: string;
  /** Peak CPU clock in MHz, where available (not on iOS or Apple Silicon). */
  cpuMaxFrequencyMhz?: number;
  /** Total physical memory in bytes. */
  totalMemory?: number;
  /** Whether the tier was scored against mobile hardware. */
  isMobile: boolean;
  /** Android only: the OS flags this as a low-RAM device (forces tier `low`). */
  isLowRamDevice?: boolean;
  /** Android 12+ only: declared media performance class (31+ → at least `mid`, 33+ → `high`). */
  mediaPerformanceClass?: number;
}

export type DeviceTierKind = 'low' | 'mid' | 'high';

export interface DeviceTier {
  /** Capability class relative to devices of the same form factor. */
  tier: DeviceTierKind;
  /** 0 (weakest) – 100 (strongest). `low` < 30 ≤ `mid` < 65 ≤ `high`. */
  score: number;
  /** The hardware signals the score was computed from. */
  hardware: HardwareInfo;
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
   * - `power`: defaults to 5000ms, floored at 1000ms
   * - others: default 2000ms, floored at 250ms
   *
   * Only the first subscriber for a given kind sets the interval.
   */
  intervalMs?: number;
}

/** Stops a watcher and removes its listener. Returned by every `watch*` function. */
export type UnwatchFn = () => Promise<void>;