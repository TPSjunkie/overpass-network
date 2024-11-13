// src/main.tsx
import { createRoot } from "react-dom/client";
import React from "react";

import { App } from "./App";
import "./index.css";

const initWasm = async () => {
  const wasmModule = await import('@/wasm/overpass_rs');
  await wasmModule.default.init();
  return wasmModule;
};
const init = async () => {
  try {
    await initWasm();
    const rootElement = document.getElementById("root");
    if (!rootElement) throw new Error("Root element not found");
    
    const root = createRoot(rootElement!);
    root
  } catch (error) {
    console.error('Failed to initialize app:', error);
    document.body.innerHTML = '<div class="error-message">Failed to load application. Please check console for details.</div>';
  }
};

init();