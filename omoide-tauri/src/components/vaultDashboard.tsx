import { useEffect } from "react";
import { useVault } from "../context/vaultContext"
import { VaultEntryMetadata } from "../services/entries.types";
import { mockVaultService } from "../services/mockVaultService";
import { useAsyncState } from "../hooks/useAsyncState";
import {VaultEmpty, VaultEntries, VaultError, VaultLoading} from "./vaultComponentState";

export const VaultDashboard = () => {
    const { lock } = useVault();
    const { data: entries, error, isLoading, execute } = useAsyncState<VaultEntryMetadata[]>();
    useEffect(() => {
        execute(() => mockVaultService.getEntries());
    }, [execute]);

    if (isLoading) {
        return <VaultLoading />;
    }
    else if (error) {
        return <VaultError error={error} />;
    }
    else if (!entries || entries.length === 0) {
        return <VaultEmpty lock={lock} />;
    }
    else {
        return <VaultEntries entries={entries} lock={lock}/>
    }
}