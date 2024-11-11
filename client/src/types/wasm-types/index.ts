// src/types/index.ts
// src/types.ts
// src/types/index.ts
// src/types.ts
import type { ReactNode } from 'react';


export type TransactionDetails = {
  address: ReactNode;
  tree: any;
  id: string;
  sender: string;
  recipient: string;
  amount: number;
  timestamp: number;
  date: string;
  channelId: number;
  groupId: number;
  nonce: number;
  seqno: number;
  hash: string;
  senderSignature: string;
  merkleRoot: string;
  type: "incoming" | "outgoing" | "internal";
  payload: string;
  OPcode: number;
  description: string;
  status: "pending" | "completed" | "failed";
  statusMessage: string;
  statusColor: string;
};export type TransactionResponse = {
  [x: string]: any;
  address: ReactNode;
  tree: any;
  id: string;
  sender: string;
  recipient: string;
  amount: number;
  timestamp: number;
  date: string;
  channelId: number;
  groupId: number;
  nonce: number;
  seqno: number;
  hash: string;
  senderSignature: string;
  merkleRoot: string;
  type: "incoming" | "outgoing" | "internal";
  payload: string;
  OPcode: number;
  description: string;
  status: "pending" | "completed" | "failed";
  statusMessage: string;
  statusColor: string;
};
export interface AnalyticsData {
  totalTransactions: number;
  totalAmount: number;
  averageTransactionAmount: number;
  averageTransactionFrequency: number;
  topSenders: string[];
  topRecipients: string[];
  transactionTypes: {
    incoming: number;
    outgoing: number;
  };
  transactionStatuses: {
    pending: number;
    completed: number;
    failed: number;
  };
  transactionDates: {
    [date: string]: number;
  };
  transactionAmounts: {
    [amount: number]: number;
  };
  transactionFrequencies: {
    [frequency: number]: number;
  };
  transactionDescriptions: {
    [description: string]: number;
  };
  transactionRecipients: {
    [recipient: string]: number;
  };
  transactionSenders: {
    [sender: string]: number;
  };
  transactionStatusesByDate: {
    [date: string]: {
      pending: number;
      completed: number;
      failed: number;
    };
  };
}
export interface MerkleProof {
  directions: any;
  siblings: any;
  root: string;
  proof: string[];
  path: number[];
}



export type OPClient = {
  open(arg0: unknown): unknown;
  getChannel(channelId: string): unknown;
  getChannelState: (channelId: string) => Promise<ChannelState>;
  updateChannelState: (channelId: string, updatedState: any) => Promise<void>;
  sendTransaction: (channelId: string, transactionJson: string) => Promise<void>;
  getTransactionHistory: (channelId: string) => Promise<Transaction[]>;
  getAnalytics: (channelId: string) => Promise<Analytics>;
  getGroupChannels: () => Promise<string[]>;
  groupChannels: (groupId: string) => Promise<Channel[]>;
  getGroupBalances: (groupId: string) => Promise<number>;
  createChannel: (channelData: any) => Promise<void>;
};
export type WalletInfo = {
  address: string;
  balance: number;
  transactions: Transaction[];
};

export type Group = {
  id: string;
  name: string;
  description: string;
  channels: Channel[];
  balance: number;
  transactions: Transaction[];
};

export interface ITransactionManager {
  getChannelState: (channelId: string) => Promise<ChannelState>;
  updateChannelState: (channelId: string, updatedState: any) => Promise<void>;
  sendTransaction: (
    channelId: string,
    transactionJson: string
  ) => Promise<void>;
  saveTransaction: (transaction: Transaction) => Promise<void>;
  storeTransaction: (transaction: Transaction) => Promise<void>;
  getTransactionHistory: (channelId: string) => Promise<Transaction[]>;
  getAnalytics: (channelId: string) => Promise<Analytics>;
  getGroupChannels: () => Promise<string[]>;
  groupChannels: (groupId: string) => Promise<Channel[]>;
  getGroupBalances: (groupId: string) => Promise<number>;
  createChannel: (channelData: any) => Promise<void>;
}

export interface SparseMerkleTree {
  getProof(key: string): Proof;
  insert(leaf: string | number): Promise<string>;
  verify(proof: Proof): Promise<boolean>;
  root: string;
  leaves: string[];
  proofs: Proof[];
}



export interface MainNavItem {
  id: number;
  title: string;
  description: string;
  iconUrl: string;
}

export interface SubNavItem {
  id: number;
  title: string;
  description?: string;
  type: 'menu' | 'widget' | 'action';
  content: React.ReactNode;
}

export type Transaction = {
  payload(arg0: string, arg1: string, payload: any, channelId: any, groupId: any, transactionType: any): unknown;
  channelId(arg0: string, arg1: string, payload: any, channelId: any, groupId: any, transactionType: any): unknown;
  transactionType(arg0: string, arg1: string, payload: any, channelId: any, groupId: any, transactionType: any): unknown;
  value: any;
  data(to: string, arg1: any, data: any, channelId: string, groupId: string, transactionType: Transaction): unknown;
  status: ReactNode;
  groupId: any;
  recipient: string;
  address: ReactNode;
  type: ReactNode;
  // Add necessary transaction properties here
  // For example:
  id: string;
  amount: number;
  from: string;
  to: string;
  timestamp: number;
};

export type Analytics = {
  // Add necessary analytics properties here
  // For example:
  totalTransactions: number;
  totalVolume: number;
};

export type Channel = {
  balance: ReactNode;
  group: string;
  // Add necessary channel properties here
  // For example:
  id: string;
  name: string;
};

export type ChannelState = {
  transactions: any;
  // Add necessary channel state properties here
  
  balance: number;
  members: string[];
};

export type Proof = {
  // Add necessary proof properties here
  // For example:
  root: string;
  leaf: string;
  siblings: string[];
};

export type WalletExtension = {
  [x: string]: any;
  // Add necessary wallet extension properties here
  // For example:
  connect: () => Promise<void>;
  disconnect: () => Promise<void>;
  getAddress: () => Promise<string>;
  signTransaction: (transaction: Transaction) => Promise<string>;
};


