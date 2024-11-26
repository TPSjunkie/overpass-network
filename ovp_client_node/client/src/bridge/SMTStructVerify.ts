import type { MerkleProof } from "@/types/wasm-types";

export class SMTStructVerify {
    constructor() {}
  

    async verify(proof: MerkleProof, leaf: string): Promise<boolean> {
        if (!proof || !proof.path || !proof.siblings || !proof.directions || !proof.root) {
            throw new Error('Invalid Merkle proof structure');
        }

        if (proof.path.length !== proof.siblings.length || proof.path.length !== proof.directions.length) {
            throw new Error('Merkle proof components length mismatch');
        }

        let computedHash = leaf;
        
        try {
            for (let i = 0; i < proof.path.length; i++) {
                const sibling = proof.siblings[i];
                const direction = proof.directions[i];

                if (typeof sibling !== 'string' || sibling.length === 0) {
                    throw new Error(`Invalid sibling at index ${i}`);
                }

                if (typeof direction !== 'number' || (direction !== 0 && direction !== 1)) {
                    throw new Error(`Invalid direction at index ${i}`);
                }

                // Concatenate and hash based on the direction
                // direction 0: current hash is left, sibling is right
                // direction 1: sibling is left, current hash is right
                if (direction === 0) {
                    computedHash = this.hash(computedHash + sibling);
                } else {
                    computedHash = this.hash(sibling + computedHash);
                }
            }

            // Verify the computed root matches the provided root
            return computedHash.toLowerCase() === proof.root.toLowerCase();
        } catch (error: unknown) {
            if (error instanceof Error) {
                throw new Error(`Merkle proof verification failed: ${error.message}`);
            } else {
                throw new Error('Merkle proof verification failed: Unknown error');
            }
        }
    }

    private hash(data: string): string {
        const crypto = require('crypto');
        const cleanData = data.toLowerCase().replace(/^0x/, '');
        const buffer = Buffer.from(cleanData, 'hex');
        return crypto.createHash('sha256').update(buffer).digest('hex');
    }
}

export default SMTStructVerify;
