<script setup lang="ts">
import { ref } from 'vue'
import { withBase } from 'vitepress'

const INSTALL = 'npm run tauri add device-info'
const copied = ref(false)

async function copyInstall() {
  try {
    await navigator.clipboard.writeText(INSTALL)
    copied.value = true
    setTimeout(() => (copied.value = false), 1500)
  } catch {
    /* clipboard blocked: the command stays selectable */
  }
}

const icons = {
  device: 'M8 2h8a2 2 0 0 1 2 2v16a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2zM11 18h2',
  battery: 'M3 7h15a2 2 0 0 1 2 2v6a2 2 0 0 1-2 2H3a2 2 0 0 1-2-2V9a2 2 0 0 1 2-2zM23 11v2M10 9l-2 3h4l-2 3',
  network: 'M2 9a15 15 0 0 1 20 0M5 12.5a10 10 0 0 1 14 0M8.5 16a5 5 0 0 1 7 0M12 19.5h.01',
  storage: 'M4 5a8 3 0 1 0 16 0 8 3 0 1 0-16 0M4 5v14c0 1.7 3.6 3 8 3s8-1.3 8-3V5M4 12c0 1.7 3.6 3 8 3s8-1.3 8-3',
  display: 'M3 4h18a1 1 0 0 1 1 1v11a1 1 0 0 1-1 1H3a1 1 0 0 1-1-1V5a1 1 0 0 1 1-1zM8 21h8M12 17v4',
  watch: 'M3 12h4l3-7 4 14 3-7h4',
  power: 'M12 2v8M18.4 6.6a9 9 0 1 1-12.8 0',
  tier: 'M4 18a8 8 0 1 1 16 0M12 18l4-6'
}

const signals = [
  { title: 'Device', body: 'Model, manufacturer, OS, a stable ID and the device name.', api: 'getDeviceInfo()', link: '/api/device-info', icon: icons.device },
  { title: 'Battery', body: 'Level, charging state and health.', api: 'watchBattery()', link: '/api/battery-info', icon: icons.battery },
  { title: 'Network', body: 'Connection type, IP and MAC address.', api: 'watchNetwork()', link: '/api/network-info', icon: icons.network },
  { title: 'Storage', body: 'Total and free space, SSD or HDD.', api: 'watchStorage()', link: '/api/storage-info', icon: icons.storage },
  { title: 'Display', body: 'Resolution, scale factor and refresh rate.', api: 'watchDisplay()', link: '/api/display-info', icon: icons.display },
  { title: 'Reactive watch', body: 'Subscribe once; get the current value, then every change.', api: 'watch*()', link: '/api/watch', icon: icons.watch },
  { title: 'Power profile', body: 'Full, balanced or saver — and the reasons behind it.', api: 'watchPowerProfile()', link: '/api/adaptive', icon: icons.power, isNew: true },
  { title: 'Device tier', body: 'Low, mid or high, with a 0–100 hardware score.', api: 'getDeviceTier()', link: '/api/adaptive', icon: icons.tier, isNew: true }
]

const profiles = [
  { name: 'Full', kind: 'good', when: 'Charging or on external power, or battery above 40% — cool, no power saving.', does: 'Everything on' },
  { name: 'Balanced', kind: 'warn', when: 'On battery at 21–40%, or the device is warming up.', does: 'Fewer effects, 30 fps' },
  { name: 'Saver', kind: 'bad', when: 'On battery at 20% or less, Low Power Mode, or too hot.', does: 'Effects off, frozen' }
]

const tested = [
  { device: 'Apple Silicon laptop (2025)', score: '90 · High', kind: 'good' },
  { device: 'Recent iPhone', score: '71 · High', kind: 'good' },
  { device: 'Windows laptop from 2013', score: '54 · Mid', kind: 'warn' },
  { device: 'Budget Android phone from 2017', score: '29 · Low', kind: 'bad' }
]

const platforms = [
  { name: 'macOS', source: 'IOKit · SystemConfiguration · NSProcessInfo', watch: 'Native events', native: true, thermal: 'Yes' },
  { name: 'Windows', source: 'WMI · GetSystemPowerStatus', watch: 'Polled', native: false, thermal: 'Not exposed by the OS' },
  { name: 'Linux', source: 'sysfs · ACPI platform profile · CPU thermal zones', watch: 'Polled', native: false, thermal: 'Yes' },
  { name: 'iOS', source: 'UIDevice · ProcessInfo', watch: 'Polled', native: false, thermal: 'Yes' },
  { name: 'Android', source: 'BatteryManager · PowerManager · cpufreq', watch: 'Polled', native: false, thermal: 'Android 10+' }
]
</script>

<template>
  <div class="di-home">
    <!-- Hero -->
    <section class="wrap hero">
      <div class="hero-copy">
        <a class="pill" :href="withBase('/api/adaptive')"><span class="dot" />New in 1.2 — power profile &amp; device tier</a>
        <h1>Know the device<br />your app runs on.</h1>
        <p class="lead">
          Battery, network, storage, display, power and hardware tier — one API for Tauri apps on Windows, macOS, Linux,
          iOS and Android, pushed to you the moment they change.
        </p>
        <div class="actions">
          <a class="btn brand" :href="withBase('/getting-started')">Get started</a>
          <a class="btn alt" :href="withBase('/api/')">API reference</a>
        </div>
        <div class="install">
          <code><span class="prompt">$</span> {{ INSTALL }}</code>
          <button type="button" :aria-label="copied ? 'Copied' : 'Copy install command'" @click="copyInstall">
            <svg v-if="!copied" width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><rect x="9" y="9" width="12" height="12" rx="2" /><path d="M5 15V5a2 2 0 0 1 2-2h10" /></svg>
            <svg v-else width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M5 12l5 5L20 7" /></svg>
          </button>
        </div>
      </div>

      <div class="hero-visual">
        <div class="code-card">
          <div class="code-bar"><i /><i /><i /><span>app.ts</span></div>
          <pre><span class="kw">import</span> { watchPowerProfile, getDeviceTier } <span class="kw">from</span> <span class="str">'tauri-plugin-device-info-api'</span>

<span class="kw">const</span> { tier } = <span class="kw">await</span> <span class="fn">getDeviceTier</span>()   <span class="cm">// 'low' | 'mid' | 'high'</span>

<span class="kw">await</span> <span class="fn">watchPowerProfile</span>(({ profile }) =&gt; {
  document.documentElement.dataset.power = profile
})</pre>
        </div>
        <div class="readouts">
          <div class="card readout">
            <div class="readout-head"><span class="eyebrow-sm">POWER PROFILE</span><span class="tag">example</span></div>
            <span class="profile-name">Balanced</span>
            <div class="scale" aria-hidden="true">
              <span class="stop done" /><span class="seg done" /><span class="stop now" /><span class="seg" /><span class="stop" />
            </div>
            <span class="reason">Battery ≤ 40%</span>
          </div>
          <div class="card readout tier">
            <svg width="76" height="76" viewBox="0 0 120 120" role="img" aria-label="Device tier score 74 of 100">
              <g transform="rotate(135 60 60)">
                <circle cx="60" cy="60" r="48" fill="none" class="ring-track" stroke-width="11" stroke-linecap="round" stroke-dasharray="226.2 301.6" />
                <circle cx="60" cy="60" r="48" fill="none" class="ring-value" stroke-width="11" stroke-linecap="round" stroke-dasharray="167.4 301.6" />
              </g>
              <text x="60" y="70" text-anchor="middle" class="ring-text">74</text>
            </svg>
            <div class="tier-copy">
              <span class="eyebrow-sm">DEVICE TIER</span>
              <span class="tier-name">High</span>
              <span class="muted small">8 cores · 8 GB · Android</span>
            </div>
          </div>
        </div>
      </div>
    </section>

    <!-- Signals -->
    <section class="band">
      <div class="wrap section">
        <header class="section-head">
          <span class="eyebrow">EVERY SIGNAL</span>
          <h2>Read it once, or watch it change.</h2>
          <p>Each getter has a watcher. Native OS events where they exist, a change-detecting poller everywhere else — your code never polls.</p>
        </header>
        <div class="signals">
          <a v-for="s in signals" :key="s.title" class="card signal" :class="{ fresh: s.isNew }" :href="withBase(s.link)">
            <div class="signal-top">
              <span class="icon"><svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path :d="s.icon" /></svg></span>
              <span v-if="s.isNew" class="new">NEW</span>
            </div>
            <span class="signal-title">{{ s.title }}</span>
            <span class="signal-body">{{ s.body }}</span>
            <code class="signal-api">{{ s.api }}</code>
          </a>
        </div>
      </div>
    </section>

    <!-- Adaptive -->
    <section class="wrap section">
      <header class="section-head wide">
        <span class="eyebrow">ADAPTIVE APPS</span>
        <h2>One signal for right now.<br />One for the hardware.</h2>
        <p>The plugin tells your app how much it can do. You decide what to turn off. The rules are written once in Rust, so every platform behaves the same.</p>
      </header>
      <div class="adaptive">
        <div class="card panel profiles-panel">
          <div class="panel-head">
            <code>getPowerProfile() · watchPowerProfile()</code>
            <span>Battery, charging, Low Power Mode and heat, folded into one recommendation.</span>
          </div>
          <div class="profiles">
            <div v-for="p in profiles" :key="p.name" class="profile" :class="p.kind">
              <span class="profile-title"><span class="pdot" />{{ p.name }}</span>
              <span class="profile-when">{{ p.when }}</span>
              <span class="profile-does">{{ p.does }}</span>
            </div>
          </div>
        </div>
        <div class="card panel tier-panel">
          <div class="panel-head">
            <code>getDeviceTier()</code>
            <span>A 0–100 score from memory, cores and clock, against devices of the same kind.</span>
          </div>
          <div class="tier-scale">
            <div class="bar" aria-hidden="true"><span class="bad" /><span class="warn" /><span class="good" /></div>
            <div class="bar-labels"><span class="bad">Low · 0–29</span><span class="warn">Mid · 30–64</span><span class="good">High · 65–100</span></div>
          </div>
          <div class="tested">
            <div v-for="d in tested" :key="d.device" class="tested-row">
              <span class="tested-device">{{ d.device }}</span>
              <span class="leader" />
              <span class="tested-score" :class="d.kind">{{ d.score }}</span>
            </div>
            <span class="muted small">Scores measured on real devices.</span>
          </div>
        </div>
      </div>
    </section>

    <!-- Platforms -->
    <section class="band">
      <div class="wrap section">
        <header class="section-head">
          <span class="eyebrow">FIVE PLATFORMS</span>
          <h2>Desktop and mobile, one API.</h2>
        </header>
        <div class="table-box">
          <table>
            <thead>
              <tr><th>Platform</th><th>Reads from</th><th>Watch</th><th>Thermal</th></tr>
            </thead>
            <tbody>
              <tr v-for="pl in platforms" :key="pl.name">
                <td class="pl-name"><a :href="withBase('/platforms/' + pl.name.toLowerCase())">{{ pl.name }}</a></td>
                <td>{{ pl.source }}</td>
                <td><span class="watch" :class="{ native: pl.native }">{{ pl.watch }}</span></td>
                <td>{{ pl.thermal }}</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </section>

    <!-- Demo -->
    <section class="wrap section">
      <div class="demo-head">
        <header class="section-head">
          <span class="eyebrow">SEE IT RUN</span>
          <h2>Every signal, live, in the demo app.</h2>
          <p>Vitals is the example app in <code>examples/tauri-app</code>: four pages built only from this plugin's getters and watchers.</p>
        </header>
        <a class="more" :href="withBase('/examples')">Run the example →</a>
      </div>
      <div class="demo-media">
        <video :src="withBase('/DeviceInfo_Promo.mp4')" autoplay muted loop playsinline preload="metadata" aria-label="The example app on macOS, iOS, Windows and Android" />
      </div>
    </section>
  </div>
</template>

<style scoped>
.di-home {
  color: var(--vp-c-text-1);
}

.wrap {
  max-width: 1200px;
  margin: 0 auto;
  padding-left: 24px;
  padding-right: 24px;
}

.band {
  background: var(--di-band);
  border-top: 1px solid var(--vp-c-divider);
  border-bottom: 1px solid var(--vp-c-divider);
}

.section {
  padding-top: 80px;
  padding-bottom: 80px;
  display: flex;
  flex-direction: column;
  gap: 40px;
}

.card {
  background: var(--di-surface);
  border: 1px solid var(--vp-c-divider);
  border-radius: 16px;
}

.muted {
  color: var(--vp-c-text-3);
}

.small {
  font-size: 12px;
}

/* Hero */
.hero {
  padding-top: 80px;
  padding-bottom: 72px;
  display: flex;
  flex-wrap: wrap;
  gap: 56px;
  align-items: center;
}

.hero-copy {
  flex: 1 1 480px;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.pill {
  align-self: flex-start;
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 5px 12px;
  border-radius: 999px;
  font-size: 13px;
  font-weight: 600;
  color: var(--vp-c-brand-1);
  background: var(--vp-c-brand-soft);
  text-decoration: none;
}

.pill .dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background: var(--vp-c-brand-1);
}

.hero h1 {
  margin: 0;
  font-size: 60px;
  line-height: 1.04;
  font-weight: 800;
  letter-spacing: -0.035em;
}

.lead {
  margin: 0;
  max-width: 540px;
  font-size: 19px;
  line-height: 1.55;
  color: var(--vp-c-text-2);
}

.actions {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
}

.btn {
  padding: 12px 22px;
  border-radius: 12px;
  font-size: 15px;
  font-weight: 600;
  text-decoration: none;
  transition: opacity 0.15s;
}

.btn:hover {
  opacity: 0.88;
}

.btn.brand {
  background: var(--vp-c-brand-1);
  color: var(--di-on-accent);
}

.btn.alt {
  background: var(--di-surface);
  border: 1px solid var(--vp-c-divider);
  color: var(--vp-c-text-1);
}

.install {
  display: flex;
  align-items: center;
  gap: 12px;
  max-width: 420px;
  padding: 10px 10px 10px 16px;
  border-radius: 12px;
  background: var(--vp-code-block-bg);
  border: 1px solid var(--vp-c-divider);
}

.install code {
  font-family: var(--vp-font-family-mono);
  font-size: 14px;
  color: var(--di-code-text);
  background: none;
  padding: 0;
}

.install .prompt {
  color: var(--vp-c-brand-1);
}

.install button {
  margin-left: auto;
  width: 34px;
  height: 34px;
  border-radius: 8px;
  border: 1px solid var(--vp-c-divider);
  color: var(--vp-c-text-3);
  display: flex;
  align-items: center;
  justify-content: center;
}

.install button:hover {
  color: var(--vp-c-text-1);
}

.hero-visual {
  flex: 1 1 460px;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.code-card {
  border-radius: 16px;
  overflow: hidden;
  background: var(--vp-code-block-bg);
  border: 1px solid var(--vp-c-divider);
}

.code-bar {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 11px 16px;
  border-bottom: 1px solid var(--vp-c-divider);
  font-family: var(--vp-font-family-mono);
  font-size: 12px;
  color: var(--vp-c-text-3);
}

.code-bar i {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background: var(--di-dot);
}

.code-bar span {
  margin-left: 8px;
}

.code-card pre {
  margin: 0;
  padding: 18px 20px;
  overflow-x: auto;
  font-family: var(--vp-font-family-mono);
  font-size: 13.5px;
  line-height: 1.75;
  color: var(--di-code-text);
}

.kw {
  color: var(--di-kw);
}

.str {
  color: var(--di-str);
}

.fn {
  color: var(--di-fn);
}

.cm {
  color: var(--di-comment);
}

.readouts {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 14px;
}

.readout {
  padding: 18px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.readout-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.eyebrow-sm {
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.1em;
  color: var(--vp-c-text-3);
}

.tag {
  font-size: 11px;
  color: var(--vp-c-text-3);
}

.profile-name {
  font-size: 30px;
  font-weight: 800;
  letter-spacing: -0.02em;
  color: var(--di-warn);
}

.scale {
  display: flex;
  align-items: center;
}

.scale .stop {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  border: 2px solid var(--di-track);
  box-sizing: border-box;
}

.scale .stop.done {
  background: var(--di-warn);
  border-color: var(--di-warn);
}

.scale .stop.now {
  width: 14px;
  height: 14px;
  background: var(--di-warn);
  border-color: var(--di-warn);
  box-shadow: 0 0 0 4px var(--di-warn-soft);
}

.scale .seg {
  flex: 1;
  height: 3px;
  background: var(--di-track);
}

.scale .seg.done {
  background: var(--di-warn);
}

.reason {
  align-self: flex-start;
  padding: 3px 9px;
  border-radius: 999px;
  font-size: 12px;
  font-weight: 600;
  color: var(--di-warn);
  background: var(--di-warn-soft);
  border: 1px solid var(--di-warn-line);
}

.readout.tier {
  flex-direction: row;
  align-items: center;
  gap: 16px;
}

.ring-track {
  stroke: var(--di-track);
}

.ring-value {
  stroke: var(--vp-c-brand-1);
}

.ring-text {
  fill: var(--vp-c-text-1);
  font-family: var(--vp-font-family-base);
  font-size: 32px;
  font-weight: 800;
}

.tier-copy {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.tier-name {
  font-size: 26px;
  font-weight: 800;
  color: var(--vp-c-brand-1);
}

/* Section headers */
.section-head {
  display: flex;
  flex-direction: column;
  gap: 12px;
  max-width: 640px;
}

.section-head.wide {
  max-width: 680px;
}

.eyebrow {
  font-size: 13px;
  font-weight: 700;
  letter-spacing: 0.12em;
  color: var(--vp-c-brand-1);
}

.section-head h2 {
  margin: 0;
  padding: 0;
  border: none;
  font-size: 38px;
  line-height: 1.15;
  font-weight: 800;
  letter-spacing: -0.025em;
}

.section-head p {
  margin: 0;
  font-size: 17px;
  line-height: 1.55;
  color: var(--vp-c-text-2);
}

.section-head code,
.demo-head code {
  font-size: 15px;
}

/* Signals */
.signals {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(250px, 1fr));
  gap: 16px;
}

.signal {
  padding: 22px;
  display: flex;
  flex-direction: column;
  gap: 12px;
  color: inherit;
  text-decoration: none;
  transition: border-color 0.15s, transform 0.15s;
}

.signal:hover {
  border-color: var(--vp-c-brand-1);
  transform: translateY(-2px);
}

.signal.fresh {
  border-color: var(--di-good-line);
}

.signal-top {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.icon {
  width: 40px;
  height: 40px;
  border-radius: 11px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--vp-c-brand-1);
  background: var(--vp-c-brand-soft);
}

.signal.fresh .icon {
  color: var(--di-on-accent);
  background: var(--vp-c-brand-1);
}

.new {
  padding: 3px 8px;
  border-radius: 999px;
  font-size: 11px;
  font-weight: 700;
  color: var(--vp-c-brand-1);
  background: var(--vp-c-brand-soft);
}

.signal-title {
  font-size: 17px;
  font-weight: 700;
}

.signal-body {
  font-size: 14px;
  line-height: 1.5;
  color: var(--vp-c-text-2);
}

.signal-api {
  margin-top: auto;
  font-family: var(--vp-font-family-mono);
  font-size: 12.5px;
  color: var(--vp-c-brand-1);
}

/* Adaptive */
.adaptive {
  display: flex;
  flex-wrap: wrap;
  gap: 20px;
}

.panel {
  min-width: 0;
  padding: 28px;
  border-radius: 20px;
  display: flex;
  flex-direction: column;
  gap: 22px;
}

.profiles-panel {
  flex: 1 1 520px;
}

.tier-panel {
  flex: 1 1 360px;
}

.panel-head {
  display: flex;
  flex-direction: column;
  gap: 4px;
  font-size: 15px;
  color: var(--vp-c-text-2);
}

.panel-head code {
  font-family: var(--vp-font-family-mono);
  font-size: 14px;
  color: var(--vp-c-brand-1);
}

.profiles {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 12px;
}

.profile {
  padding: 16px;
  border-radius: 14px;
  border: 1px solid;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.profile.good {
  --c: var(--di-good);
  background: var(--di-good-soft);
  border-color: var(--di-good-line);
}

.profile.warn {
  --c: var(--di-warn);
  background: var(--di-warn-soft);
  border-color: var(--di-warn-line);
}

.profile.bad {
  --c: var(--di-bad);
  background: var(--di-bad-soft);
  border-color: var(--di-bad-line);
}

.profile-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 17px;
  font-weight: 800;
  color: var(--c);
}

.pdot {
  width: 9px;
  height: 9px;
  border-radius: 50%;
  background: var(--c);
}

.profile-when {
  font-size: 13px;
  line-height: 1.45;
  color: var(--vp-c-text-2);
}

.profile-does {
  margin-top: auto;
  font-size: 12px;
  color: var(--vp-c-text-3);
}

.tier-scale {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.bar {
  display: flex;
  height: 12px;
  border-radius: 999px;
  overflow: hidden;
}

.bar span:nth-child(1) {
  width: 30%;
}

.bar span:nth-child(2),
.bar span:nth-child(3) {
  width: 35%;
}

.bar .bad {
  background: var(--di-bad);
}

.bar .warn {
  background: var(--di-warn);
}

.bar .good {
  background: var(--di-good);
}

.bar-labels {
  display: grid;
  grid-template-columns: 30% 35% 35%;
  font-size: 13px;
  font-weight: 600;
}

.bad {
  color: var(--di-bad);
}

.warn {
  color: var(--di-warn);
}

.good {
  color: var(--di-good);
}

.tested {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.tested-row {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 14px;
}

.tested-device {
  color: var(--vp-c-text-2);
}

.leader {
  flex: 1;
  border-bottom: 1px dashed var(--vp-c-divider);
}

.tested-score {
  font-weight: 700;
  white-space: nowrap;
}

/* Platforms */
.table-box {
  overflow-x: auto;
  border-radius: 16px;
  border: 1px solid var(--vp-c-divider);
  background: var(--di-surface);
}

.table-box table {
  display: table;
  width: 100%;
  min-width: 720px;
  margin: 0;
  border-collapse: collapse;
  font-size: 14px;
}

.table-box th {
  padding: 14px 20px;
  text-align: left;
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--vp-c-text-3);
  border-bottom: 1px solid var(--vp-c-divider);
}

.table-box td {
  padding: 14px 20px;
  color: var(--vp-c-text-2);
  border-bottom: 1px solid var(--vp-c-divider);
}

.table-box tr:last-child td {
  border-bottom: none;
}

.pl-name a {
  font-weight: 700;
  color: var(--vp-c-text-1);
  text-decoration: none;
}

.pl-name a:hover {
  color: var(--vp-c-brand-1);
}

.watch {
  padding: 3px 9px;
  border-radius: 999px;
  font-size: 12px;
  font-weight: 600;
  color: var(--vp-c-text-2);
  background: var(--di-track);
  white-space: nowrap;
}

.watch.native {
  color: var(--vp-c-brand-1);
  background: var(--vp-c-brand-soft);
}

/* Demo */
.demo-head {
  display: flex;
  flex-wrap: wrap;
  align-items: flex-end;
  gap: 16px;
}

.more {
  margin-left: auto;
  font-size: 15px;
  font-weight: 600;
  color: var(--vp-c-brand-1);
  text-decoration: none;
}

.demo-media {
  border-radius: 20px;
  overflow: hidden;
  border: 1px solid var(--vp-c-divider);
  background: #000;
}

.demo-media video {
  display: block;
  width: 100%;
  height: auto;
}

/* Phone */
@media (max-width: 760px) {
  .wrap {
    padding-left: 20px;
    padding-right: 20px;
  }

  .hero {
    padding-top: 44px;
    padding-bottom: 56px;
    gap: 36px;
  }

  .hero h1 {
    font-size: 40px;
  }

  .lead {
    font-size: 17px;
  }

  .section {
    padding-top: 56px;
    padding-bottom: 56px;
    gap: 32px;
  }

  .section-head h2 {
    font-size: 29px;
  }

  .code-card pre {
    padding: 14px 16px;
    font-size: 12px;
  }

  .readouts,
  .profiles {
    grid-template-columns: minmax(0, 1fr);
  }

  .panel {
    padding: 22px;
  }
}

@media (prefers-reduced-motion: reduce) {
  .signal,
  .btn {
    transition: none;
  }

  .signal:hover {
    transform: none;
  }
}
</style>
