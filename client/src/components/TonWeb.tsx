/*import React from "react";
import TonWeb from "tonweb";


const tonweb = new TonWeb();

// Example function to connect to a TON wallet
async function connectWallet() {
  try {
    const wallet = tonweb.wallet.create({});
    if (wallet.address) {
      console.log("Wallet Address:", wallet.address.toString(true, true, true));
    } else {
      console.error("Wallet address is undefined");
    }
  } catch (error) {
    console.error("Error connecting wallet:", error);
  }
}

function ConnectTONButton() {
  return <button onClick={connectWallet}>Connect TON Wallet</button>;
}

function App() {
  return (
    <div>
      <ConnectTONButton />
    </div>
  );
}

export { App, ConnectTONButton, connectWallet };
export default App;
//*/


