import { createContext, ReactNode, useContext, useEffect, useState } from "react";
import { OrchestratorState } from "../services/vault.types";
import { mockVaultService } from "../services/mockVaultService";

interface VaultContext {
    state: OrchestratorState;
    unlock: (password: string) => Promise<void>;
    lock: () => Promise<void>;
}

interface VaultContextProviderProp {
    children: ReactNode;
}

const vaultContext = createContext<VaultContext | null>(null);

export const VaultContextProvider = ({ children }: VaultContextProviderProp) => {
    const [state, setState] = useState<OrchestratorState>("Locked");

    useEffect(() => {
        mockVaultService.getVaultState().then((s) => setState(s.orchestratorState))
    }, []);

    const unlock = async (password: string) => {
        await mockVaultService.unlockVault(password);
        setState("Unlocked");
    };

    const lock = async () => {
        await mockVaultService.lockVault();
        setState("Locked");
    };

    return (
        <vaultContext.Provider value={{ state, unlock, lock }}>
            {children}
        </vaultContext.Provider>
    )
};

export const useVault = () => {
    const context = useContext(vaultContext);
    if (!context){
        throw new Error("Cannot create vault context");
    }
    return context;
};