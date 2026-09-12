export type OrchestratorState = "Locked" | "Unlocked" | "AwaitingReprompt";
export type EncryptionAlgorithm = 'AES-256-GCM';

export interface VaultState {
    /**
     * Auth orchestrator state:
     * - Locked
     * - Unlocked
     * - AwaitingReprompt
     */
    orchestratorState: OrchestratorState;

    /**
     * Number of remaining master password entry retries.
     */
    retriesRemaining: number;

    /**
     * Current SRS interval in hours. (optional)
     */
    currentInterval?: number;

    /**
     * Seconds until the next SRS challenge is due. (optional)
     */
    nextSRS?: number;

    /**
     * Vault state error code (optional)
     */
    lastError?: string;
}

export interface ArgonKDFConfig {
    /**
     * Memory used.
     */
    memoryCost: number;

    /**
     * Iterations of derivation.
     */
    iterations: number;

    /**
     * Number of derivation threads to be used.
     */
    parallelCost: number;
}

export interface VaultInfo {
    /**
     * Argon KDF config.
     */
    kdfConfig: ArgonKDFConfig;

    /**
     * Entry encryption algorithm.
     */
    encryptionAlgorithm: EncryptionAlgorithm;

    /**
     * Vault version.
     */
    version: string;

    /**
     * Is vault initialized?.
     */
    isInitialized: boolean;
}

export interface VaultContext {
    state: OrchestratorState;
    unlock: (password: string) => Promise<void>;
    lock: () => Promise<void>;
}