import type { PowerProfileReason } from "tauri-plugin-device-info-api";

/** What the background scene is actually rendering, for the HUD. */
export type SceneStats = { particles: number; links: boolean; fps: number };

export const REASON_LABELS: Record<PowerProfileReason, string> = {
    lowPowerMode: "Low Power Mode",
    thermalCritical: "Thermal critical",
    thermalSerious: "Running hot",
    thermalFair: "Warming up",
    batteryCritical: "Battery ≤ 20%",
    batteryLow: "Battery ≤ 40%",
};
