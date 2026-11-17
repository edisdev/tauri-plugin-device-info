<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import type { PowerProfileKind } from "tauri-plugin-device-info-api";

  import { store } from "./lib/state/device.svelte";
  import { theme } from "./lib/state/theme.svelte";
  import { capitalize, type Page } from "./lib/state/format";
  import { REASON_LABELS } from "./lib/adaptive/types";
  import Sidebar from "./lib/layout/Sidebar.svelte";
  import ChargingOverlay from "./lib/ChargingOverlay.svelte";
  import Overview from "./lib/pages/Overview.svelte";
  import Adaptive from "./lib/pages/Adaptive.svelte";
  import Power from "./lib/pages/Power.svelte";
  import System from "./lib/pages/System.svelte";

  const PAGES: Page[] = ["overview", "adaptive", "power", "system"];

  // Hash routing (#/power) — no router dependency, works in the Tauri webview.
  const pageFromHash = (): Page => {
    const id = location.hash.replace(/^#\/?/, "") as Page;
    return PAGES.includes(id) ? id : "overview";
  };
  let page = $state<Page>(pageFromHash());

  onMount(() => {
    const onHash = () => {
      page = pageFromHash();
      window.scrollTo(0, 0);
    };
    window.addEventListener("hashchange", onHash);
    store.start();
    return () => {
      window.removeEventListener("hashchange", onHash);
      store.stop();
    };
  });

  // The whole app adapts: [data-power="saver"] in style.css drops blur and animations.
  $effect(() => {
    document.documentElement.dataset.power = store.effectiveProfile;
  });

  // Theme tokens live on <html data-theme> (see style.css).
  $effect(() => {
    document.documentElement.dataset.theme = theme.current;
    // Native title bar too (Windows keeps it white in a dark app otherwise).
    getCurrentWindow()
      .setTheme(theme.current === "light" ? "light" : "dark")
      .catch(() => {}); // not supported on mobile
  });

  // Announce profile changes (not the initial value). The effect re-runs on
  // every power update, so the hide timer lives outside it.
  let toast = $state<{ profile: PowerProfileKind; detail: string } | null>(null);
  let previousProfile: PowerProfileKind | null = null;
  let toastTimer: ReturnType<typeof setTimeout> | undefined;
  $effect(() => {
    const current = store.effectiveProfile;
    const power = store.power;
    if (!power) return;
    if (previousProfile !== null && previousProfile !== current) {
      toast = {
        profile: current,
        detail:
          store.simulate !== "auto"
            ? "Simulated"
            : power.reasons.map((r) => REASON_LABELS[r]).join(" · ") || "All clear",
      };
      clearTimeout(toastTimer);
      toastTimer = setTimeout(() => (toast = null), 3500);
    }
    previousProfile = current;
  });

  // Celebrate a real false → true charging transition (never the initial value).
  let showCharging = $state(false);
  let wasCharging: boolean | null = null;
  let chargingTimer: ReturnType<typeof setTimeout> | undefined;
  $effect(() => {
    const charging = store.battery?.isCharging;
    if (charging === undefined) return;
    if (wasCharging === false && charging) {
      showCharging = true;
      clearTimeout(chargingTimer);
      chargingTimer = setTimeout(() => (showCharging = false), 4000);
    }
    wasCharging = charging;
  });

  $effect(() => () => {
    clearTimeout(toastTimer);
    clearTimeout(chargingTimer);
  });
</script>

<ChargingOverlay show={showCharging} />

<div class="shell">
  <Sidebar {page} />

  <main>
    {#if store.error}
      <div class="error" role="alert"><strong>Error:</strong> {store.error}</div>
    {/if}

    {#if page === "overview"}
      <Overview />
    {:else if page === "adaptive"}
      <Adaptive />
    {:else if page === "power"}
      <Power />
    {:else}
      <System />
    {/if}
  </main>
</div>

{#if toast}
  <div class="toast" data-profile={toast.profile} role="status">
    <strong>Switched to {capitalize(toast.profile)}</strong>
    <span>{toast.detail}</span>
  </div>
{/if}

<style>
  .shell {
    display: flex;
    min-height: 100vh;
  }

  main {
    flex: 1;
    min-width: 0;
    max-width: 1400px;
    padding: 28px 32px 40px;
    padding-top: calc(env(safe-area-inset-top) + 28px);
  }

  @media (max-width: 820px) {
    main {
      padding: 20px 16px 110px;
      padding-top: calc(env(safe-area-inset-top) + 20px);
    }
  }

  .error {
    margin-bottom: 16px;
    padding: 12px 16px;
    border-radius: 10px;
    background: var(--bad-bg);
    border: 1px solid var(--bad-border);
    color: var(--bad-text);
  }

  .toast {
    --toast-color: var(--good);
    position: fixed;
    right: 24px;
    bottom: 24px;
    z-index: 50;
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 12px 16px;
    border-radius: 12px;
    background: var(--toast-bg);
    border: 1px solid var(--toast-color);
    box-shadow: 0 8px 28px rgba(0, 0, 0, 0.45);
    animation: toast-in calc(380ms * var(--m)) var(--spring) both;
  }

  @media (max-width: 820px) {
    .toast {
      left: 16px;
      right: 16px;
      bottom: calc(env(safe-area-inset-bottom) + 84px);
    }
  }

  .toast[data-profile="balanced"] {
    --toast-color: var(--warn);
  }
  .toast[data-profile="saver"] {
    --toast-color: var(--bad);
  }

  .toast strong {
    font-size: 0.9rem;
    color: var(--toast-color);
  }

  .toast span {
    font-size: 0.78rem;
    color: var(--body);
  }

  @keyframes toast-in {
    from {
      opacity: 0;
      transform: translateY(16px) scale(0.96);
    }
  }
</style>
