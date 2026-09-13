import { useState } from "react";
import { useVault } from "../context/vaultContext";
import { useAsyncState } from "../hooks/useAsyncState";

export const VaultLogin = () => {
    const { unlock } = useVault();
    const [password, setPassword] = useState('');
    const { error, isLoading, execute } = useAsyncState<void>();

    const handleUnlock = async () => {
        try {
            await execute(() => unlock(password));
            console.log('Vault unlocked! Transitioning to dashboard...');
        } catch (err) {
            setPassword('');
        }
    };

    return (
        <div className="flex flex-col items-center justify-center h-screen bg-gray-900 text-white">
            <h1 className="text-2xl font-bold mb-4">思い出</h1>
            <input
                type="password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                className="p-2 rounded bg-gray-800 border border-gray-700 mb-2 w-64"
                placeholder="Master Password"
                disabled={isLoading}
            />
            <button
                onClick={handleUnlock}
                disabled={isLoading}
                className="bg-blue-600 hover:bg-blue-500 p-2 rounded w-64 transition-colors"
            >
                {isLoading ? 'Deriving Key...' : 'Unlock'}
            </button>
            {error && <p className="text-red-400 mt-2 text-sm">{error}</p>}
        </div>
    );
}