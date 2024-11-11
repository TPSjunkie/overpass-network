import type { MerkleProof } from '@/types/wasm-types';
import type { WasmBridge } from './WasmBridgeTypes';

export class SparseMerkleTreeManager {
    private tree: any;

    constructor(wasmBridge: WasmBridge, depth: number) {
        this.tree = wasmBridge.SparseMerkleTree.new(depth);
    }

    async insert(key: string, value: string): Promise<void> {
        await this.tree.insert(key, value);
    }

    async getProof(key: string): Promise<MerkleProof> {
        const proof = await this.tree.getProof(key);
        return proof;
    }

    async verifyProof(proof: MerkleProof, key: string, value: string): Promise<boolean> {
        return await this.tree.verify(proof, key, value);
    }

    async serialize(): Promise<Uint8Array> {
        return this.tree.serialize();
    }

    async deserialize(bytes: Uint8Array): Promise<void> {
        this.tree = this.tree.deserialize(bytes);
    }

    getRoot(): string {
        return this.tree.root || "";
    }
}
