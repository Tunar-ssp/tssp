import { error as showError, success as showSuccess } from '$lib/stores/notifications';

export async function withLoadingState<T>(
  asyncFn: () => Promise<T>,
  options?: { errorMessage?: string; successMessage?: string }
): Promise<T | null> {
  try {
    return await asyncFn();
  } catch (e) {
    const message = options?.errorMessage || (e instanceof Error ? e.message : 'An error occurred');
    showError(message);
    return null;
  }
}

export function formatErrorMessage(error: unknown, fallback = 'An error occurred'): string {
  if (error instanceof Error) return error.message;
  if (typeof error === 'string') return error;
  return fallback;
}

export async function loadDataWithHandler<T>(
  asyncFn: () => Promise<T>,
  onSuccess?: (data: T) => void,
  options?: { errorMessage?: string; successMessage?: string }
): Promise<T | null> {
  try {
    const data = await asyncFn();
    if (options?.successMessage) showSuccess(options.successMessage);
    onSuccess?.(data);
    return data;
  } catch (e) {
    const message = formatErrorMessage(e, options?.errorMessage || 'Failed to load data');
    showError(message);
    return null;
  }
}

export function createAsyncHandler(options?: { errorMessage?: string; successMessage?: string }) {
  return {
    withLoading: <T,>(fn: () => Promise<T>) => withLoadingState(fn, options),
    loadData: <T,>(fn: () => Promise<T>, onSuccess?: (data: T) => void) =>
      loadDataWithHandler(fn, onSuccess, options),
  };
}
