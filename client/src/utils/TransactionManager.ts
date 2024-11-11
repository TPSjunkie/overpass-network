import { TonClient, Address, toNano, SendMode, Cell, Contract } from '@ton/ton';
import { WalletContractV4 } from '@ton/ton';
import WalletExtension from '@/wrappers/wallet_extension';
import { KeyPair } from '@ton/crypto';
import WasmTransactionSender from './wasmTransactionSender';
import GroupUIManager from './GroupUIManager';
import { generateRandomKeyPair } from '@/crypto/op_crypto';
import { TonConnectUI } from '@tonconnect/ui-react';
import { Transaction } from "../types/wasm-types/index";

class TransactionManager {
  static initiateTransaction(arg0: number, selectedGroup: string) {
    throw new Error('Method not implemented.');
  }
  private groupUIManager: typeof GroupUIManager;
  private client: TonClient;
  private wallet: WalletContractV4;
  private keyPair: KeyPair;

  constructor(groupUIManager: typeof GroupUIManager, client: TonClient, wallet: WalletContractV4) {
    this.groupUIManager = groupUIManager;
    this.client = client;
    this.wallet = wallet;
    this.keyPair = {} as KeyPair;
  }

  static async getWalletBalance(address: string): Promise<string> {
    const client = new TonClient({ endpoint: 'https://toncenter.com/api/v2/jsonRPC' });
    const balance = await client.getBalance(Address.parse(address));
    return balance.toString();
  }

  static async getTransactions(address: string): Promise<Transaction[]> {
    return TransactionManager.fetchTransactions(address);
  }

  static async sendTransaction(tonConnectUI: TonConnectUI, recipient: string, amount: number): Promise<void> {
    await tonConnectUI.sendTransaction({
      validUntil: Math.floor(Date.now() / 1000) + 60,
      messages: [
        {
          address: recipient,
          amount: toNano(amount.toString()).toString(),
        },
      ],
    });
  }

  static async createChannel(walletAddress: string, channelName: string): Promise<void> {
    console.log(`Creating channel ${channelName} for wallet ${walletAddress}`);
  }

  static async joinChannel(walletAddress: string, channelId: string): Promise<void> {
    console.log(`Wallet ${walletAddress} joining channel ${channelId}`);
  }

  static async leaveChannel(walletAddress: string, channelId: string): Promise<void> {
    console.log(`Wallet ${walletAddress} leaving channel ${channelId}`);
  }

  static async updateChannelSettings(walletAddress: string, channelId: string, settings: any): Promise<void> {
    console.log(`Updating settings for channel ${channelId}`);
  }

  async initialize(): Promise<void> {
    this.keyPair = await generateRandomKeyPair();
  }

  static async fetchTransactions(address: string): Promise<Transaction[]> {
    const transactions = localStorage.getItem(`transactions_${address}`);
    if (transactions) {
      return JSON.parse(transactions);
    }
    return [];
  }

  async newTransaction(transaction: Transaction): Promise<void> {
    const { amount } = transaction;
    const walletAddress = this.wallet.address.toString();
    const walletBalance = await this.client.getBalance(Address.parse(walletAddress));
    const amountInNano = toNano(amount.toString());
    const fee = toNano('0.01');
    const totalAmount = amountInNano + fee;

    if (walletBalance < totalAmount) {
      throw new Error('Insufficient balance');
    }

    if (this.shouldUseWasm(transaction)) {
      await this.processWasmTransaction(transaction);
    } else {
      await this.processBlockchainTransaction(transaction);
    }

    if (transaction.groupId) {
      await this.groupUIManager.updateGroupBalance(transaction.groupId, amount);
    }
  }

  private shouldUseWasm(transaction: Transaction): boolean {
    return transaction.type === 'internal' || transaction.amount < parseFloat(toNano('1').toString());
  }

  private async processWasmTransaction(transaction: Transaction): Promise<void> {
    const recipient = transaction.to || transaction.recipient;
    const { amount, groupId } = transaction;
    const transactionTypeNumber = transaction.type === 'internal' ? 1 : 0;

    if (!recipient || !groupId) {
      throw new Error('Missing required transaction parameters');
    }

    WasmTransactionSender.sendTransaction(
      this.keyPair,
      this.wallet.address.toString(),
      recipient,
      Number(amount),
      transactionTypeNumber,
      groupId,
      this.wallet.address.toString(),
      0n,
      new Cell(),
      SendMode.PAY_GAS_SEPARATELY,
      this.wallet,
      new Cell()
    );
    await this.groupUIManager.updateGroupBalance(groupId, amount);

    const newTransaction = await this.groupUIManager.newTransaction({
      hash: '',
      recipient,
      amount,
      groupId,
    });

    newTransaction.status = 'completed';
    newTransaction.statusMessage = 'Transaction completed (WASM)';
    newTransaction.statusColor = 'green';
  }

  private async processBlockchainTransaction(transaction: Transaction): Promise<void> {
    const recipient = transaction.to || transaction.recipient;
    const { amount, groupId } = transaction;

    if (!recipient || !groupId) {
      throw new Error('Missing required transaction parameters');
    }

    const amountInNano = toNano(amount.toString());
    const walletExtension = new WalletExtension(this.wallet.address);
    const secretKeyHex = Buffer.from(this.keyPair.secretKey).toString('hex');

    await walletExtension.transfer({
      secretKey: secretKeyHex,
      seqno: 0,
      to: Address.parse(recipient),
      amount: amountInNano,
      sendMode: SendMode.PAY_GAS_SEPARATELY,
    });

    await this.groupUIManager.updateGroupBalance(groupId, amount);

    const newTransaction = await this.groupUIManager.newTransaction({
      hash: '',
      recipient,
      amount,
      groupId,
    });

    newTransaction.status = 'completed';
    newTransaction.statusMessage = 'Transaction completed (Blockchain)';
    newTransaction.statusColor = 'green';
  }
}

export default TransactionManager;
