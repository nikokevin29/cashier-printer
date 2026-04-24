<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import { api } from '$lib/api';
  import { toast } from '$lib/stores.svelte';
  import { openUrl } from '@tauri-apps/plugin-opener';

  let { children } = $props();

  onMount(() => {
    api.purgeOldOrders().catch(() => {});
  });

  const navLinks = [
    { href: '/new',      label: 'Baru',    icon: 'receipt'  },
    { href: '/history',  label: 'Riwayat', icon: 'history'  },
    { href: '/settings', label: 'Setelan', icon: 'settings' },
  ];
</script>

<div class="app">
  <!-- MD3 Navigation Rail -->
  <nav class="nav-rail">
    <div class="rail-brand">
      <span class="brand-logo">PPO</span>
      <span class="brand-name">Print Paste<br>Order</span>
    </div>

    <div class="rail-items">
      {#each navLinks as link}
        {@const active = $page.url.pathname === link.href ||
          ($page.url.pathname.startsWith('/edit') && link.href === '/history')}
        <a href={link.href} class="rail-item" class:active>
          <div class="rail-indicator">
            <span
              class="material-symbols-outlined"
              style:font-variation-settings={active
                ? "'FILL' 1,'wght' 500,'GRAD' 0,'opsz' 24"
                : "'FILL' 0,'wght' 400,'GRAD' 0,'opsz' 24"}
            >{link.icon}</span>
          </div>
          <span class="rail-label">{link.label}</span>
        </a>
      {/each}
    </div>
  </nav>

  <!-- Content area -->
  <div class="main-area">
    <main class="content">
      {@render children()}
    </main>

    <footer class="watermark">
      <button class="watermark-link" onclick={() => openUrl('https://rejekiamerta.com')}>
        CV REJEKI AMERTA JAYA &nbsp;·&nbsp; © 2026
      </button>
    </footer>
  </div>
</div>

{#if toast.message}
  <div class="toast toast-{toast.message.type}" role="alert">
    {toast.message.text}
  </div>
{/if}

<style>
  /* ── MD3 Design Tokens ─────────────────────────────────────── */
  :root {
    --md-primary:            #1a1a2e;
    --md-on-primary:         #ffffff;
    --md-primary-container:  #e8eaf6;
    --md-secondary:          #3949ab;
    --md-surface:            #f8f9ff;
    --md-surface-variant:    #e8eaf6;
    --md-surface-container:  #eeeef8;
    --md-on-surface:         #1c1b1f;
    --md-on-surface-variant: #49454f;
    --md-outline:            #7986cb;
    --md-outline-variant:    #cac4d0;
    --md-error:              #b3261e;
    --md-error-container:    #f9dedc;
    --md-success:            #1b6b24;
    --md-elev-1: 0 1px 2px rgba(0,0,0,.25), 0 1px 3px 1px rgba(0,0,0,.12);
    --md-elev-2: 0 1px 2px rgba(0,0,0,.25), 0 2px 6px 2px rgba(0,0,0,.12);
    --md-elev-3: 0 4px 8px 3px rgba(0,0,0,.12), 0 1px 3px rgba(0,0,0,.25);
  }

  /* ── Global resets ─────────────────────────────────────────── */
  :global(*, *::before, *::after) {
    box-sizing: border-box;
    margin: 0;
    padding: 0;
  }

  :global(body) {
    font-family: 'Roboto', system-ui, sans-serif;
    font-size: 14px;
    background: var(--md-surface);
    color: var(--md-on-surface);
    overflow: hidden;
  }

  /* ── App shell ─────────────────────────────────────────────── */
  .app {
    display: flex;
    height: 100vh;
  }

  /* ── Navigation Rail ───────────────────────────────────────── */
  .nav-rail {
    width: 80px;
    background: var(--md-primary);
    display: flex;
    flex-direction: column;
    align-items: center;
    flex-shrink: 0;
    user-select: none;
  }

  .rail-brand {
    width: 100%;
    padding: 14px 8px 12px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    border-bottom: 1px solid rgba(255,255,255,.1);
  }

  .brand-logo {
    font-size: 15px;
    font-weight: 700;
    letter-spacing: .06em;
    color: #fff;
  }

  .brand-name {
    font-size: 8.5px;
    font-weight: 400;
    color: rgba(255,255,255,.45);
    text-align: center;
    line-height: 1.3;
    letter-spacing: .03em;
  }

  .rail-items {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
    padding: 10px 0;
    width: 100%;
  }

  .rail-item {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    text-decoration: none;
    width: 100%;
    padding: 4px 0;
  }

  .rail-indicator {
    width: 56px;
    height: 32px;
    border-radius: 16px;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: background .18s;
  }

  .rail-item:hover .rail-indicator {
    background: rgba(255,255,255,.09);
  }

  .rail-item.active .rail-indicator {
    background: rgba(255,255,255,.18);
  }

  .rail-item .material-symbols-outlined {
    font-size: 22px;
    color: rgba(255,255,255,.5);
    transition: color .18s;
  }

  .rail-item.active .material-symbols-outlined {
    color: #fff;
  }

  .rail-label {
    font-size: 10px;
    font-weight: 500;
    letter-spacing: .02em;
    color: rgba(255,255,255,.5);
    transition: color .18s;
  }

  .rail-item.active .rail-label {
    color: #fff;
    font-weight: 600;
  }

  /* ── Main area ─────────────────────────────────────────────── */
  .main-area {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    background: var(--md-surface);
  }

  .content {
    flex: 1;
    overflow-y: auto;
    padding: 1.5rem;
  }

  /* ── Watermark footer ──────────────────────────────────────── */
  .watermark {
    flex-shrink: 0;
    text-align: center;
    background: var(--md-primary);
  }

  .watermark-link {
    display: block;
    width: 100%;
    padding: .28rem 0;
    background: none;
    border: none;
    font-size: .62rem;
    font-weight: 600;
    letter-spacing: .12em;
    text-transform: uppercase;
    color: rgba(255,255,255,.25);
    cursor: pointer;
    transition: color .15s;
    font-family: inherit;
  }
  .watermark-link:hover { color: rgba(255,255,255,.7); }

  /* ── Toast ─────────────────────────────────────────────────── */
  .toast {
    position: fixed;
    bottom: 1.5rem;
    left: calc(80px + (100vw - 80px) / 2);
    transform: translateX(-50%);
    padding: .6rem 1.25rem;
    border-radius: 8px;
    font-size: .875rem;
    font-weight: 500;
    z-index: 9999;
    box-shadow: var(--md-elev-3);
    animation: slideUp .2s ease-out;
    font-family: inherit;
  }

  .toast-success { background: var(--md-success); color: #fff; }
  .toast-error   { background: var(--md-error);   color: #fff; }

  @keyframes slideUp {
    from { opacity: 0; transform: translateX(-50%) translateY(8px); }
    to   { opacity: 1; transform: translateX(-50%) translateY(0);   }
  }
</style>
