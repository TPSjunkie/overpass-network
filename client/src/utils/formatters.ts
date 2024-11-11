// ./src/utils/formatters.ts
export const formatDate = (date: Date): string => {
    const options: Intl.DateTimeFormatOptions = {
        year: 'numeric',
        month: 'long',
        day: 'numeric',
    };
    return date.toLocaleDateString('en-US', options);
};

export const formatTON = (amount: number): string => {
    return (amount / 1000000000).toFixed(2);
};  

