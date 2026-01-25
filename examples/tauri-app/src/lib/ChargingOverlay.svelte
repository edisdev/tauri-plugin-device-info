<script lang="ts">
    let { show }: { show: boolean } = $props();
</script>

{#if show}
    <div class="charging-overlay">
        <div class="charging-icon">⚡</div>
        <p>Charging Started</p>
    </div>
{/if}

<style>
    .charging-overlay {
        position: fixed;
        top: 50%;
        left: 50%;
        transform: translate(-50%, -50%);
        background: rgba(46, 204, 113, 0.2);
        /* Green tint for positive action */
        border: 2px solid #2ecc71;
        backdrop-filter: blur(20px);
        padding: 3rem;
        border-radius: 24px;
        z-index: 1000;
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        box-shadow: 0 0 50px rgba(46, 204, 113, 0.4);
        animation: popIn 0.3s cubic-bezier(0.175, 0.885, 0.32, 1.275);
    }

    .charging-icon {
        font-size: 5rem;
        margin-bottom: 1rem;
        filter: drop-shadow(0 0 10px #2ecc71);
    }

    p {
        font-size: 1.5rem;
        font-weight: 700;
        color: #fff;
        margin: 0;
        text-shadow: 0 2px 4px rgba(0, 0, 0, 0.5);
    }

    @keyframes popIn {
        from {
            transform: translate(-50%, -50%) scale(0.8);
            opacity: 0;
        }
        to {
            transform: translate(-50%, -50%) scale(1);
            opacity: 1;
        }
    }
</style>
