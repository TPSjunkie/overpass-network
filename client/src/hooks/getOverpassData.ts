// ./src/hooks/getOverpassData.ts
import { useEffect, useState } from "react";
import type { Channel } from "../types/wasm-types/index";
import exp from "constants";

export const useOverpassData = (query: string) => {
  const [channels, setChannels] = useState<Channel[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchData = async () => {
      try {
        const response = await fetch('YOUR_API_ENDPOINT_HERE');
        const data: Channel[] = await response.json();
        setChannels(data);
      } catch (error) {
        setError(error instanceof Error ? error.message : 'An unknown error occurred');
      } finally {
        setLoading(false);
      }
    };
    fetchData();
  }, []);

  return { channels, loading, error };
}

export default useOverpassData;
