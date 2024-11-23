/// <reference types="vite/client" />

declare module '@wasm/overpass_rs' {
    export function init(): Promise<void>;
    export function init_panic_hook(): void;
    export function initiate_transaction(...args: any[]): any;
  }