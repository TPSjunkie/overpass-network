// ./components.tsx
export const RetryableError = ({ 
    message, 
    onRetry 
  }: { 
    message: string; 
    onRetry: () => void;
  }) => (
    <div className="flex flex-col items-center justify-center p-4 text-center">
      <div className="text-xs mb-4 px-4 py-2 bg-[#0f380f] text-[#9bbc0f]">
        {message}
      </div>
      <button
        onClick={onRetry}
        className="px-4 py-2 text-xs border border-[#0f380f] hover:bg-[#0f380f] hover:text-[#9bbc0f] transition-colors"
      >
        PRESS A TO RETRY
      </button>
    </div>
  );
  
  export const RetroLoading = () => (
    <div className="flex flex-col items-center justify-center p-4">
      <div className="text-xs animate-pulse">
        LOADING...
      </div>
      <div className="mt-2 flex space-x-1">
        {[0, 1, 2].map((i) => (
          <div
            key={i}
            className="w-2 h-2 bg-[#0f380f] animate-bounce"
            style={{ animationDelay: `${i * 200}ms` }}
          />
        ))}
      </div>
    </div>
  );