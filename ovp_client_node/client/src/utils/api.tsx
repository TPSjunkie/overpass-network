import { getTonapi, setTonapiClient, useTonClient } from '@/store/tonClient'
import { useEffect } from 'react'
import { TonClient } from '@ton/ton'
import useLocalStorage from 'react-use-localstorage'
import { getHttpEndpoint } from '@orbs-network/ton-access'
import { Switch } from '@/components/ui/switch'
import { Label } from '@/components/ui/label'
import { initWASM } from "../wasm/wasmINIT/initWASM";import getOverpassData from '../hooks/getOverpassData';

export async function fetchTransactions(address: string, isTestnet: boolean) {
    const endpoint = isTestnet ? 'https://testnet.toncenter.com/api/v2/jsonRPC' : 'https://toncenter.com/api/v2/jsonRPC';
    const response = await fetch(`${endpoint}?method=getTransactions&account=${address}`);
    const data = await response.json();
    return data;
}
export function ApiSettings() {
  const [isTestnet, setTestnet] = useLocalStorage('deployerIsTestnet', 'false')
  const tonClient = useTonClient()

  useEffect(() => {
    const updateNetwork = async () => {
      try {
        console.log('Changing network');
        const network = isTestnet === 'true' ? 'testnet' : 'mainnet';
        const endpoint = await getHttpEndpoint({ network });
        
        if (!endpoint) {
          throw new Error(`Failed to get HTTP endpoint for ${network}`);
        }

        tonClient.set(new TonClient({ endpoint }));
        setTonapiClient(getTonapi(isTestnet === 'true'));

        console.log(`Network changed to ${network}`);
      } catch (error) {
        console.error('Error updating network:', error);
      }
    };

    updateNetwork();
  }, [isTestnet, tonClient]);

  const handleNetworkChange = (checked: boolean) => {
    setTestnet(String(checked));
  };

  return (
    <div className="flex items-center space-x-2">
      <Switch
        id="apiTestnetInput"
        checked={isTestnet === 'true'}
        onCheckedChange={handleNetworkChange}
      />
      <Label htmlFor="apiTestnetInput">Use Testnet</Label>
    </div>
  )
}

export { initWASM, getOverpassData };
