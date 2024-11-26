/// <reference types="vite/client" />
/// <reference types="vite-plugin-wasm/client" />

declare module '@wasm/overpass_rs' {
    export * from '@/types/overpass_rs';
    const initWasm: () => Promise<void>;
    export default initWasm;
  }
  
  declare module '*.wasm' {
    const content: WebAssembly.Module;
    export default content;
  }
  
  declare module 'worker-loader!*' {
    class WebpackWorker extends Worker {
      constructor();
    }
    export default WebpackWorker;
  }
  
  interface Window {
    __WB_DISABLE_DEV_LOGS: boolean;
  }