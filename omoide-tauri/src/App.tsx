import "./App.css";
import { useVault, VaultContextProvider } from "./context/vaultContext";
import { VaultLogin } from "./components/vaultLogin";

const AppContent = () => {
  const {state} = useVault();

  switch(state) {
    case "Locked":
      return <VaultLogin />;
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
