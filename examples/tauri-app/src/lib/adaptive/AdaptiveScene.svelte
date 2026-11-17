<!--
  Live background that visibly adapts:
  - tier decides how much is drawn (particle count, connecting links)
  - power profile decides how often (60 fps → 30 fps → one frozen frame)
-->
<script lang="ts">
    import type { PowerProfileKind, DeviceTierKind } from "tauri-plugin-device-info-api";
    import type { SceneStats } from "./types";

    let {
        profile,
        tier,
        theme,
        onstats,
    }: {
        profile: PowerProfileKind;
        tier: DeviceTierKind;
        /** Re-reads the theme's --scene-* colors when it changes. */
        theme: string;
        onstats?: (stats: SceneStats) => void;
    } = $props();

    let canvas: HTMLCanvasElement;

    const BASE_COUNT: Record<DeviceTierKind, number> = { low: 24, mid: 64, high: 130 };
    const COUNT_FACTOR: Record<PowerProfileKind, number> = { full: 1, balanced: 0.5, saver: 0.35 };
    const PALETTE: Record<PowerProfileKind, string[]> = {
        full: ["#646cff", "#8b8cff", "#aeb3ff", "#c4b5fd"],
        balanced: ["#facc15", "#fbbf24", "#fde68a"],
        saver: ["#6b7280", "#9ca3af"],
    };
    const LINK_DISTANCE = 110;

    type Particle = { x: number; y: number; vx: number; vy: number; r: number; color: string };

    $effect(() => {
        const ctx = canvas.getContext("2d");
        if (!ctx) return;

        const reducedMotion = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
        const animate = profile !== "saver" && !reducedMotion;
        const frameSkip = profile === "balanced" ? 2 : 1; // 60 → 30 fps
        const links = profile === "full" && tier !== "low";
        const count = Math.round(BASE_COUNT[tier] * COUNT_FACTOR[profile]);
        // Full-profile colors come from the active theme's --scene-* tokens.
        void theme;
        const css = getComputedStyle(document.documentElement);
        const token = (name: string) => css.getPropertyValue(name).trim();
        const themed = ["--scene-1", "--scene-2", "--scene-3", "--scene-4"].map(token).filter(Boolean);
        const linkRgb = token("--scene-link") || "140, 145, 255";
        const colors = profile === "full" && themed.length ? themed : PALETTE[profile];

        let width = 0;
        let height = 0;
        let particles: Particle[] = [];

        const resize = () => {
            const dpr = window.devicePixelRatio || 1;
            width = canvas.clientWidth;
            height = canvas.clientHeight;
            canvas.width = width * dpr;
            canvas.height = height * dpr;
            ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
            if (particles.length === 0) {
                particles = Array.from({ length: count }, () => ({
                    x: Math.random() * width,
                    y: Math.random() * height,
                    vx: (Math.random() - 0.5) * 0.6,
                    vy: (Math.random() - 0.5) * 0.6,
                    r: 1.2 + Math.random() * 2.2,
                    color: colors[Math.floor(Math.random() * colors.length)],
                }));
            }
            if (!animate) draw();
        };

        const draw = () => {
            ctx.clearRect(0, 0, width, height);

            if (links) {
                for (let i = 0; i < particles.length; i++) {
                    for (let j = i + 1; j < particles.length; j++) {
                        const a = particles[i];
                        const b = particles[j];
                        const d = Math.hypot(a.x - b.x, a.y - b.y);
                        if (d < LINK_DISTANCE) {
                            ctx.strokeStyle = `rgba(${linkRgb}, ${0.22 * (1 - d / LINK_DISTANCE)})`;
                            ctx.lineWidth = 1;
                            ctx.beginPath();
                            ctx.moveTo(a.x, a.y);
                            ctx.lineTo(b.x, b.y);
                            ctx.stroke();
                        }
                    }
                }
            }

            for (const p of particles) {
                ctx.globalAlpha = profile === "saver" ? 0.35 : 0.85;
                ctx.fillStyle = p.color;
                ctx.beginPath();
                ctx.arc(p.x, p.y, p.r, 0, Math.PI * 2);
                ctx.fill();
            }
            ctx.globalAlpha = 1;
        };

        const step = () => {
            for (const p of particles) {
                p.x += p.vx * frameSkip;
                p.y += p.vy * frameSkip;
                if (p.x < 0 || p.x > width) p.vx *= -1;
                if (p.y < 0 || p.y > height) p.vy *= -1;
            }
        };

        const observer = new ResizeObserver(resize);
        observer.observe(canvas);
        resize();

        // Measured, not assumed: the HUD shows what the scene really renders.
        let raf = 0;
        let frame = 0;
        let drawn = 0;
        let windowStart = performance.now();

        const loop = (now: number) => {
            raf = requestAnimationFrame(loop);
            if (frame++ % frameSkip !== 0) return;
            step();
            draw();
            drawn++;
            if (now - windowStart >= 1000) {
                onstats?.({ particles: count, links, fps: Math.round((drawn * 1000) / (now - windowStart)) });
                drawn = 0;
                windowStart = now;
            }
        };

        if (animate) {
            raf = requestAnimationFrame(loop);
        } else {
            onstats?.({ particles: count, links, fps: 0 });
        }

        return () => {
            cancelAnimationFrame(raf);
            observer.disconnect();
        };
    });
</script>

<canvas bind:this={canvas} class:frozen={profile === "saver"}></canvas>

<style>
    canvas {
        position: absolute;
        inset: 0;
        width: 100%;
        height: 100%;
        pointer-events: none;
    }

    .frozen {
        filter: grayscale(1);
    }
</style>
