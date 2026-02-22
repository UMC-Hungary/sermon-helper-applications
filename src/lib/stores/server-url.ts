import { writable } from 'svelte/store';

export const serverUrl = writable<string>('http://localhost:3737');
export const serverPort = writable<number>(3737);
export const authToken = writable<string>('');
