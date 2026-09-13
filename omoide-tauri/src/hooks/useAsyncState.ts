import { useCallback, useState } from "react";

interface AsyncState<T> {
    data: T | null;
    error: string | null;
    isLoading: boolean | undefined;   
}

export function useAsyncState<T>() {
    const [asyncState, setAsyncState] = useState<AsyncState<T>>({
        data: null,
        error: null,
        isLoading: false
    });

    const execute = useCallback(async (asyncFunction: () => Promise<T>) => {
        setAsyncState({
            data: null,
            error: null,
            isLoading: true
        });

        try {
            const res = await asyncFunction();
            setAsyncState({
                data: res,
                error: null,
                isLoading: false
            });
            return res;
        } catch (err) {
            const errorMessage = err instanceof Error ? err.message : "An unknown error occured";
            setAsyncState({
                data: null,
                error: errorMessage,
                isLoading: false
            });
            throw err;
        }
    }, []);

    const reset = useCallback(() => {
        setAsyncState({
            data: null,
            error: null,
            isLoading: false,
        });
    }, []);
    return {...asyncState, execute, reset};
}