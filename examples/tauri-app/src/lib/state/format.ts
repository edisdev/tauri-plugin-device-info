/** Formatting, masking and platform helpers shared by the pages. */

export type Page = "overview" | "adaptive" | "power" | "system";

export type Platform = "macOS" | "Windows" | "Linux" | "iOS" | "Android";

export function detectPlatform(): Platform {
    const ua = navigator.userAgent;
    if (/Android/i.test(ua)) return "Android";
    if (/iPhone|iPad|iPod/i.test(ua)) return "iOS";
    // iPadOS reports a desktop Mac user agent; touch points give it away.
    if (/Mac/i.test(ua)) return navigator.maxTouchPoints > 1 ? "iOS" : "macOS";
    if (/Win/i.test(ua)) return "Windows";
    return "Linux";
}

export const platform = detectPlatform();

/** Apple platforms count storage in powers of 1000, Windows/Linux tools in 1024. */
const GB = platform === "macOS" || platform === "iOS" ? 1e9 : 1024 ** 3;

/** Battery health as shown: desktop reports a percentage ("100.0" → "100%"), mobile a word ("Good"). */
export function formatHealth(health: string | undefined | null): string {
    if (!health) return "—";
    const n = Number(health);
    return Number.isFinite(n) ? `${Math.round(n)}%` : health;
}

export function gb(bytes: number | undefined | null): number {
    return bytes ? bytes / GB : 0;
}

export function formatGb(bytes: number | undefined | null): string {
    if (!bytes) return "—";
    const value = gb(bytes);
    return value >= 1000 ? `${(value / 1000).toFixed(value >= 10_000 ? 0 : 1)} TB` : `${Math.round(value)} GB`;
}

/** RAM is always counted in binary units: 24 GiB is "24 GB" on every OS. */
export function formatMemory(bytes: number | undefined | null): string {
    return bytes ? `${Math.round(bytes / 1024 ** 3)} GB` : "—";
}

/** "Ssd" / "Hdd" from the backend → "SSD" / "HDD"; anything else capitalized. */
export function storageLabel(type: string | undefined | null): string {
    if (!type) return "Disk";
    return /^(ssd|hdd)$/i.test(type) ? type.toUpperCase() : capitalize(type);
}

export function formatTime(at: number | Date): string {
    return new Date(at).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
}

export function capitalize(s: string | undefined | null): string {
    return s ? s.charAt(0).toUpperCase() + s.slice(1) : "";
}

/** Network type as people say it: "wifi" → "Wi-Fi". */
export function networkLabel(type: string | undefined | null): string {
    switch (type) {
        case "wifi":
            return "Wi-Fi";
        case "ethernet":
            return "Ethernet";
        case "cellular":
            return "Cellular";
        case "no_connection":
            return "Offline";
        case undefined:
        case null:
        case "":
        case "unknown":
            return "Unknown";
        default:
            return type;
    }
}

/** Keeps only the last `keep` characters: "••••••••4E2A". */
export function maskTail(value: string | undefined | null, keep = 4): string {
    if (!value) return "—";
    if (value.length <= keep) return value;
    return "•".repeat(Math.min(8, value.length - keep)) + value.slice(-keep);
}

/** Hides a MAC address entirely, keeping only its shape: "••:••:••:••:••:••". */
export function maskMac(mac: string | undefined | null): string {
    if (!mac) return "—";
    const parts = mac.split(":");
    if (parts.length !== 6) return mac; // "restricted" / "unavailable" on mobile
    return parts.map(() => "••").join(":");
}

/** Keeps the network part of an IPv4 address ("192.168.•.•"); IPv6 keeps its first group. */
export function maskIp(ip: string | undefined | null): string {
    if (!ip) return "—";
    if (ip.includes(".")) {
        const octets = ip.split(".");
        return [...octets.slice(0, 2), "•", "•"].join(".");
    }
    return `${ip.split(":")[0]}:••••`;
}

export type WatchKind = "battery" | "network" | "storage" | "display" | "device" | "power";

/**
 * How the plugin watches each kind on this platform — mirrors the backend:
 * macOS has native event sources for battery, display and network; everything
 * else falls back to change-detecting polling with these default intervals.
 */
export function watchMode(kind: WatchKind): { native: boolean; label: string } {
    const NATIVE_MAC: Partial<Record<WatchKind, string>> = {
        battery: "IOKit",
        display: "Core Graphics",
        network: "SystemConfiguration",
    };
    if (platform === "macOS" && NATIVE_MAC[kind]) {
        return { native: true, label: `Native event · ${NATIVE_MAC[kind]}` };
    }
    const seconds = { storage: 10, device: 60, power: 5 }[kind as "storage" | "device" | "power"] ?? 2;
    return {
        native: false,
        label: kind === "storage" ? `Polled every ${seconds} s — no OS event exists` : `Polled every ${seconds} s`,
    };
}
