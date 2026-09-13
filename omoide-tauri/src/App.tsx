import "./App.css";
import { useVault, VaultContextProvider } from "./context/vaultContext";
import { VaultLogin } from "./components/vaultLogin";
import { VaultDashboard } from "./components/vaultDashboard";

const AppContent = () => {
  const {state} = useVault();

  switch(state) {
    case "Locked":
      return <VaultLogin />;
    case "Unlocked":
      return <VaultDashboard />;
  }
}
function App() {
  return (
    <VaultContextProvider>
      <AppContent />
    </VaultContextProvider>
  );
}

export default App;
