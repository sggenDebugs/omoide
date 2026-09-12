import { VaultState } from "./vault.types"

// Mock vault state
let currentState: VaultState = {
    orchestratorState: "Locked",
    retriesRemaining: 2,
    nextSRS: 300
};

export const mockVaultService = {
    /**
     * Current vault state get processing stub
     */
    getVaultState: async (): Promise<VaultState> => {
        // Mock invoke function call
        await new Promise((resolve) => setTimeout(resolve, 100));
        return { ...currentState };
    },

    /**
     * Unlock vault stub
     */
    unlockVault: async (password: string): Promise<void> => {
        await new Promise((resolve) => setTimeout(resolve, 500));

        if (password == "omoide") {
            currentState.orchestratorState = "Unlocked";
            currentState.retriesRemaining = 3;
            currentState.nextSRS = 300;
        } else {
            currentState.retriesRemaining = Math.max(
                0,
                (currentState.retriesRemaining || 0) - 1
            );
            throw new Error("Wrong password!");
        }
    },

    /**
     * Lock vault stub
     */
    lockVault: async (): Promise<void> => {
        await new Promise((resolve) => setTimeout(resolve, 500));
        currentState.orchestratorState = "Locked";
        currentState.retriesRemaining = 3;
    },

    /**
     * SRS reprompt checking stub
     */
    checkSRSReprompt: async (): Promise<boolean> => {
        await new Promise((resolve) => setTimeout(resolve, 100));
        const isDue = Math.random() > 0.7;
        if (isDue && currentState.orchestratorState == "Locked") {
            currentState.orchestratorState = "AwaitingReprompt";
        }
        return isDue;
    }
};