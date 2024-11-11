import { TonClient4 } from 'ton';
import { getHttpV4Endpoint } from '@orbs-network/ton-access';

export const initTonClientV4 = async () => {
  try {
    // Get the decentralized RPC endpoint
    const endpoint = await getHttpV4Endpoint();

    // Initialize the TonClient4 with the endpoint
    const client4 = new TonClient4({ endpoint });

    return client4;
  } catch (error) {
    console.error('Error initializing TonClient4:', error);
    throw error;
  }
};

export const getLatestBlock = async (client4: TonClient4) => {
  try {
    const latestBlock = await client4.getLastBlock();
    return latestBlock;
  } catch (error) {
    console.error('Error fetching latest block:', error);
    throw error;
  }
};
