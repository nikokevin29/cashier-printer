<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '$lib/api';
  import { showToast, showError } from '$lib/stores.svelte';
  import GuidedTextarea from '$lib/GuidedTextarea.svelte';

  const CHAR_WIDTH: Record<string, number> = { '58mm': 32, '75mm': 42, '80mm': 48 };

  let customerName = $state('');
  let content = $state('');
  let isSubmitting = $state(false);
  let charWidth = $state(48);

  onMount(async () => {
    try {
      const s = await api.getSettings();
      charWidth = CHAR_WIDTH[s.paper_size] ?? 48;
    } catch { /* use default */ }
  });

  async function submit() {
    const name = customerName.trim();
    const body = content.trim();
    if (!name) { showToast('Nama pelanggan wajib diisi', 'error'); return; }
    if (!body)  { showToast('Isi pesanan wajib diisi', 'error');   return; }

    isSubmitting = true;
    try {
      const id = await api.createOrder(name, body);
      await api.printOrder(id);
      showToast('Pesanan berhasil dicetak');
      customerName = '';
      content = '';
    } catch (err) {
      showError(err);
    } finally {
      isSubmitting = false;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') {
      e.preventDefault();
      submit();
    }
  }
</script>

<div class="page">
  <h2>Pesanan Baru</h2>

  <form onsubmit={(e) => { e.preventDefault(); submit(); }}>
    <div class="field">
      <label for="customer" class="field-label">Nama Pelanggan</label>
      <input
        id="customer"
        class="field-input"
        type="text"
        placeholder="Contoh: Pak Budi"
        bind:value={customerName}
        disabled={isSubmitting}
        autocomplete="off"
      />
    </div>

    <div class="field">
      <label for="content" class="field-label">
        Isi Pesanan
        <span class="label-hint">— garis biru = batas {charWidth} kolom</span>
      </label>
      <GuidedTextarea
        id="content"
        bind:value={content}
        {charWidth}
        placeholder="Tulis atau paste daftar pesanan di sini..."
        rows={10}
        disabled={isSubmitting}
        onkeydown={handleKeydown}
      />
      <span class="field-support">Ctrl+Enter untuk cetak langsung</span>
    </div>

    <button type="submit" class="btn-filled" disabled={isSubmitting}>
      <span class="material-symbols-outlined">print</span>
      {isSubmitting ? 'Mencetak...' : 'Cetak Pesanan'}
    </button>
  </form>
</div>

<style>
  .page { max-width: 580px; margin: 0 auto; }

  h2 {
    font-size: 1.375rem;
    font-weight: 500;
    color: var(--md-on-surface);
    margin-bottom: 1.5rem;
    letter-spacing: .01em;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 6px;
    margin-bottom: 1.25rem;
  }

  .field-label {
    font-size: .75rem;
    font-weight: 500;
    color: var(--md-on-surface-variant);
    letter-spacing: .05em;
    text-transform: uppercase;
  }

  .label-hint {
    font-weight: 400;
    text-transform: none;
    letter-spacing: 0;
    color: var(--md-outline);
    font-size: .72rem;
  }

  .field-input {
    height: 48px;
    padding: 0 16px;
    border: 1px solid var(--md-outline-variant);
    border-radius: 4px;
    font-size: .9375rem;
    font-family: 'Roboto', sans-serif;
    color: var(--md-on-surface);
    background: #fff;
    transition: border .15s;
    outline: none;
    width: 100%;
  }
  .field-input:focus {
    border: 2px solid var(--md-primary);
    padding: 0 15px;
  }
  .field-input:disabled {
    background: var(--md-surface-variant);
    color: var(--md-on-surface-variant);
    cursor: not-allowed;
  }

  .field-support {
    font-size: .72rem;
    color: var(--md-on-surface-variant);
    text-align: right;
  }

  .btn-filled {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    width: 100%;
    height: 40px;
    padding: 0 24px;
    background: var(--md-primary);
    color: var(--md-on-primary);
    border: none;
    border-radius: 20px;
    font-size: .875rem;
    font-weight: 500;
    font-family: 'Roboto', sans-serif;
    letter-spacing: .01em;
    cursor: pointer;
    transition: box-shadow .15s, opacity .15s;
    margin-top: .5rem;
  }
  .btn-filled .material-symbols-outlined { font-size: 18px; }
  .btn-filled:hover:not(:disabled) { box-shadow: var(--md-elev-1); }
  .btn-filled:active:not(:disabled) { box-shadow: none; opacity: .92; }
  .btn-filled:disabled { opacity: .38; cursor: not-allowed; }
</style>
