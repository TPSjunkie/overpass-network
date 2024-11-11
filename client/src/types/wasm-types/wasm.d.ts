// src/types/wasm.d.ts

declare module '@wasm/overpass_rs/package.json' {
  const content: {
      name: string;
      version: string;
      files: string[];
      module: string;
      types: string;
      sideEffects: boolean;
      author?: string;
      description?: string;
      license?: string;
      repository?: {
          type: string;
          url: string;
      };
      keywords?: string[];
      bugs?: {
          url: string;
      };
      homepage?: string;
      dependencies?: Record<string, string>;
      devDependencies?: Record<string, string>;
      peerDependencies?: Record<string, string>;
      engines?: {
          node?: string;
          npm?: string;
      };
  };
  export default content;
}

declare module '*.wasm' {
  const content: ArrayBuffer;
  export default content;
  export const instanceMemory: WebAssembly.Memory;
  export const instantiate: (options?: WebAssembly.Imports) => Promise<WebAssembly.Instance>;
  export const module: WebAssembly.Module;
}