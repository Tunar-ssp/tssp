import { writable } from "svelte/store";

export interface BannerState {
  tone: "info" | "success" | "warning" | "error";
  title: string;
  detail?: string;
}

export const bannerState = writable<BannerState | null>({
  tone: "info",
  title: "Frontend migration in progress",
  detail: "The new Svelte shell is being scaffolded alongside the legacy dashboard until feature parity is reached.",
});

export const commandPaletteOpen = writable(false);
export const commandQuery = writable("");

export function showBanner(next: BannerState | null) {
  bannerState.set(next);
}

export function openCommandPalette(seed = "") {
  commandQuery.set(seed);
  commandPaletteOpen.set(true);
}

export function closeCommandPalette() {
  commandPaletteOpen.set(false);
}
