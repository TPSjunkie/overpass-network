// ./hooks/useTokenContract.ts

import { TonClient, WalletContractV4, internal } from 'ton'
import { mnemonicNew, mnemonicToPrivateKey } from '@ton/crypto'

// Create Client
const tonClient = new TonClient({
  endpoint: 'https://toncenter.com/api/v2/jsonRPC',
})

// Function to generate new key and create wallet contract
async function createWalletContract() {
  // Generate new key
  let mnemonics = await mnemonicNew()
  let keyPair = await mnemonicToPrivateKey(mnemonics)

  // Create wallet contract
  let workchain = 0 // Usually you need a workchain 0
  let wallet = WalletContractV4.create({ workchain, publicKey: keyPair.publicKey })
  let contract = tonClient.open(wallet)
  
  return { contract, keyPair }
}

// Function to send transfer
async function sendTransfer(contract: { sendTransfer: (arg0: { seqno: any; secretKey: any; messages: any[] }) => any; getSeqno: () => any; address: any }, keyPair: { secretKey: any }) {
  await contract.sendTransfer({
    seqno: await contract.getSeqno(),
    secretKey: keyPair.secretKey,
    messages: [internal({
      to: contract.address,
      value: '0',
      bounce: false,
      body: 'Debug message'
    })]
  })
}

// Usage example (uncomment to use)
// (async () => {
//   const { contract, keyPair } = await createWalletContract()
//   await sendTransfer(contract, keyPair)
// })()