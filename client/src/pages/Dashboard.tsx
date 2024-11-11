import React, { useState, useEffect, useCallback } from 'react';
import { ChannelContract} from "../services/api";
import { Button } from '../components/ui/button';
import Balance from '../components/Balance';
import TransactionHistory from '../components/TransactionHistory';
import AnalyticsReport from '../components/AnalyticsReport';
import SendTransaction from '../components/SendTransaction';
import BocInteraction from '../components/BocInteraction';
import { toast } from 'react-toastify';
import { toNano } from '@ton/ton';
import { useTonWallet, useTonConnectUI, TonConnectUI } from '@tonconnect/ui-react';
import TokenList from '../components/TokenList';
import GlobalMarketData from '../components/GlobalMarketData';
import { fetchTransactions } from '@/utils/offChain';
import { Input } from '@/components/ui/input';
import { CopyButton } from '@/components/items/CopyButton';
import { CardContent } from '@/components/ui/card';
import { Card, CardHeader, CardTitle } from 'react-bootstrap';
import { Label } from 'recharts';
import { connector } from '@/connector/connection-ton';

const Dashboard: React.FC = () => {
  const [connector, setConnector] = useState<any>();
  const [wallets, setWallets] = useState<any>([]);
  const [isConnected, setIsConnected] = useState(false);
  const [userWallet, setUserWallet] = useState<any>();
  const [loading, setLoading] = useState(false);
  const [tonConnectUI] = useTonConnectUI();
  const [transactionDetails, setTransactionDetails] = useState({
    to: '',
    amount: '',
    payload: ''
  });
  const [transactionState, setTransactionState] = useState({
    status: '',
    txHash: '',
    error: '',
    to: '',
    amount: '',
    payload: '',
  });

  useEffect(() => {
    const initConnector = async () => {
      const newConnector = new TonConnectUI({
        manifestUrl: 'https://overpass.network/tonconnect-manifest.json',
      });

      const walletsList = await newConnector.getWallets();
      setConnector(newConnector);
      setWallets(walletsList);

      newConnector.onStatusChange(
        async (wallet) => {
          if (wallet) {
            setIsConnected(true);
            setUserWallet(wallet);
          } else {
            setIsConnected(false);
            setUserWallet(null);
          }
        },
        console.error,
        async () => {
          setIsConnected(false);
          setUserWallet(null);
        }
      );
    };
    initConnector();
  }, []);

  const sendTransaction = useCallback(async () => {
    if (!tonConnectUI || !transactionState.to || !transactionState.amount) {
      toast.error('Invalid transaction details or wallet not connected');
      return;
    }

    try {
      const result = await tonConnectUI.sendTransaction({
        validUntil: Math.floor(Date.now() / 1000) + 300,
        messages: [
          {
            address: transactionState.to,
            amount: toNano(transactionState.amount).toString(),
            payload: transactionState.payload || '',
          },
        ],
      });

 
      toast.success('Transaction sent successfully!');
    } catch (error) {
      console.error('Error sending transaction:', error);
      if (error instanceof Error && error.message === 'User rejected the transaction') {
        setTransactionState((prevState) => ({
          ...prevState,
          status: 'rejected',
          error: ''
        }));
        toast.error('Transaction rejected by user');
      } else {
        setTransactionState((prevState) => ({
          ...prevState,
          status: 'error',
          error: error instanceof Error ? error.message : String(error)
        }));
        toast.error('Failed to send transaction');
      }
    }
  }, [tonConnectUI, transactionState.to, transactionState.amount, transactionState.payload]);

  const connectWallet = useCallback(async () => {
    if (!connector) return;

    try {
      setLoading(true);
      const walletConnectionSource = {
        jsBridgeKey: 'tonkeeper',
      };
      await connector.connect(walletConnectionSource);
    } catch (e) {
      console.error(e);
      toast.error('Failed to connect wallet');
    } finally {
      setLoading(false);
    }
  }, [connector]);

  const disconnectWallet = useCallback(async () => {
    if (!connector) return;

    try {
      setLoading(true);
      await connector.disconnect();
    } catch (e) {
      console.error(e);
      toast.error('Failed to disconnect wallet');
    } finally {
      setLoading(false);
    }
  }, [connector]);

  function sendTonConnectTx(event: React.MouseEvent<HTMLButtonElement>) {
    sendTransaction();
  }

  return (
    <div className="container mx-auto p-4">
      <h1 className="text-3xl font-bold mb-4">Dashboard</h1>
      {isConnected ? (
        <>
          <Button onClick={disconnectWallet} disabled={loading}>
            {loading ? 'Disconnecting...' : 'Disconnect Wallet'}
          </Button>
          <Balance balance={userWallet?.account?.toString() || ''} />
          <SendTransaction onSendTransaction={async (to: string, amount: string, payload: string) => {
            setTransactionState(prev => ({
              ...prev,
              to,
              amount,
              payload
            }));
            await sendTransaction();
          }} />
          <TransactionHistory transactions={[]} fetchTransactions={async () => {
            return [];
          }} />
          <AnalyticsReport transactions={[]} />
          <BocInteraction />
          <TokenList tokens={[]} />
          <GlobalMarketData />
        </>
      ) : (
        <Button onClick={connectWallet} disabled={loading}>
          {loading ? 'Connecting...' : 'Connect Wallet'}
        </Button>
      )}
      <Card>
        <CardHeader>
          <CardTitle>Transaction Details</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            <div className="flex items-center gap-2">
              <div>TonConnect:</div>
              {userWallet ? (
                <Button onClick={sendTonConnectTx}>Send Transaction</Button>
              ) : (
                <div>Connect TonConnect wallet to send tx</div>
              )}
            </div>

            <div className="space-y-2">
              <Label>From:</Label>
              <div className="flex items-center gap-2">
                <Input id="from" value={userWallet?.account?.address || ''} readOnly />
                <CopyButton value={userWallet?.account?.address || ''} />
              </div>
            </div>
            <div className="space-y-2">
              <Label>To:</Label>
              <div className="flex items-center gap-2">
                <Input
                  id="to"
                  value={transactionDetails.to}
                  onChange={(e) => setTransactionDetails({
                    ...transactionDetails,
                    to: e.target.value,
                  })}
                />
              </div>
            </div>

            <div className="space-y-2">
              <Label>Amount (TON):</Label>
              <div className="flex items-center gap-2">
                <Input
                  id="amount"
                  type="number"
                  value={transactionDetails.amount}
                  onChange={(e) => setTransactionDetails({
                    ...transactionDetails,
                    amount: e.target.value,
                  })}
                />
              </div>
            </div>

            <div className="space-y-2">
              <Label>Payload:</Label>
              <div className="flex items-center gap-2">
                <Input
                  id="payload"
                  value={transactionDetails.payload}
                  onChange={(e) => setTransactionDetails({
                    ...transactionDetails,
                    payload: e.target.value,
                  })}
                />
                <CopyButton value={transactionDetails.payload} />
              </div>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
};

export { Dashboard };