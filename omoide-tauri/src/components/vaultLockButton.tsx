export const LockVaultButton = ({ lock }: { lock: () => Promise<void> }) => (
    <button
        onClick={lock}
        className="bg-red-600 hover:bg-red-500 px-4 py-2 rounded transition-colors"
    >
        Lock Vault
    </button>
);