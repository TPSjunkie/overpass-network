// ./src/utils/messageBuilder.ts
import {} from '@/utils/bocutils';
export const buildMessage = (message: string) => {
    return `Hello, ${message}!`;
};
export const buildMessageWithParams = (params: Record<string, unknown>) => {
    const { name, age } = params;
    return `Hello, ${name}! You are ${age} years old.`;
};
export const buildMessageWithParamsAndOptions = (
    params: Record<string, unknown>,
    options: Record<string, unknown>,
) => {
    const { name, age } = params;
    const { greeting, message } = options;
    return `${greeting}, ${name}! ${message} You are ${age} years old.`;
};
