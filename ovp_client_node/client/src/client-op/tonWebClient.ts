import { getHttpEndpoint } from '@orbs-network/ton-access';
import TonWeb from 'tonweb';

export const initTonWebClient = async () => {
  try {
    // Get the decentralized RPC endpoint
    const endpoint = await getHttpEndpoint({ network: 'mainnet' });

    // Initialize the TonWeb client with the HTTP provider
    const tonweb = new TonWeb(new TonWeb.HttpProvider(endpoint));

    return tonweb;
  } catch (error) {
    console.error('Error initializing TonWeb client:', error);
    throw error;
  }
};

export const getBalance = async (tonweb: any, address: string) => {
  try {
    const balance = await tonweb.getBalance(address);
    return balance;
  } catch (error) {
    console.error('Error fetching balance from TonWeb:', error);
    throw error;
  }
};
