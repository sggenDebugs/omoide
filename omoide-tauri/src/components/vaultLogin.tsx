import { useState } from "react";
import { useVault } from "../context/vaultContext";

export const VaultLogin = () => {
    const { unlock } = useVault();
    const [password, setPassword] = useState('');
    const [error, setError] = useState<string | null>(null);
    const [isLoading, setIsLoading] = useState(false);

    const handleUnlock = async () => {
        setIsLoading(true);
        setError(null);
        try {
            await unlock(password);
            console.log('Vault unlocked! Transitioning to dashboard...');
        } catch (err) {
            setError(err instanceof Error ? err.message : 'An unknown error occurred');
            setPassword('');
        } finally {
            setIsLoading(false);
        }
    };

    return (
        <div className="flex flex-col items-center justify-center h-screen bg-gray-900 text-white">
            <h1 className="text-2xl font-bold mb-4">Omoide</h1>
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