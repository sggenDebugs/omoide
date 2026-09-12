import { useState } from "react";
import { mockVaultService } from "../services/mockVaultService";

export const VaultLogin = () => {
    const [password, setPassword] = useState('');
    const [error, setError] = useState<string | null>(null);

    const handleUnlock = async () => {
        try {
            setError(null);
            await mockVaultService.unlockVault(password);
            console.log('Vault unlocked! Transitioning to dashboard...');
        } catch (err) {
            setError(err instanceof Error ? err.message : 'An unknown error occurred');
        }
    };

    return (
        <div className="p-4">
            <h2 className="text-xl font-bold mb-4">Unlock Omoide</h2>
            <input
                type="password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                className="border p-2 w-full mb-2"
                placeholder="Master Password"
            />
            <button onClick={handleUnlock} className="bg-blue-500 text-white p-2 rounded">
                Unlock
            </button>
            {error && <p className="text-red-500 mt-2">{error}</p>}
        </div>
    );
}