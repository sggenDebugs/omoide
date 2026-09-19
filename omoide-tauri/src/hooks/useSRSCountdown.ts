import { Dispatch, SetStateAction, useCallback, useEffect, useRef, useState } from 'react'

type CountdownOptions = {
  countStart: number

  intervalMs?: number
  isIncrement?: boolean

  countStop?: number
}

type SRSCountdownControllers = {
  startSRSCountdown: () => void
  stopSRSCountdown: () => void
  resetSRSCountdown: () => void
}

interface UseCounterReturn {
    count: number;
    decrementCount: () => void;
    reset: () => void;
    setCount: Dispatch<SetStateAction<number>>;
}

interface UseBooleanReturn {
    val: boolean;
    setVal: Dispatch<SetStateAction<boolean>>;
    setTrue: () => void;
    setFalse: () => void;
    toggle: () => void;
}

function useCounter(initialVal: number): UseCounterReturn {
    const [count, setCount] = useState(initialVal);
    const decrementCount = useCallback(() => {
        setCount((x: number) => x - 1);
    }, []);
    const reset = useCallback(() => {
        setCount(initialVal);
    }, [initialVal]);
    return {
        count,
        decrementCount,
        reset,
        setCount
    };
}

function useInterval(callback: () => void, delay: number | null) {
    const storedCallback = useRef(callback);
    useEffect(() => {
        storedCallback.current = callback;
    }, [callback]);
    useEffect(() => {
        if (delay === null) return;
        const id = setInterval(() => {
            storedCallback.current();
        }, delay);
        return () => {
            clearInterval(id);
        }
    }, [delay]);
}

function useBoolean(defaultValue: boolean = false): UseBooleanReturn {
    const [val, setVal] = useState(defaultValue);
    const setTrue = useCallback(() => {
        setVal(true);
    }, []);
    const setFalse = useCallback(() => {
        setVal(false);
    }, []);
    const toggle = useCallback(() => {
        setVal((x: boolean) => !x);
    }, []);

    return {
        val,
        setVal,
        setTrue,
        setFalse,
        toggle
    };
}

export function useSRSCountdown({
  countStart,
  intervalMs = 1000,
  isIncrement = false,
}: CountdownOptions): [number, SRSCountdownControllers] {
  const {
    count,
    decrementCount,
    reset: resetCounter,
  } = useCounter(countStart)

  const {
    val: isCountdownRunning,
    setTrue: startSRSCountdown,
    setFalse: stopSRSCountdown,
  } = useBoolean(false)

  const resetSRSCountdown = useCallback(() => {
    stopSRSCountdown()
    resetCounter()
  }, [stopSRSCountdown, resetCounter])

  const countdownCallback = useCallback(() => {
    if (count === 0) {
      stopSRSCountdown()
      return
    }
      decrementCount()
  }, [count, decrementCount, isIncrement, stopSRSCountdown]);

  useInterval(countdownCallback, isCountdownRunning ? intervalMs : null)

  return [count, { startSRSCountdown, stopSRSCountdown, resetSRSCountdown }]
}