import exp from "constants";

// ./src/utils/channelManager.ts
export const channelManager = {
  channels: new Map<string, string>(),

  addChannel: (channelId: string, channelAddress: string): void => {
    channelManager.channels.set(channelId, channelAddress);
  },

  getChannelAddress: (channelId: string): string | undefined => {
    return channelManager.channels.get(channelId);
  },

  removeChannel: (channelId: string): boolean => {
    return channelManager.channels.delete(channelId);
  },

  updateChannelAddress: (channelId: string, newAddress: string): boolean => {
    if (channelManager.channels.has(channelId)) {
      channelManager.channels.set(channelId, newAddress);
      return true;
    }
    return false;
  },

  getAllChannels: (): Map<string, string> => {
    return new Map(channelManager.channels);
  },

  clearAllChannels: (): void => {
    channelManager.channels.clear();
  },

  getChannelCount: (): number => {
    return channelManager.channels.size;
  },

  channelExists: (channelId: string): boolean => {
    return channelManager.channels.has(channelId);
  },

  getChannelIds: (channelId: number): string[] => {
    return Array.from(channelManager.channels.keys());
  },

  getChannelAddresses: (): string[] => {
    return Array.from(channelManager.channels.values());
  }
};

export default channelManager;
