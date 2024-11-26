import { TonClient, Address } from "@ton/ton";
import type { Transaction, WalletExtension } from '../types/wasm-types/index';
import { WasmTransactionSender } from '../utils/wasmTransactionSender';import getOverpassData from '../hooks/getOverpassData';
import { Builder, Cell, type Contract } from '@ton/core';
import { initWASM } from '@/wasm/wasmINIT/initWASM';
import { ChannelContract, WSSparseMerkleTree } from "@/wasm/overpass_rs";
import OffChainState from "@/wasm/overpass_rs";import SparseMerkleTree from "@/wasm/overpass_rs";import ContractStorage from "@/wasm/overpass_rs";


interface OPClientConfig {
    tonEndpoint: string;
    tonApiKey: string;
    overpassEndpoint: string;
    overpassApiKey: string;
}

export const getOPClientConfig = (): OPClientConfig => {
    const tonEndpoint = process.env.TON_ENDPOINT;
    const tonApiKey = process.env.TON_API_KEY;
    const overpassEndpoint = process.env.OVERPASS_ENDPOINT;
    const overpassApiKey = process.env.OVERPASS_API_KEY;

    if (!tonEndpoint || !tonApiKey || !overpassEndpoint || !overpassApiKey) {
        throw new Error('TON_ENDPOINT, TON_API_KEY, OVERPASS_ENDPOINT, and OVERPASS_API_KEY must be set in the environment variables');
    }

    return {
        tonEndpoint,
        tonApiKey,
        overpassEndpoint,
        overpassApiKey,
    };
};

export const createTonClient = (): TonClient => {
    const { tonEndpoint, tonApiKey } = getOPClientConfig();
    
    try {
        return new TonClient({
            endpoint: tonEndpoint,
            apiKey: tonApiKey,
        });
    } catch (error) {
        console.error('Failed to create TonClient:', error);
        throw new Error('Unable to initialize TonClient. Please check your configuration and try again.');
    }
};

export const interactWithSmartContract = async (address: string, method: string, params: any[]): Promise<any> => {
    const tonClient = createTonClient();
    const contract = Address.parse(address);
    
    try {
        const result = await tonClient.runMethod(contract, method, params);
        return result.stack;
    } catch (error) {
        console.error('Failed to interact with smart contract:', error);
        throw new Error('Unable to interact with smart contract. Please check your parameters and try again.');
    }
};

export const handleBOC = (bocData: string): Cell => {
    try {
        return Cell.fromBoc(Buffer.from(bocData, 'base64'))[0];
    } catch (error) {
        console.error('Failed to handle BOC:', error);
        throw new Error('Unable to process BOC. Please check your data and try again.');
    }
};
export const sendWasmTransaction = async (contract: Contract, message: Cell): Promise<string> => {
    try {
        const contractAddress = contract.address.toString();
        const workChain = BigInt(contract.address.workChain);
        
        // Get contract state using direct property access
        const contractState = contract.state || '';
        const contractCode = contract.code || '';
        const balance = contract.balance || BigInt(0);
        const lastTx = contract.lastTransactionId || '';
        const publicKey = contract.publicKey || '';
        const walletType = contract.type || '';
        const seqno = contract.seqno || BigInt(0);
        const expireAt = contract.expireAt || BigInt(0);
        const signature = contract.signature || '';

        const txHash = WasmTransactionSender.sendTransaction(
            contractAddress,
            publicKey,
            message.toBoc().toString('base64'),
            balance,
            lastTx,
            contractCode,
            contractState,
            workChain,
            walletType,
            seqno,
            expireAt,
            signature
        );
        
        if (typeof txHash !== 'string') {
            throw new Error('Transaction hash is not a string');
        }
        return txHash;
    } catch (error) {
        console.error('Failed to send WASM transaction:', error);
        throw new Error('Unable to send WASM transaction. Please check your contract and message data.');
    }
};export const fetchOverpassData = async (dataId: string): Promise<any> => {
    try {
        const data = getOverpassData(dataId);
        return data;
    } catch (error) {
        console.error('Failed to fetch Overpass data:', error);
        throw new Error('Unable to fetch Overpass data. Please check your data ID and try again.');
    }
};export const createCellFromBoc = (bocString: string): Cell => {
    try {
        return Cell.fromBoc(Buffer.from(bocString, 'base64'))[0];
    } catch (error) {
        console.error('Failed to create Cell from BOC:', error);
        throw new Error('Unable to create Cell from BOC. Please check your BOC string and try again.');
    }
};

export const buildMessage = (params: Record<string, unknown>): Cell => {
    const builder = new Builder();
    try {
        validateParams(params);
        const messageContent = buildMessageContent(params);
        
        builder.storeUint(messageContent.opcode, 32);
        builder.storeAddress(Address.parse(messageContent.address));
        builder.storeCoins(messageContent.amount);
        
        if (messageContent.payload) {
            builder.storeRef(Cell.fromBoc(Buffer.from(messageContent.payload, 'base64'))[0]);
        }
        
        if (messageContent.stateInit) {
            builder.storeRef(Cell.fromBoc(Buffer.from(messageContent.stateInit, 'base64'))[0]);
        }
        
        return builder.endCell();
    } catch (error) {
        console.error('Failed to build message:', error);
        if (error instanceof Error) {
            throw new Error(`Unable to build message: ${error.message}`);
        } else {
            throw new Error('Unable to build message due to an unknown error');
        }
    }
};

function validateParams(params: Record<string, unknown>) {
    if (!params.opcode || typeof params.opcode !== 'number') {
        throw new Error('Invalid opcode parameter');
    }
    if (!params.address || typeof params.address !== 'string') {
        throw new Error('Invalid address parameter');
    }
    if (!params.amount || (typeof params.amount !== 'string' && typeof params.amount !== 'number')) {
        throw new Error('Invalid amount parameter');
    }
    if (params.payload !== undefined && typeof params.payload !== 'string') {
        throw new Error('Invalid payload parameter');
    }
    if (params.stateInit !== undefined && typeof params.stateInit !== 'string') {
        throw new Error('Invalid stateInit parameter');
    }
    try {
        BigInt(params.amount as string | number);
    } catch {
        throw new Error('Invalid amount format - must be convertible to BigInt');
    }
    try {
        Address.parse(params.address as string);
    } catch {
        throw new Error('Invalid address format');
    }
}

function buildMessageContent(params: Record<string, unknown>): {
    opcode: number;
    address: string;
    amount: bigint;
    payload?: string;
    stateInit?: string;
} {
    // Implement message content building logic here
    return {
        opcode: params.opcode as number,
        address: params.address as string,
        amount: BigInt(params.amount as string | number),
        payload: params.payload as string | undefined,
        stateInit: params.stateInit as string | undefined,
    };
}

export const initializeWasm = async (): Promise<void> => {
    await initWasm();
};

export const createWalletExtension = (address: Address): WalletExtension => {
    return new WalletExtension(address);
};
export const createChannelContract = (secretKey: Uint8Array): ChannelContract => {
    return new ChannelContract(secretKey);
};

export const createChannelState = (
    ownerAddress: Uint8Array,
    channelId: number,
    balance: bigint,
    nonce: bigint,
    merkleRoot: Uint8Array
): ChannelState => {
    return new ChannelState(ownerAddress, channelId, balance, nonce, merkleRoot);
};

export const createContractStorage = (): ContractStorage => {
    return new ContractStorage();
};

export const createOffChainState = (
    ownerAddress: Uint8Array,
    seqno: number,
    nonce: number,
    currentTime: bigint,
    offChainContract: Uint8Array,
    latestRoot: Uint8Array
): OffChainState => {
    return new OffChainState(ownerAddress, seqno, nonce, currentTime, offChainContract, latestRoot);
};

export const createSparseMerkleTree = (depth: number): SparseMerkleTree => {
    return new SparseMerkleTree(depth);
};

export const createWSSparseMerkleTree = (): WSSparseMerkleTree => {
    return new WSSparseMerkleTree();
};

export const createTransaction = (transactionJson: string): Transaction => {
    return JSON.parse(transactionJson) as Transaction;
};
export default {
    getOPClientConfig,
    createTonClient,
    interactWithSmartContract,
    handleBOC,
    sendWasmTransaction,
    fetchOverpassData,
    createCellFromBoc,
    buildMessage,
    initializeWasm,
    createWalletExtension,
    createChannelContract,
    createChannelState,
    createContractStorage,
    createOffChainState,
    createSparseMerkleTree,
    createWSSparseMerkleTree,
    createTransaction,
};
