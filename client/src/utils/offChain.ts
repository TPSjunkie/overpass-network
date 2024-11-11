// ./src/utils/blockchain.ts
import { TonClient, WalletContractV4, Cell, Address, beginCell, toNano, fromNano } from "@ton/ton";
import { channelManager } from "./channelManager";


export async function fetchTransactions(address: string, client: TonClient): Promise<any[]> {
  const walletAddress = "your-wallet-address";
  const transactions = await client.getTransactions(Address.parse(walletAddress), { limit: 20 });
  return transactions;
}

export async function getChannelBalance(channelId: number, number: any): Promise<number> {
  const client = new TonClient({
    endpoint: "https://toncenter.com/api/v2/jsonRPC",
  });
  const channelAddress = channelManager.getChannelAddress(channelId.toString());
  if (!channelAddress) {
    throw new Error("Channel not found");
  }
  const balance = await client.getBalance(Address.parse(channelAddress));
  return Number(fromNano(balance));
}

export async function getChannelState(channelId: number): Promise<any> {
    const client = new TonClient({
        endpoint: "https://toncenter.com/api/v2/jsonRPC",
    });
    const channelAddress = channelManager.getChannelAddress(
        channelId.toString(),
    );
    if (!channelAddress) {
        throw new Error("Channel not found");
    }
    const state = await client.getContractState(Address.parse(channelAddress));
    return state;
}