import { useState, useEffect, useRef, useCallback } from 'react';

interface UseIdleTimerOptions {
    timeout: number;
    onIdleCallback: () => void;
    onActiveCallback?: () => void;
}

export function useIdleTimer({ timeout, onIdleCallback, onActiveCallback }: UseIdleTimerOptions) {
    const [isIdle, setIsIdle] = useState(false);
    const timerID = useRef<number | null>(null);

    const toIdle = useCallback(() => {
        setIsIdle(true);
        onIdleCallback();
    }, [onIdleCallback]);

    const resetTimer = useCallback(() => {
        if (timerID.current) {
            clearTimeout(timerID.current);
        }
        if (isIdle) {
            setIsIdle(false);
            onActiveCallback && onActiveCallback();
        }
        timerID.current = window.setTimeout(toIdle, timeout);
    }, [timeout, toIdle, onActiveCallback, isIdle]);

    useEffect(() => {
        const events = ["mousemove", "mousedown", "keydown", "scroll", "touchstart"];
        events.forEach(event =>
            window.addEventListener(event, resetTimer, true)
        );
        timerID.current = window.setTimeout(toIdle, timeout);
        return () => {
            if (timerID.current)
                clearTimeout(timerID.current);
            events.forEach(event =>
                window.removeEventListener(event, resetTimer, true)
            );
        };
    }, [resetTimer, toIdle, timeout]);
    return { isIdle };
}
