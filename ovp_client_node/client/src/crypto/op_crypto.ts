// ./src/crypto/op_crypto.ts
import { type KeyPair, getSecureRandomBytes, keyPairFromSeed } from "@ton/crypto";
import { randomBytes } from "@noble/hashes/utils";

export const crypto = typeof window !== 'undefined' ? window.crypto : null;

export async function generateRandomKeyPair(): Promise<KeyPair> {
    let randomSeed: Buffer;
    
    if (crypto && crypto.getRandomValues) {
        const tempArray = new Uint8Array(32);
        crypto.getRandomValues(tempArray);
        randomSeed = Buffer.from(tempArray);
    } else {
        // Use a WASM-friendly alternative
        randomSeed = Buffer.from(randomBytes(32));
    }
    
    // Generate a key pair from the random seed
    const keyPair = keyPairFromSeed(randomSeed);
    
    return keyPair;
}

export async function generateRandomSeed(): Promise<Buffer> {
    if (crypto && crypto.getRandomValues) {
        const tempArray = new Uint8Array(32);
        crypto.getRandomValues(tempArray);
        return Buffer.from(tempArray);
    } else {
        // Use a WASM-friendly alternative
        return Buffer.from(randomBytes(32));
    }
}

export async function generateRandomSecretKey(): Promise<Buffer> {
    if (crypto && crypto.getRandomValues) {
        const tempArray = new Uint8Array(32);
        crypto.getRandomValues(tempArray);
        return Buffer.from(tempArray);
    } else {
        // Use a WASM-friendly alternative
        return Buffer.from(randomBytes(32));
    }
}
