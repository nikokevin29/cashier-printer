export const toast = $state<{
  message: { text: string; type: 'success' | 'error' } | null;
}>({ message: null });

let toastTimer: ReturnType<typeof setTimeout> | null = null;

export function showToast(text: string, type: 'success' | 'error' = 'success') {
  if (toastTimer) clearTimeout(toastTimer);
  toast.message = { text, type };
  toastTimer = setTimeout(() => {
    toast.message = null;
    toastTimer = null;
  }, 3000);
}

export function showError(err: unknown) {
  const msg =
    err && typeof err === 'object' && 'message' in err
      ? String((err as { message: unknown }).message)
      : String(err);
  showToast(msg, 'error');
}
