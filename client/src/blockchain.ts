// ./src/utils/blockchain.ts
import { TonClient, Address, fromNano } from "@ton/ton";
import { channelManager } from "./channelManager";


export async function fetchTransactions(address: string, client: TonClient): Promise<any[]> {
  const transactions = await client.getTransactions(Address.parse(address), { limit: 20 });
  return transactions;
}
export async function getChannelState(
  channelId: string,
  client: TonClient,
): Promise<any> {
  const channelAddress = channelManager.getChannelAddress(channelId);
  if (!channelAddress) {
    throw new Error(`Channel with ID ${channelId} not found`);
  }
  const channelData = await client.getContractState(
    Address.parse(channelAddress)
  );
  return channelData;
}

export async function getChannelBalance(
  channelId: string,
  client: TonClient,
): Promise<number> {
  const channelAddress = channelManager.getChannelAddress(channelId);
  if (!channelAddress) {
    throw new Error(`Channel with ID ${channelId} not found`);
  }
  const channelData = await client.getContractState(
    Address.parse(channelAddress),
  );
  const balance = channelData.balance;
  return fromNano(balance.toString());
}
export async function getChannelAddress(
  channelId: string,
  client: TonClient,
)