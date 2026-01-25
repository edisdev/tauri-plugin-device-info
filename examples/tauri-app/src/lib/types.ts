export interface DeviceInfo {
    device_name?: string;
    model?: string;
    manufacturer?: string;
    android_id?: string;
    serial?: string;
    uuid?: string;
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
