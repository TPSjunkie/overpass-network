// ./src/services/groupChannels.tsx

/// This file contains the logic for fetching group channels from the OP Client.
import { useQuery } from "@tanstack/react-query";
export const GroupChannels = () => {
    return useQuery({
        queryKey: ["groupChannels"],
        queryFn: GroupChannels,
    });
};
