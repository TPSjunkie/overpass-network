// src/main.tsx
import { createRoot } from "react-dom/client";
import { TonClient } from "@ton/ton";
import App from "./App";
import "./index.css";

const client = new TonClient({
  endpoint: "https://toncenter.com/api/v2/jsonRPC"
});

const initWasm = async () => {
  const wasmModule = await import('@/wasm/overpass_rs');
  await wasmModule.init();
  return wasmModule;
};

const init = async () => {
  try {
    await initWasm();
    const rootElement = document.getElementById("root");
    if (!rootElement) throw new Error("Root element not found");
    
    const root = createRoot(rootElement);
    root.render(<App />);
  } catch (error) {
    console.error('Failed to initialize app:', error);
    document.body.innerHTML = '<div class="error-message">Failed to load application. Please check console for details.</div>';
  }
};
init();