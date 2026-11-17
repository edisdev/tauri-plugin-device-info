/**
 * One shared, reactive view of the device for every page.
 *
 * Watchers are started once for the whole app. Besides the latest value of
 * each kind, the store keeps what the pages chart: an event log and per-kind
 * history, all built from real `watch*` events since the app started.
 */
import {
    getDeviceInfo,
    getDeviceTier,
    watchBattery,
    watchNetwork,
    watchStorage,
    watchDisplay,
    watchPowerProfile,
} from "tauri-plugin-device-info-api";
import type {
    DeviceInfo,
    BatteryInfo,
    NetworkInfo,
    StorageInfo,
    DisplayInfo,
    PowerProfile,
    PowerProfileKind,
    DeviceTier,
    UnwatchFn,
} from "tauri-plugin-device-info-api";
import { capitalize, formatGb, gb, networkLabel, type WatchKind } from "./format";
import { REASON_LABELS } from "../adaptive/types";

export type LogEvent = { id: number; at: number; kind: WatchKind; api: string; text: string };
export type Sample = { at: number; value: number };
export type NetworkSegment = { at: number; type: string };

const MAX_EVENTS = 50;
const MAX_SAMPLES = 720;

class DeviceStore {
    device = $state<DeviceInfo | null>(null);
    tier = $state<DeviceTier | null>(null);
    battery = $state<BatteryInfo | null>(null);
    network = $state<NetworkInfo | null>(null);
    storage = $state<StorageInfo | null>(null);
    display = $state<DisplayInfo | null>(null);
    power = $state<PowerProfile | null>(null);

    events = $state<LogEvent[]>([]);
    batteryHistory = $state<Sample[]>([]);
    storageHistory = $state<Sample[]>([]);
    networkHistory = $state<NetworkSegment[]>([]);
    lastChange = $state<Partial<Record<WatchKind, { at: number; text: string }>>>({});
    error = $state<string | null>(null);

    readonly startedAt = Date.now();
    /** Ticks so session charts keep extending to "now" between events. */
    now = $state(Date.now());

    /**
     * IP, MAC, device name and hardware IDs are masked everywhere until the
     * person presses Show (System page), so screenshots and recordings of the
     * demo don't leak them.
     */
    reveal = $state(false);

    /** "auto" follows the device; anything else previews that profile app-wide. */
    simulate = $state<"auto" | PowerProfileKind>("auto");
    effectiveProfile = $derived<PowerProfileKind>(
        this.simulate === "auto" ? (this.power?.profile ?? "full") : this.simulate,
    );

    #unwatchers: UnwatchFn[] = [];
    #ticker: ReturnType<typeof setInterval> | undefined;
    #nextId = 0;
    #started = false;
    /** Bumped by every stop(), so a start() still awaiting can tell it is stale. */
    #generation = 0;

    async start() {
        if (this.#started) return;
        this.#started = true;
        const generation = ++this.#generation;
        const live = () => generation === this.#generation;
        this.#ticker = setInterval(() => (this.now = Date.now()), 15_000);

        // Everything starts independently: one failing read or watcher must not
        // keep the others from starting, or leak the ones that did.
        // Device info and tier are static for the session: read once. Everything
        // else is pushed: the first callback is the current value.
        const guard =
            <T>(handler: (value: T) => void) =>
            (value: T) => {
                if (live()) handler(value);
            };
        const watchers: [string, Promise<UnwatchFn>][] = [
            ["watchBattery", watchBattery(guard((info) => this.#onBattery(info)))],
            ["watchNetwork", watchNetwork(guard((info) => this.#onNetwork(info)))],
            ["watchStorage", watchStorage(guard((info) => this.#onStorage(info)))],
            ["watchDisplay", watchDisplay(guard((info) => this.#onDisplay(info)))],
            ["watchPowerProfile", watchPowerProfile(guard((info) => this.#onPower(info)))],
        ];

        const [device, tier, ...started] = await Promise.allSettled([
            getDeviceInfo(),
            getDeviceTier(),
            ...watchers.map(([, promise]) => promise),
        ]);

        const unwatchers: UnwatchFn[] = [];
        const errors: string[] = [];
        started.forEach((result, i) => {
            if (result.status === "fulfilled") unwatchers.push(result.value as UnwatchFn);
            else errors.push(`${watchers[i][0]}: ${result.reason}`);
        });

        // stop() ran while we were awaiting (HMR, remount): undo what this
        // start() created instead of handing it to the store.
        if (!live()) {
            unwatchers.forEach((unwatch) => unwatch().catch(() => {}));
            return;
        }

        if (device.status === "fulfilled") this.device = device.value as DeviceInfo;
        else errors.push(`getDeviceInfo: ${device.reason}`);
        if (tier.status === "fulfilled") this.tier = tier.value as DeviceTier;
        else errors.push(`getDeviceTier: ${tier.reason}`);

        this.#unwatchers = unwatchers;
        this.error = errors.length ? errors.join("; ") : null;
    }

    stop() {
        this.#generation++;
        clearInterval(this.#ticker);
        this.#ticker = undefined;
        this.#unwatchers.forEach((unwatch) => unwatch().catch(() => {}));
        this.#unwatchers = [];
        this.#started = false;
    }

    #log(kind: WatchKind, api: string, text: string) {
        const at = Date.now();
        this.now = at;
        this.events.unshift({ id: this.#nextId++, at, kind, api, text });
        if (this.events.length > MAX_EVENTS) this.events.pop();
        this.lastChange[kind] = { at, text };
    }

    #sample(history: Sample[], value: number | undefined | null) {
        if (value === undefined || value === null) return;
        const at = Date.now();
        this.now = at;
        history.push({ at, value });
        if (history.length > MAX_SAMPLES) history.shift();
    }

    #onBattery(info: BatteryInfo) {
        const prev = this.battery;
        this.battery = info;
        this.#sample(this.batteryHistory, info.level);
        if (!prev || info.level === undefined) return;

        const level = Math.round(info.level);
        if (prev.isCharging !== info.isCharging) {
            this.#log("battery", "watchBattery", `Charger ${info.isCharging ? "connected" : "disconnected"} at ${level}%`);
        } else if (Math.round(prev.level ?? -1) !== level) {
            this.#log("battery", "watchBattery", `Battery ${info.isCharging ? "charged" : "drained"} to ${level}%`);
        }
    }

    #onNetwork(info: NetworkInfo) {
        const prev = this.network;
        this.network = info;
        const type = info.networkType ?? "unknown";
        if (this.networkHistory.at(-1)?.type !== type) {
            this.now = Date.now();
            this.networkHistory.push({ at: this.now, type });
        }
        if (!prev) return;

        if (prev.networkType !== info.networkType) {
            this.#log(
                "network",
                "watchNetwork",
                `${networkLabel(prev.networkType)} → ${networkLabel(info.networkType)}`,
            );
        } else if (prev.ipAddress !== info.ipAddress) {
            this.#log("network", "watchNetwork", "IP address changed");
        }
    }

    #onStorage(info: StorageInfo) {
        const prev = this.storage;
        this.storage = info;
        this.#sample(this.storageHistory, info.freeSpace);
        if (!prev) return;

        // Free space jitters constantly; only log changes a person would notice.
        const delta = gb(info.freeSpace) - gb(prev.freeSpace);
        if (Math.abs(delta) >= 1) {
            this.#log(
                "storage",
                "watchStorage",
                `Free space ${delta < 0 ? "dropped" : "grew"} to ${formatGb(info.freeSpace)}`,
            );
        }
    }

    #onDisplay(info: DisplayInfo) {
        const prev = this.display;
        this.display = info;
        if (!prev) return;
        this.#log(
            "display",
            "watchDisplay",
            `Display changed to ${info.width} × ${info.height}${info.refreshRate ? ` @ ${Math.round(info.refreshRate)} Hz` : ""}`,
        );
    }

    #onPower(info: PowerProfile) {
        const prev = this.power;
        this.power = info;
        if (!prev || prev.profile === info.profile) return;
        const why = info.reasons.map((r) => REASON_LABELS[r]).join(", ") || "all clear";
        this.#log("power", "watchPowerProfile", `${capitalize(prev.profile)} → ${capitalize(info.profile)}: ${why}`);
    }
}

export const store = new DeviceStore();
