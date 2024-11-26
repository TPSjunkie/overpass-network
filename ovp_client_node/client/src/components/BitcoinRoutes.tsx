import { Route } from "react-router-dom";
import { BitcoinDashboard } from "../pages/BitcoinDashboard";
import { BitcoinBridge } from "./BitcoinBridge";
import { OverpassBitcoinControl } from "./OverpassBitcoinControl";

const BitcoinRoutes = () => {
  return (
    <>
      <Route path="/bitcoin" element={<BitcoinDashboard />} />
      <Route path="/bitcoin/bridge" element={<BitcoinBridge />} />
      <Route path="/overpass/bitcoin" element={<OverpassBitcoinControl />} />
    </>
  );
};

export default BitcoinRoutes;