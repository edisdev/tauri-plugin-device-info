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