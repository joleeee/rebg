import { writable } from 'svelte/store';

export const selectedAddress = writable(null);
export const selectedIdx = writable(null);
export const showSymbols = writable(true);