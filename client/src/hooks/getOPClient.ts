// ./src/hooks/getOPClient.ts
import type { OPClient, Channel } from "@/types/wasm-types/index";
import getOverpassData from '@/hooks/getOverpassData';
import { TonClient } from "@ton/ton";

let isInitialized = false;

