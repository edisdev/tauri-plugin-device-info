<script lang="ts">
    let { loading, fetchData }: { loading: boolean; fetchData: () => void } =
        $props();
</script>

<header class="sticky-header">
    <div class="header-content">
        <h1>Device Info</h1>
        <button onclick={fetchData} disabled={loading}>
            {loading ? "Refreshing..." : "Refresh Data"}
        </button>
    </div>
</header>

<style>
    .sticky-header {
        position: sticky;
        top: 0;
        left: 0;
        width: 100%;
        z-index: 100;
        background: rgba(
            18,
            18,
            18,
            0.85
        ); /* Dark semi-transparent background */
        backdrop-filter: blur(12px); /* Glassmorphism */
        -webkit-backdrop-filter: blur(12px);
        border-bottom: 1px solid rgba(255, 255, 255, 0.1);
        /* padding: 0.75rem 2rem; */
        box-sizing: border-box;
        transition: all 0.3s ease;
        padding-inline: 2rem;
        padding-bottom: 2rem;
        padding-top: calc(
            env(safe-area-inset-top) + 2rem
        ); /* Push content below Dynamic Island */
    }

    .header-content {
        max-width: 1200px; /* Match grid max-width */
        margin: 0 auto;
        display: flex;
        justify-content: space-between;
        align-items: center;
        flex-wrap: wrap;
        gap: 1rem;
    }

    h1 {
        margin: 0;
        font-size: 2rem; /* Smaller font for sticky header */
        font-weight: 700;
        background: linear-gradient(to right, #646cff, #9089fc);
        -webkit-background-clip: text;
        background-clip: text;
        color: transparent;
    }

    button {
        background: linear-gradient(135deg, #646cff 0%, #535bf2 100%);
        color: white;
        border: none;
        padding: 0.9rem 1.2rem;
        border-radius: 8px;
        cursor: pointer;
        font-size: 1rem;
        font-weight: 600;
        transition: all 0.3s ease;
        box-shadow: 0 4px 12px rgba(100, 108, 255, 0.3);
        white-space: nowrap;
    }

    button:hover {
        transform: translateY(-2px);
        box-shadow: 0 6px 16px rgba(100, 108, 255, 0.4);
    }

    button:disabled {
        background: #3a3a3a;
        color: #888;
        cursor: not-allowed;
        box-shadow: none;
        transform: none;
    }

    @media (max-width: 480px) {
        .sticky-header {
            /* padding: 0.75rem 1rem; */
            padding-inline: 2rem;
            padding-bottom: 1rem;
            padding-top: calc(
                env(safe-area-inset-top) + 1rem
            ); /* Push content below Dynamic Island */
        }
        h1 {
            font-size: 1.5rem;
        }
    }
</style>
