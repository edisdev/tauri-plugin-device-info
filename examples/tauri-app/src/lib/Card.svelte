<script lang="ts">
    import type { Snippet } from "svelte";
    let { title, children }: { title: string; children: Snippet } = $props();
</script>

<div class="card">
    <h2>{title}</h2>
    {@render children()}
</div>

<style>
    .card {
        background: rgba(36, 36, 36, 0.6);
        padding: 2rem;
        border-radius: 16px;
        border: 1px solid rgba(255, 255, 255, 0.05);
        backdrop-filter: blur(10px);
        box-shadow: 0 8px 32px rgba(0, 0, 0, 0.2);
        transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
        width: 300px; /* Default width, flex-grow handled by parent if needed */
        flex-grow: 1;
        min-width: 250px;
        display: flex;
        flex-direction: column;
    }

    .card:hover {
        transform: translateY(-4px);
        background: rgba(45, 45, 45, 0.7);
        border-color: rgba(100, 108, 255, 0.2);
        box-shadow: 0 12px 40px rgba(0, 0, 0, 0.3);
    }

    h2 {
        margin-top: 0;
        margin-bottom: 1.5rem;
        font-size: 1.4rem;
        font-weight: 600;
        color: #aeb3ff;
        border-bottom: 1px solid rgba(255, 255, 255, 0.1);
        padding-bottom: 1rem;
    }
</style>
