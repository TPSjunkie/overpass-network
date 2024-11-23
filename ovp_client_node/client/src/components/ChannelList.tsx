import React, { useState, useEffect } from 'react';
import { useTonConnect } from '../hooks/useTonConnect';

interface Channel {
  id: number;
  balance: string;
  group: string;
}

const ChannelList: React.FC = () => {
  const [channels, setChannels] = useState<Channel[]>([]);
  const [groups, setGroups] = useState<string[]>(['checking', 'savings', 'custom']);
  const { walletInfo } = useTonConnect();

  useEffect(() => {
    if (walletInfo) {
      fetchChannels();
    }
  }, [walletInfo]);

  const fetchChannels = async () => {
    if (walletInfo) {
      const walletExt = await walletInfo.getWallet();
      const channelCount = await walletExt.get_channel_count();
      
      const fetchedChannels: Channel[] = [];
      for (let i = 1; i <= channelCount; i++) {
        try {
          const channelState = await walletExt.get_channel(i);
          if (channelState) {
            fetchedChannels.push({
              id: i,
              balance: channelState.balance.toString(),
              group: await walletExt.get_channel_group(i) || 'checking',
            });
          }
        } catch (error) {
          console.error(`Error fetching channel ${i}:`, error);
        }
      }
      setChannels(fetchedChannels);
    }
  };

  const updateChannelGroup = (channelId: number, newGroup: string) => {
    setChannels(channels.map((channel) => 
      channel.id === channelId ? { ...channel, group: newGroup } : channel
    ));
  };

  return (
    <div className="channel-list bg-pip-boy-panel p-4 rounded-lg shadow-pip-boy">
      <h3 className="text-xl font-semibold text-pip-boy-green mb-4">Channels</h3>
      {channels.map((channel) => (
        <div key={channel.id} className="channel-item mb-2 flex justify-between items-center">
          <span>Channel {channel.id}: {channel.balance} TON</span>
          <select 
            value={channel.group}
            onChange={(e) => updateChannelGroup(channel.id, e.target.value)}
            className="bg-pip-boy-dark-green text-pip-boy-text p-1 rounded"
          >
            {groups.map((group) => (
              <option key={group} value={group}>{group}</option>
            ))}
          </select>
        </div>
      ))}
    </div>
  );
};

export default ChannelList;