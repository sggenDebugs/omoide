import { VaultEntryMetadata } from "../services/entries.types"
import { mockVaultService } from "../services/mockVaultService";

export const VaultLoading = () => (
    <div className="min-h-screen bg-gray-900 text-white flex items-center justify-center">
        <h1>Loading your memories...</h1>
    </div>
);

export const VaultError = ({ error }: { error: string }) => (
    <div className="min-h-screen bg-gray-900 text-white flex items-center justify-center p-8">
        <div className="text-red-500">
            <h2 className="text-xl font-bold mb-2">Failed to load vault</h2>
            <p>{error}</p>
        </div>
    </div>
);

export const VaultEmpty = ({ lock }: { lock: () => Promise<void> }) => (
    <div className="min-h-screen bg-gray-900 text-white p-8 flex flex-col items-center justify-center">
        <h1 className="text-3xl font-bold mb-4">思い出 Vault</h1>
        <p className="text-gray-400">Your vault is empty. Add your first entry!</p>
        <button onClick={lock} className="mt-8 bg-red-600 hover:bg-red-500 px-4 py-2 rounded">
            Lock Vault
        </button>
    </div>
);

export const VaultEntries = ({ entries, lock }: {
    entries: VaultEntryMetadata[]; 
    lock: () => Promise<void>;
}) => (
    <div className="min-h-screen bg-gray-900 text-white p-8">
        <header className="flex justify-between items-center mb-8">
            <h1 className="text-3xl font-bold">思い出 Vault</h1>
            <button
                onClick={lock}
                className="bg-red-600 hover:bg-red-500 px-4 py-2 rounded transition-colors"
            >
                Lock Vault
            </button>
        </header>

        <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
            {entries.map((entry) => (
                <div key={entry.id} className="bg-gray-800 p-4 rounded-lg border border-gray-700 flex flex-col h-40">
                    <div className="flex-grow">
                        <h3 className="font-semibold text-lg truncate">{entry.title}</h3>
                        <p className="text-gray-400 text-sm truncate">{entry.username}</p>
                        {entry.url && (
                            <a href={entry.url} target="_blank" rel="noreferrer" className="text-blue-400 text-xs hover:underline block truncate">
                                {entry.url}
                            </a>
                        )}
                    </div>
                    <div className="mt-2 pt-2 border-t border-gray-700">
                        <button
                            onClick={() => mockVaultService.copyPassword(entry.id)}
                            className="w-full bg-gray-700 hover:bg-gray-600 py-1.5 rounded text-sm">
                            Copy Password
                        </button>
                    </div>
                </div>
            ))}
        </div>
    </div>
);