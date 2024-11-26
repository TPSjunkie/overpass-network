// ./src/services/setChannels.ts
export const setChannels = (channels: string[]) => {
    localStorage.setItem("channels", JSON.stringify(channels));
};
export default setChannels;
