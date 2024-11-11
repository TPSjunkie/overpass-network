// src/pages/WalletOP.tsx
import { Base64 } from '@tonconnect/protocol';
import type { Address, beginCell, contractAddress as calculateContractAddress, Cell, StateInit } from 'ton';

import React, { useEffect, useState } from "react";
import { useOverpassData } from "../hooks/getOverpassData";
import type { Channel } from "../types/wasm-types";
import { useTonConnect } from "../hooks/useTonConnect";

interface GroupChannel {
  group: string;
  channels: string[];
}

interface GroupBalance {
  group: string;
  balances: number;
}


interface Proof {
  root: string;
  path: string[];
  values: string[];
}


interface Transaction {
  id: string;
  amount: number;
  date: string;
  channelId: number;
  hash: string;
  nonce: number;
  payload: string;
  recipient: string;
  sender: string;
  senderSignature: string;
  seqno: number;
  status: string;
  statusColor: string;
  statusMessage: string;
  timestamp: number;
  type: string;
  merkleRoot: string;
  tree: {
    root: string;
    leaves: any[];
  };
  description: string;
}

const WalletOP: React.FC = () => {
  const { walletInfo, isLoading, error } = useTonConnect();
  const [channelId] = useState<number>(0);
  const [channelBalance] = useState<number>(0);
  const [transaction] = useState<Transaction>({
    id: "",
    amount: 0,
    date: "",
    channelId: 0,
    hash: "",
    nonce: 0,
    payload: "",
    recipient: "",
    sender: "",
    senderSignature: "",
    seqno: 0,
    status: "pending",
    statusColor: "",
    statusMessage: "",
    timestamp: 0,
    type: "incoming",
    merkleRoot: "",
    tree: {
      root: "",
      leaves: [],
    },
    description: "",
  });
  const [, setUnusedState] = useState<number>(0);
  const [proof] = useState<Proof>({
    root: "",
    path: [],
    values: [],
  });
  const { channels } = useOverpassData("");

  const [groupChannels, setGroupChannels] = useState<GroupChannel[]>([]);
  const [groupBalances, setGroupBalances] = useState<GroupBalance[]>([]);
  const [selectedGroup] = useState<string>("");
  const [selectedChannel] = useState<Channel | null>(null);
  useEffect(() => {
    if (channels && channels.length > 0) {
      const groupedChannels = channels.reduce<{ [key: string]: Channel[] }>(
        (acc, channel) => {
          const group = channel.group || "Ungrouped";
          if (!acc[group]) {
            acc[group] = [];
          }
          acc[group].push(channel);
          return acc;
        },
        {}
      );
      setGroupChannels(
        Object.entries(groupedChannels).map(([group, channels]) => ({
          group,
          channels: channels.map(channel => channel.id),
        }))
      );
    }
  }, [channels]);
  useEffect(() => {
    if (groupChannels && groupChannels.length > 0) {
      const groupedBalances = groupChannels.reduce(
        (acc: { [key: string]: number }, groupChannel: GroupChannel) => {
          const group = groupChannel.group;
          if (!acc[group]) {
            acc[group] = 0;
          }
          acc[group] += groupChannel.channels.length;
          return acc;
        },
        {}
      );
      setGroupBalances(
        Object.entries(groupedBalances).map(([group, balances]) => ({
          group,
          balances,
        }))
      );
    }
  }, [groupChannels]);

  useEffect(() => {
    if (selectedGroup) {
      // Fetch or compute data here
    } else {
      // Reset selections
    }
  }, [selectedGroup]);

  return (
    <div>
      <h1>WalletOP Component</h1>
      <div>
        <h2>Wallet Information</h2>
        <p>Address: {walletInfo?.address}</p>
        <p>Balance: {walletInfo?.balance}</p>
        <p>Is Loading: {isLoading ? "Yes" : "No"}</p>
        <p>Error: {error}</p>
        <h2>Channel Information</h2>
        <p>Channel ID: {channelId}</p>
        <p>Channel Balance: {channelBalance}</p>
        <h2>Transaction Information</h2>
        <p>Transaction ID: {transaction.id}</p>
        <p>Transaction Amount: {transaction.amount}</p>
        <p>Transaction Date: {transaction.date}</p>
        <p>Transaction Channel ID: {transaction.channelId}</p>
        <p>Transaction Hash: {transaction.hash}</p>
        <p>Transaction Nonce: {transaction.nonce}</p>
        <p>Transaction Payload: {transaction.payload}</p>
        <p>Transaction Recipient: {transaction.recipient}</p>
        <p>Transaction Sender: {transaction.sender}</p>
        <p>Transaction Sender Signature: {transaction.senderSignature}</p>
        <p>Transaction Seqno: {transaction.seqno}</p>
        <p>Transaction Status: {transaction.status}</p>
        <p>Transaction Status Color: {transaction.statusColor}</p>
        <p>Transaction Status Message: {transaction.statusMessage}</p>
        <p>Transaction Timestamp: {transaction.timestamp}</p>
        <p>Transaction Type: {transaction.type}</p>
        <p>Transaction Merkle Root: {transaction.merkleRoot}</p>
        <h2>Transaction Tree Information</h2>
        <p>Transaction Tree Root: {transaction.tree.root}</p>
        <p>Transaction Tree Leaves: {transaction.tree.leaves.length}</p>
        <h2>Transaction Tree Proof Information</h2>
        <p>Transaction Tree Proof Path: {proof.path.length}</p>
        <p>Transaction Tree Proof Values: {proof.values.length}</p>
        <h2>Transaction Tree Proof Root</h2>
        <p>Transaction Tree Proof Root: {proof.root}</p>
        <h2>Group Channels</h2>
        <ul>
          {groupChannels.map((groupChannel) => (
            <li key={groupChannel.group}>
              {groupChannel.group}
            </li>
          ))}
        </ul>
        <h2>Group Balances</h2>
        <ul>
          {groupBalances.map((groupBalance, index) => (
            <li key={index}>
              {groupBalance.group}: {groupBalance.balances} TON
            </li>
          ))}
        </ul>
        <h2>Selected Group</h2>
        <p>Selected Group: {selectedGroup}</p>
        <h2>Selected Channel</h2>
        <p>Selected Channel: {selectedChannel?.name}</p>
        <h2>Selected Transaction</h2>
        <p>Selected Transaction: {transaction.id}</p>
        <h2>Selected Group Channels</h2>
        <ul>
          {groupChannels.map((groupChannel) => (
            <li key={groupChannel.group}>
              {groupChannel.group}: {groupChannel.channels.length} channels
            </li>
          ))}
        </ul>
        <h2>Selected Group Balance</h2>
        <ul>
          {groupBalances.map((groupBalance, index) => (
            <li key={index}>
              {groupBalance.group}: {groupBalance.balances} TON
            </li>
          ))}
        </ul>
        <h2>Channel Information</h2>
        {channels && channels.map((channel: Channel) => (
          <div key={channel.id}>
            <h3>{channel.name}</h3>
            <p>Balance: {channel.balance}</p>
          </div>
        ))}
      </div>
    </div>
  );
};

export default WalletOP;