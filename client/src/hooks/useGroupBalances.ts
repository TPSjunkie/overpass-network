// ./src/hooks/useGroupBalances.ts

/// This file contains the logic for fetching group balances from the OP Client.

import { useQuery } from "@tanstack/react-query";
import { OpClient } from "../utils/op_client";export interface GroupBalance {
  groupId: string;
  balance: number;
}

export const useGroupBalances = () => {
    return useQuery<GroupBalance[], Error>({
        queryKey: ["groupBalances"],
        queryFn: async () => {
            try {
                const opClient = new OpClient();
                const provider = new OpClient.ContractProvider(); // Modified this line
                const balances = await opClient.fetchGroupBalances(provider);
                if (Array.isArray(balances)) {
                    return balances as GroupBalance[];
                } else {
                    throw new Error("Unexpected response format");
                }
            } catch (error) {
                throw new Error("Failed to fetch group balances");
            }
        },
        staleTime: 5 * 60 * 1000, // 5 minutes        gcTime: 10 * 60 * 1000, // 10 minutes
    });
};export default useGroupBalances;
