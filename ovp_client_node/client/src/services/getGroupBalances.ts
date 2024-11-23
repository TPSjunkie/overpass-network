// ./src/components/GroupBalances.tsx

/// This file contains the logic for fetching group balances from the OP Client.
import { useQuery } from "@tanstack/react-query";

export const GroupBalances = () => {
  return useQuery({
    queryKey: ["groupChannels"],
    queryFn: GroupBalances,
  });
};