// src/utils/GroupUIManager.ts

import { getChannelBalance } from './offChain';

class GroupUIManager {
  [x: string]: any;
    private groups: {
      checking: Set<string>;
      savings: Set<string>;
      custom: Set<string>;
    };
  
    constructor() {
      this.groups = {
        checking: new Set(),
        savings: new Set(),
        custom: new Set()
      };
    }
  

  assignChannelToGroup(channelId: number, groupType: string) {
    switch (groupType.toLowerCase()) {
      case 'checking':
        this.groups.checking.add(channelId.toString());
        break;
      case 'savings':
        this.groups.savings.add(channelId.toString());
        break;
      case 'custom':
        this.groups.custom.add(channelId.toString());
        break;
      default:
        throw new Error(`Unknown group type: ${groupType}`);
    }
  }

  async getGroupBalances(userFriendlyAddress: string): Promise<{
    totalBalance: bigint; checking: bigint; savings: bigint; custom: bigint; total: bigint 
}> {
    const checkingBalance = await this.getGroupBalance('checking', 0, this.groups.checking.size);
    const savingsBalance = await this.getGroupBalance('savings', 0, this.groups.savings.size);
    const customBalance = await this.getGroupBalance('custom', 0, this.groups.custom.size);
    const totalBalance = checkingBalance + savingsBalance + customBalance;

    return {
      totalBalance,
      checking: checkingBalance,
      savings: savingsBalance,
      custom: customBalance,
      total: totalBalance
    };
  }

  async getGroupBalancesPaginated(page: number, pageSize: number) {
    const startIndex = (page - 1) * pageSize;
    const endIndex = startIndex + pageSize;

    const checkingBalance = await this.getGroupBalance('checking', startIndex, endIndex);
    const savingsBalance = await this.getGroupBalance('savings', startIndex, endIndex);
    const customBalance = await this.getGroupBalance('custom', startIndex, endIndex);

    return {
      breakdown: {
        checking: checkingBalance,
        savings: savingsBalance,
        custom: customBalance
      },
      totalBalance: checkingBalance + savingsBalance + customBalance,
    };
  }

  private async getGroupBalance(groupType: 'checking' | 'savings' | 'custom', startIndex: number, endIndex: number): Promise<bigint> {
    const group = this.groups[groupType];
    let total = 0n;

    const channelIds = Array.from(group).slice(startIndex, endIndex);
    for (const channelId of channelIds) {
      const balance = await this.getChannelBalance(parseInt(channelId));
      total += BigInt(balance);
    }

    return total;
  }

  removeChannelFromGroup(channelId: number, groupType: string) {
    const group = this.groups[groupType.toLowerCase() as keyof typeof this.groups];
    group.delete(channelId.toString());
  }

  getChannelsInGroup(groupType: string): number[] {
    return Array.from(this.groups[groupType.toLowerCase() as keyof typeof this.groups]).map(Number);
  }

  moveChannelBetweenGroups(channelId: number, fromGroup: string, toGroup: string) {
    this.removeChannelFromGroup(channelId, fromGroup);
    this.assignChannelToGroup(channelId, toGroup);
  }

  async getChannelBalance(channelId: number): Promise<number> {
    return await getChannelBalance(channelId, 0);
  }

  clearGroup(groupType: string) {
    this.groups[groupType.toLowerCase() as keyof typeof this.groups].clear();
  }

  getAllGroups(): string[] {
    return Object.keys(this.groups);
  }
  async getTotalBalance(): Promise<bigint> {
    const balances = await this.getGroupBalances('');
    return balances.totalBalance;
  }
  getGroupSize(groupType: string): number {
    return this.groups[groupType.toLowerCase() as keyof typeof this.groups].size;
  }
}

export default new GroupUIManager();
