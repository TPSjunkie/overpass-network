import type { Cell } from "@ton/core";


export enum RootOpCode {
    SubmitEpoch = 0x01,
    ValidateEpoch = 0x02,
    FinalizeEpoch = 0x03,
    UpdateGlobalRoot = 0x10,
    ValidateGlobalState = 0x11,
    RegisterIntermediate = 0x20,
    RemoveIntermediate = 0x21,
    ValidateIntermediate = 0x22,
    tryFrom,
    CreateChannel,
}

export enum ChannelOpCode {
    CreatePayment = 0xA0,
    UpdateState = 0xA1,
    FinalizeState = 0xA2,
    DisputeState = 0xA3,
    InitChannel = 0xA4,
    ValidatePayment = 0xB0,
    ValidateState = 0xB1,
    ValidateFinalState = 0xB2,
    ValidateDispute = 0xB3,
    ValidateSettlement = 0xB4,
    ProcessPayment = 0xD1,
    InitiateSettlement = 0xE0,
    ProcessSettlement = 0xE1,
    FinalizeSettlement = 0xE2,
    ValidateTransition = 0xF0,
    VerifyProof = 0xF1,
    GetChannel = 0xF2,
    tryFrom,
}

export enum OpCode {
    Root,
    Intermediate,
    Wallet,
    Channel,
    Storage,
    fromNumber
}

export class OpCodeUtils {
    static toU8(opCode: OpCode): number {
        switch (opCode) {
            case OpCode.Root: return 0;
            case OpCode.Intermediate: return 1;
            case OpCode.Wallet: return 2;
            case OpCode.Channel: return 3;
            case OpCode.Storage: return 4;
            default: throw new Error("Invalid OpCode");
        }
    }

    static fromU8(value: number): OpCode | undefined {
        if (value >= 0 && value <= 4) {
            return value as OpCode;
        }
        return undefined;
    }
}
export interface Operation {
    op_code(): OpCode;
    validate(): boolean;
    execute(): Promise<void>;
}

export interface OperationResult {
    success: boolean;
    op_code: OpCode;
    message?: string;
    data?: Uint8Array;
}
export interface OperationFactory {
    create(op_code: OpCode, data: Uint8Array): Operation;
}
export interface Operation {
    op_code(): OpCode;
    validate(): boolean;
    execute(): Promise<void>;
    toCell(): Cell;
    toCellWithStateInit(): Cell;
    toCellWithPayload(): Cell;  
    toCellWithStateInitAndPayload(): Cell;
}
export enum StorageOpCode {
    ChargeNode = 0xC0,
    DischargeNode = 0xC1,
    ValidateBattery = 0xC2,
    PropagateState = 0xD0,
    SyncState = 0xD1,
    ValidateSync = 0xD2,
    ReplicateState = 0xE0,
    ValidateReplica = 0xE1,
    tryFrom,
}

export enum WalletOpCode {
    CreateChannel = 0x70,
    UpdateChannel = 0x71,
    CloseChannel = 0x72,
    ValidateChannel = 0x73,
    UpdateWalletTree = 0x80,
    ValidateWalletTree = 0x81,
    UpdateWalletState = 0x90,
    ValidateWalletState = 0x91,
    UpdateBalance = 0xB0,
    ValidateBalance = 0xB1,
    CreateTransaction = 0xC0,
    ValidateTransaction = 0xC1,
    ProcessTransaction = 0xC2,
    GenerateWalletProof = 0xD0,
    VerifyWalletProof = 0xD1,
    tryFrom,
}



export class OVPOpCodeConverter {
    static fromChannelOpCode(code: ChannelOpCode): number {
        return code;
    }

    static fromRootOpCode(code: RootOpCode): number {
        return code;
    }

    static fromStorageOpCode(code: StorageOpCode): number {
        return code;
    }

    static fromWalletOpCode(code: WalletOpCode): number {
        return code;
    }

    static fromIntermediateOpCode(code: IntermediateOpCode): number {
        return code;
    }
}

export enum WalletExtensionStateChangeOp {
    CreateChannel = 0x70,
    UpdateChannel = 0x71,
    CloseChannel = 0x72,
    ValidateChannel = 0x73,
    ProcessChannel = 0x74,
    ValidateTransaction = 0x80,
    ProcessTransaction = 0x81,
    FinalizeState = 0x82,
}

export class OpCodeConverter {
    static tryFrom(arg0: number): any {
        throw new Error("Method not implemented.");
    }
    static fromChannelOpCode(code: ChannelOpCode): number {
        return code;
    }

    static toChannelOpCode(value: number): ChannelOpCode {
        if (Object.values(ChannelOpCode).includes(value)) {
            return value as ChannelOpCode;
        }
        throw new Error("Invalid Channel operation code");
    }

    static toRootOpCode(value: number): RootOpCode {
        if (Object.values(RootOpCode).includes(value)) {
            return value as RootOpCode;
        }
        throw new Error("Invalid Root operation code");
    }

    static toStorageOpCode(value: number): StorageOpCode {
        if (Object.values(StorageOpCode).includes(value)) {
            return value as StorageOpCode;
        }
        throw new Error("Invalid Storage operation code");
    }

    static toWalletOpCode(value: number): WalletOpCode {
        if (Object.values(WalletOpCode).includes(value)) {
            return value as WalletOpCode;
        }
        throw new Error("Invalid Wallet operation code");
    }

    static toWalletExtensionStateChangeOp(value: number): WalletExtensionStateChangeOp {
        if (Object.values(WalletExtensionStateChangeOp).includes(value)) {
            return value as WalletExtensionStateChangeOp;
        }
        throw new Error("Invalid WalletExtensionStateChangeOp operation code");
    }
}
export enum IntermediateOpCode {
    RequestChannelOpen = 0x20,
    ApproveChannelOpen = 0x21,
    RequestChannelClose = 0x22,
    ProcessChannelClose = 0x23,
    RequestManualRebalance = 0x24,
    CheckChannelBalances = 0x25,
    ValidateRebalance = 0x26,
    ExecuteRebalance = 0x27,
    RegisterWallet = 0x30,
    UpdateWalletRoot = 0x31,
    ValidateWalletRoot = 0x32,
    ProcessWalletRoot = 0x33,
    ValidateStateUpdate = 0x34,
    AssignStorageNode = 0x40,
    UpdateStorageNode = 0x41,
    ValidateStorageNode = 0x42,
    StoreWalletState = 0x43,
    VerifyStorageState = 0x44,
    ReplicateState = 0x45,
    PrepareRootSubmission = 0x50,
    SubmitToRoot = 0x51,
    UpdateIntermediateState = 0x52,
    ValidateIntermediateState = 0x53,
    UpdateTree = 0x60,
    ValidateTree = 0x61,
    tryFrom
}

export class IntermediateOpCodeConverter {
    static tryFrom(value: number): IntermediateOpCode | Error {
        switch (value) {
            case 0x20: return IntermediateOpCode.RequestChannelOpen;
            case 0x21: return IntermediateOpCode.ApproveChannelOpen;
            case 0x22: return IntermediateOpCode.RequestChannelClose;
            case 0x23: return IntermediateOpCode.ProcessChannelClose;
            case 0x24: return IntermediateOpCode.RequestManualRebalance;
            case 0x25: return IntermediateOpCode.CheckChannelBalances;
            case 0x26: return IntermediateOpCode.ValidateRebalance;
            case 0x27: return IntermediateOpCode.ExecuteRebalance;
            case 0x30: return IntermediateOpCode.RegisterWallet;
            case 0x31: return IntermediateOpCode.UpdateWalletRoot;
            case 0x32: return IntermediateOpCode.ValidateWalletRoot;
            case 0x33: return IntermediateOpCode.ProcessWalletRoot;
            case 0x34: return IntermediateOpCode.ValidateStateUpdate;
            case 0x40: return IntermediateOpCode.AssignStorageNode;
            case 0x41: return IntermediateOpCode.UpdateStorageNode;
            case 0x42: return IntermediateOpCode.ValidateStorageNode;
            case 0x43: return IntermediateOpCode.StoreWalletState;
            case 0x44: return IntermediateOpCode.VerifyStorageState;
            case 0x45: return IntermediateOpCode.ReplicateState;
            case 0x50: return IntermediateOpCode.PrepareRootSubmission;
            case 0x51: return IntermediateOpCode.SubmitToRoot;
            case 0x52: return IntermediateOpCode.UpdateIntermediateState;
            case 0x53: return IntermediateOpCode.ValidateIntermediateState;
            case 0x60: return IntermediateOpCode.UpdateTree;
            case 0x61: return IntermediateOpCode.ValidateTree;
            default: return new Error("Invalid Intermediate operation code");
        }
    }
}
describe('OpCode Conversions', () => {
    test('channel opcode conversion', () => {
        expect(OpCodeConverter.tryFrom(0xA0)).toBe(ChannelOpCode.CreatePayment);
        expect(() => OpCodeConverter.tryFrom(0xFF)).toThrow("Invalid Channel operation code");
        expect(ChannelOpCode.CreatePayment).toBe(0xA0);
    });
    test('root opcode conversion', () => {
        expect(RootOpCode.SubmitEpoch).toBe(0x01);
        expect(RootOpCode.ValidateEpoch).toBe(0x02);
        expect(RootOpCode.FinalizeEpoch).toBe(0x03);
        expect(RootOpCode.UpdateGlobalRoot).toBe(0x10);
        expect(RootOpCode.ValidateGlobalState).toBe(0x11);
        expect(RootOpCode.RegisterIntermediate).toBe(0x20);
        expect(RootOpCode.RemoveIntermediate).toBe(0x21);
        expect(RootOpCode.ValidateIntermediate).toBe(0x22);
    });
    test('storage opcode conversion', () => {
        expect(StorageOpCode.ChargeNode).toBe(0xC0);
        expect(StorageOpCode.DischargeNode).toBe(0xC1);
        expect(StorageOpCode.ValidateBattery).toBe(0xC2);
        expect(StorageOpCode.PropagateState).toBe(0xD0);
        expect(StorageOpCode.SyncState).toBe(0xD1);
        expect(StorageOpCode.ValidateSync).toBe(0xD2);
        expect(StorageOpCode.ReplicateState).toBe(0xE0);
        expect(StorageOpCode.ValidateReplica).toBe(0xE1);
    });

    test('intermediate opcode conversion', () => {
        expect(IntermediateOpCodeConverter.tryFrom(0x20)).toBe(IntermediateOpCode.RequestChannelOpen);
        expect(() => IntermediateOpCodeConverter.tryFrom(0xFF)).toThrow("Invalid Intermediate operation code");
        expect(IntermediateOpCode.RequestChannelOpen).toBe(0x20);
    });

});

