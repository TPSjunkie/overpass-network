// ./src/services/setBalance.ts
export const setBalance = (balance: number) => {
    localStorage.setItem("balance", balance.toString());
};

export default setBalance;