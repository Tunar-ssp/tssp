import { writable } from "svelte/store";
import { probeAuthStatus, type AuthSnapshot } from "../api";

interface AuthState extends AuthSnapshot {
  loading: boolean;
  error: string | null;
}

export const authState = writable<AuthState>({
  required: false,
  user: null,
  loading: true,
  error: null,
});

export async function hydrateAuth() {
  authState.set({
    required: false,
    user: null,
    loading: true,
    error: null,
  });

  try {
    const snapshot = await probeAuthStatus();
    authState.set({
      ...snapshot,
      loading: false,
      error: null,
    });
  } catch (error) {
    authState.set({
      required: false,
      user: null,
      loading: false,
      error: error instanceof Error ? error.message : "Failed to load auth state",
    });
  }
}
