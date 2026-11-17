/**
 * The demo's color theme. Until the person picks one, it follows the system's
 * light/dark setting; an explicit pick is remembered across launches.
 */
export type Theme = "light" | "dark" | "colorful";

export const THEMES: { id: Theme; label: string; swatch: [string, string] }[] = [
    { id: "light", label: "Light", swatch: ["#fffcf7", "#d9b48c"] },
    { id: "dark", label: "Dark", swatch: ["#000000", "#30d158"] },
    { id: "colorful", label: "Colorful", swatch: ["#1a0f36", "#ec4899"] },
];

const KEY = "device-info-demo:theme";

function readSaved(): Theme | null {
    try {
        const saved = localStorage.getItem(KEY);
        return THEMES.some((t) => t.id === saved) ? (saved as Theme) : null;
    } catch {
        return null; // storage can be unavailable; the system theme still works
    }
}

const lightQuery = window.matchMedia("(prefers-color-scheme: light)");

class ThemeStore {
    choice = $state<Theme | null>(readSaved());
    system = $state<Theme>(lightQuery.matches ? "light" : "dark");
    current = $derived<Theme>(this.choice ?? this.system);

    constructor() {
        lightQuery.addEventListener("change", (e) => (this.system = e.matches ? "light" : "dark"));
    }

    set(theme: Theme) {
        this.choice = theme;
        try {
            localStorage.setItem(KEY, theme);
        } catch {
            // Not persisted this time; the choice still applies for the session.
        }
    }

    /** Next theme in order — used by the compact phone switcher. */
    cycle() {
        const i = THEMES.findIndex((t) => t.id === this.current);
        this.set(THEMES[(i + 1) % THEMES.length].id);
    }
}

export const theme = new ThemeStore();
