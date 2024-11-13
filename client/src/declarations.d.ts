// ./declarations.d.ts

declare module '@wasm/overpass_rs' {
    export function init_panic_hook(): void;
    export function initiate_transaction(
      channelId: string,
      messageCell: string,
      keyPair: any,
      transactionType: number,
      groupId: string
    ): Promise<string>;
    
    export default function(options?: {
        module?: WebAssembly.Module;
        imports?: WebAssembly.Imports;
    }): Promise<void>;
}

declare module '@tonconnect/ui-react' {
    export interface TonConnectUIProviderProps {
        manifestUrl: string;
    }
    export class TonConnectUIProvider extends React.Component<TonConnectUIProviderProps> {
        constructor(props: TonConnectUIProviderProps);
        getChildContext(): {
            connector: TonConnectUI;
        };
    }
}

declare module '@tonconnect/ui' {
    export interface TonConnectUI {
        connected: boolean;
        connectionRestored: Promise<void>;
        connector: TonConnectUI;
        open(wallet: WalletContractV4): Promise<void>;
        connect(wallet: WalletContractV4): Promise<void>;
        disconnect(): Promise<void>;
        sendTransaction(transaction: any): Promise<void>;
    }
    export class TonConnectUI {
        account: any;
        connectWallet() {
          throw new Error('Method not implemented.');
        }
        onStatusChange(p0: (wallet: any) => void, error: { (...data: any[]): void; (...data: any[]): void; (message?: any, ...optionalParams: any[]): void; }, arg0: (wallet: any) => Promise<void>) {
          throw new Error('Method not implemented.');
        }
        getWallets() {
          throw new Error('Method not implemented.');
        }
        constructor(props: any);
        connected: boolean;
        connectionRestored: Promise<void>;
        connector: TonConnectUI;
        open(wallet: WalletContractV4): Promise<void>;
        connect(wallet: WalletContractV4): Promise<void>;
        disconnect(): Promise<void>;
        sendTransaction(transaction: any): Promise<void>;
    }
}

declare module '@tonconnect/sdk' {
    export interface WalletInfoRemote {
        universalLink: string;
        bridgeUrl: string;
        jsBridgeKey: string;
    }
    export interface WalletInfoCurrentlyEmbedded {
        jsBridgeKey: string;
    }
    export interface WalletInfoCurrentlyInjected {
		[x: string]: any;
        jsBridgeKey: string;
    }
}

declare module '@ton/core' {
    export class Address {
        hash: any;
      balance: any;
      walletAddress: Address | PromiseLike<Address>;
        static parseRaw(arg0: string): Address {
            throw new Error('Method not implemented.');
        }
        constructor(workchain: number, hash: Uint8Array);
        static parse(source: string): Address;
        toString(urlSafe?: boolean, testOnly?: boolean, bounceable?: boolean): string;
        toCell(): Cell;
    }

    export class Cell {
      [x: string]: any;
		static fromBase64(boc: any) {
			throw new Error('Method not implemented.');
		}
        bits: BitString;
        refs: Cell[];
      static EMPTY: Maybe<string | Cell>;
        constructor();
        static fromBoc(boc: Buffer | Uint8Array): Cell[];
        toBoc(opts?: { idx?: boolean; crc32?: boolean }): Buffer;
        hash(): Buffer;
        beginParse(opts?: { throwOnRefs?: boolean }): Slice;
        get length(): number;
    }

    export interface BitString {
        length: number;
        get(index: number): boolean;
        set(index: number, value: boolean): void;
        get length(): number;
        toString(base?: number): string;
        toJSON(): string;
        toArray(): boolean[];
        toUint(bits: number): number;
        toUintBe(bits: number): number;
        toUintLe(bits: number): number;
        toUint8(): number;
    }

    export class Slice {
        constructor(cell: Cell);
        loadBuffer(bits: number): Buffer;
        loadAddress(): Address | null;
        loadCoins(): bigint;
        loadUint(bits: number): number;
        loadUintBe(bits: number): number;
        loadUintLe(bits: number): number;
        loadUint8(): number;
        get bits(): BitString;
    }

    export interface ContractProvider {
        external(message: Cell): Promise<void>;
        internal(via: Sender, args: { value: bigint; bounce?: boolean; body?: Cell }): Promise<void>;
        getBalance(): Promise<bigint>;
        getSeqno(): Promise<number>;
        getPublicKey(): Promise<bigint>;
        getChannelState(channelId: number): Promise<{
            id: number;
            balance: bigint;
            nonce: number;
            group: number;
            merkleRoot: Cell;
        }>;
        getPendingTransaction(channelId: number): Promise<{
            id: number;
            balance: bigint;
            nonce: number;
            group: number;
            merkleRoot: Cell;
        }>;
        getChannelBalance(channelId: number): Promise<bigint>;
        getChannelCount(): Promise<number>;
        getPreAuthorizedIntermediate(channelId: number): Promise<Address>;
    }

    export interface Sender {
        address: Address | null;
        balance: bigint;
        send(message: Cell): Promise<void>;
        sendMessage(message: {
            to: string;
            value: bigint;
            data?: Cell;
            bounce?: boolean;
            mode?: number;
            body?: Cell;
            code?: Cell;
        }): Promise<void>;
    }

    export function beginCell(): {
        storeAddress(address: Address | null): CellBuilder;
        storeBuffer(buffer: Buffer): CellBuilder;
        storeCoins(amount: bigint): CellBuilder;
        storeInt(value: number, bits: number): CellBuilder;
        storeRef(cell: Cell): CellBuilder;
        storeString(str: string): CellBuilder;
        storeUint(value: number, bits: number): CellBuilder;
        storeUint8(value: number): CellBuilder;
        storeUint16(value: number): CellBuilder;
        storeUint32(value: number): CellBuilder;
        storeUint64(value: number): CellBuilder;    
        endCell(): Cell;
    }

    export interface Contract {
        address: Address | null;
        balance: bigint;
        send(message: Cell): Promise<void>;
        sendMessage(message: {
            to: string;
            value: bigint;
            data?: Cell;
            bounce?: boolean;
            mode?: number;
            body?: Cell;
            code?: Cell;
        }): Promise<void>;
        getBalance(): Promise<bigint>;
        getSeqno(): Promise<number>;
        getPublicKey(): Promise<bigint>;
        getChannelState(channelId: number): Promise<{
            id: number;
            balance: bigint;
            nonce: number;
            group: number;
            merkleRoot: Cell;
            init?: { code: Cell; data: Cell };
        }>;
        getPendingTransaction(channelId: number): Promise<{
            id: number;
            balance: bigint;
            nonce: number;
            group: number;
            merkleRoot: Cell;
        }>;
        getChannelBalance(channelId: number): Promise<bigint>;
        getChannelCount(): Promise<number>;
        getPreAuthorizedIntermediate(channelId: number): Promise<Address>;
    }

    export type StateInit = {
        code?: Cell;
        data?: Cell;
        split_depth?: number;
        special?: boolean;
    }

    export type SendMode = number;

    export type ExternalMessage = {
        to: Address;
        from?: Address;
        importFee?: bigint;
        bounce?: boolean;
        body: Cell;
    }

    export type Maybe<T> = T | null | undefined;

    export interface CellBuilder {
        storeCoins(value: bigint): unknown;
        storeAddress(intermediateAddress: Address): unknown;
        storeUint(subwallet: number, arg1: number): unknown;
        bits: BitString;
        refs: Cell[];
        storeBit(value: boolean): CellBuilder;
        storeBits(bits: BitString): CellBuilder;
        storeRef(cell: Cell): CellBuilder;
        storeSlice(slice: Slice): CellBuilder;
        storeBuilder(builder: CellBuilder): CellBuilder;
        endCell(): Cell;
        clone(): CellBuilder;
    }
}

declare module '@ton/ton' {
    import { Address, Cell, Contract, StateInit, SendMode, ExternalMessage, Maybe } from '@ton/core';

    export class TonClient {
        provider(jettonAddress: Address) {
          throw new Error('Method not implemented.');
        }
        static getClient() {
            throw new Error("Method not implemented.");
        }
        open(wallet: WalletContractV4): unknown {
          throw new Error('Method not implemented.');
        }
        constructor(config: { endpoint: string; timeout?: number });
        isTestnet: boolean;
        
        async runMethod(
            address: Address | Contract,
            name: string,
            params?: any[]
        ): Promise<{
            stack: Array<{ type: string; value: any }>;
            gasUsed: number;
            exitCode: number;
        }>;

        async getBalance(address: Address): Promise<bigint>;
        
        async getTransactions(
            address: Address,
            opts: { 
                limit: number;
                lt?: string;
                hash?: string;
                to_lt?: string;
                inclusive?: boolean;
            }
        ): Promise<Transaction[]>;

        async sendMessage(message: { to: string; value: bigint; data?: Cell }): Promise<void>;

        async getContractState(address: Address): Promise<{
            balance: bigint;
            code: Cell | null;
            data: Cell | null;
            lastTransaction: {
                lt: string;
                hash: string;
            } | null;
            state: AccountStatus;
        }>;
    }

    export class WalletContractV4 implements Contract {
        static create(config: {
            publicKey: Buffer;
            workchain?: number;
            stateInit?: StateInit;
        }): WalletContractV4;

        createTransfer(config: {
            seqno: number;
            secretKey: Buffer;
            messages: {
                to: string | Address;
                value: bigint;
                bounce?: boolean;
                body?: Cell;
            }[];
            sendMode?: SendMode;
            timeout?: number;
        }): ExternalMessage;

        address: Address;
        init?: { code: Cell; data: Cell };


    }    export interface Transaction {
        lt: string;
        hash: string;
        prevTransactionHash: string;
        prevTransactionLt: string;
        now: number;
        outMessagesCount: number;
        origStatus: AccountStatus;
        endStatus: AccountStatus;
        totalFees: bigint;
        inMessage?: Message;
        outMessages: Message[];
    }

    export type AccountStatus = 'active' | 'uninit' | 'frozen';

    export interface Message {
        body: Cell;
        info: {
            type: 'internal' | 'external';
            src?: Address;
            dest?: Address;
            value?: {
                coins: bigint;
            };
            bounce?: boolean;
            bounced?: boolean;
            createdAt?: number;
            createdLt?: string;
        };
        source?: string;
        destination?: string;
        value: bigint;
        fwd_fee?: bigint;
        ihr_fee?: bigint;
        created_lt?: string;
        body_hash?: string;
        msg_data?: {
            text?: string;
            bytes?: string;
        };
    }

    export { Address, Cell, Contract } from '@ton/core';
}

export {};
