import { VaultEntryMetadata } from "./entries.types";
import { VaultAuthState } from "./vault.types"

// Mock vault state
let currentState: VaultAuthState = {
    orchestratorState: "Locked",
    retriesRemaining: 2,
    nextSRS: 300
};

const mockEntries: VaultEntryMetadata[] = [
    { id: "1", title: "Sample Entry 1", username: "sampleEntry1", createdAt: 1718200000, updatedAt: 1718200000 },
    { id: "2", title: "Sample Entry 2", username: "sampleEntry2", url: "https://www.sample2.com", createdAt: 20000000, updatedAt: 20000000 },
    { id: "3", title: "Sample Entry 3", username: "sampleEntry3", url: "https://www.sample3.com", createdAt: 9999999999, updatedAt: 9999999999 }
]

let nextSRSTime: number | null = null;

export const mockVaultService = {
    /**
     * Current vault state get processing stub
     */
    getVaultState: async (): Promise<VaultAuthState> => {
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

            if (currentState.nextSRS) {
                const timeoutMillis = currentState.nextSRS * 1000;
                nextSRSTime = Date.now() + timeoutMillis;
            }
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
        if (isDue && currentState.orchestratorState == "Unlocked") {
            currentState.orchestratorState = "AwaitingReprompt";
        }
        return isDue;
    },

    /**
     * Get entries stub
     */
    getEntries: async (): Promise<VaultEntryMetadata[]> => {
        await new Promise((resolve) => setTimeout(resolve, 300));
        if (currentState.orchestratorState != "Unlocked") {
            throw new Error("Vault is not yet unlocked!");
        }
        return mockEntries;

    },

    /**
     * Copy password stub
     */
    copyPassword: async (id: string): Promise<void> => {
        await new Promise((resolve) => setTimeout(resolve, 300));
        console.log(`[REDACTED] password copied for ${id}.`);
    },

    /**
    * Simulates user activity, resetting the timer.
    */
    resetRepromptTimer: async (): Promise<void> => {
        if (currentState.orchestratorState === 'Unlocked' && nextSRSTime && currentState.nextSRS) {
            const timeoutMs = currentState.nextSRS * 1000;
            nextSRSTime = Date.now() + timeoutMs;
        }
    },

    /**
    * Returns the absolute timestamp for the next reprompt.
    * Useful for calculating the progress bar width in React.
    */
    getNextRepromptTimestamp: async (): Promise<number | null> => {
        return nextSRSTime;
    }
};
