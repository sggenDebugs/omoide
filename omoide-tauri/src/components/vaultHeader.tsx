import { LockVaultButton } from "./vaultLockButton";

export const VaultHeader = ({ lock }: { lock: () => Promise<void> }) => (
    <header className="flex justify-between items-center mb-8">
        <h1 className="text-3xl font-bold">思い出 Vault</h1>
        <LockVaultButton lock={lock} />
    </header>
);