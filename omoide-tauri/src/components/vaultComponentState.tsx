import { useState } from "react";
import { useSRSCountdown } from "../hooks/useSRSCountdown";
import { VaultEntryMetadata } from "../services/entries.types"
import { mockVaultService } from "../services/mockVaultService";
import { VaultHeader } from "./vaultHeader";

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
    <div className="min-h-screen bg-gray-900 text-white p-8">
        <VaultHeader lock={lock} />
        <p className="text-gray-400">Your vault is empty. Add your first entry!</p>
    </div>
);

export const VaultEntries = ({ entries, lock }: {
    entries: VaultEntryMetadata[];
    lock: () => Promise<void>;
}) => {
    const [intervalVal] = useState<number>(1000); 
    const [ count, {startSRSCountdown, stopSRSCountdown, resetSRSCountdown} ] = useSRSCountdown({
    countStart: 60,
    intervalMs: intervalVal,
  }); 
    return (
    <div className="min-h-screen bg-gray-900 text-white p-8">
        <VaultHeader lock={lock} />
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
        <div>
            <p className="font-semibold text-lg truncate">Count: {count}</p>
            <button onClick={startSRSCountdown}>start</button>
            <button onClick={stopSRSCountdown}>stop</button>
            <button onClick={resetSRSCountdown}>reset</button>
        </div>
    </div>
)
};