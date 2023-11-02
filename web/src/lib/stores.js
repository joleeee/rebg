import { writable } from 'svelte/store';

export const selectedAddress = writable(0);
export const selectedIdx = writable(null);
export const showSymbols = writable(true);