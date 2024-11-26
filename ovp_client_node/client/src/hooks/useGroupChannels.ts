// ./src/hooks/useGroupChannels.ts
import { useQuery } from "@tanstack/react-query";

export const useGroupChannels = (groupId: number) => {
    return useQuery({
        queryKey: ["groupChannels", groupId],
        queryFn: async () => {
            const response = await fetch(`/api/groups/${groupId}/channels`);
            return response.json();
        },
    });
};

export default useGroupChannels;