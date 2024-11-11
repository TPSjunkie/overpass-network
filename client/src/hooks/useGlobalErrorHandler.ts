// src/hooks/useGlobalErrorHandler.ts

import { useEffect } from 'react';
import { toast } from 'react-toastify';

const useGlobalErrorHandler = () => {
  useEffect(() => {
    const handleUnhandledRejection = (event: PromiseRejectionEvent) => {
      console.error('Unhandled promise rejection:', event.reason);
      toast.error('An unexpected error occurred.');
    };

    window.addEventListener('unhandledrejection', handleUnhandledRejection);

    return () => {
      window.removeEventListener('unhandledrejection', handleUnhandledRejection);
    };
  }, []);
};

export default useGlobalErrorHandler;
