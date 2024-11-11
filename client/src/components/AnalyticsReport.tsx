import React from 'react';
import { BarChart, Bar, XAxis, YAxis, Tooltip, Legend, ResponsiveContainer, PieChart, Pie, Cell, LineChart, Line } from 'recharts';
import { useTonConnect } from '../hooks/useTonConnect';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import type { Transaction } from '../types/wasm-types';
import { formatTON } from '../utils/formatters';

interface AnalyticsReportProps {
  transactions: Transaction[];
}

const AnalyticsReport: React.FC<AnalyticsReportProps> = ({ transactions }) => {
  const { walletInfo } = useTonConnect();
  const [transactionData, setTransactionData] = React.useState<any>({});
  const [loading, setLoading] = React.useState(true);
  const [error, setError] = React.useState<string | null>(null);

  React.useEffect(() => {
    const processTransactions = () => {
      if (transactions.length === 0) {
        setError('No transactions available.');
        setLoading(false);
        return;
      }

      try {
        setLoading(true);
        
        // Process transactions
        const volumeData: { date: string; volume: number }[] = [];
        const typeData: { type: string; count: number }[] = [];
        let totalVolume = 0;
        const typeCount: { [key: string]: number } = {};

        transactions.forEach(tx => {
          // Process volume data
          const date = new Date(Number(tx.timestamp) * 1000).toLocaleDateString('en-US', { month: 'short' });
          const existingVolumeEntry = volumeData.find(entry => entry.date === date);
          if (existingVolumeEntry) {
            existingVolumeEntry.volume += Number(tx.amount);
          } else {
            volumeData.push({ date, volume: Number(tx.amount) });
          }

          // Process type data
          const type = tx.amount.toString().startsWith('-') ? 'Send' : 'Receive';
          typeCount[type] = (typeCount[type] || 0) + 1;

          // Calculate total volume
          totalVolume += Math.abs(Number(tx.amount));
        });

        // Convert type count to array
        for (const [type, count] of Object.entries(typeCount)) {
          typeData.push({ type, count });
        }

        const processedData = {
          volumeData: volumeData.sort((a, b) => new Date(a.date).getTime() - new Date(b.date).getTime()),
          typeData,
          totalTransactions: transactions.length,
          totalVolume: formatTON(totalVolume),
          averageTransactionSize: formatTON(totalVolume / transactions.length),
        };

        setTransactionData(processedData);
        setLoading(false);
      } catch (err) {
        console.error('Error processing transaction data:', err);
        setError('Failed to process transaction data. Please try again later.');
        setLoading(false);
      }
    };

    processTransactions();
  }, [transactions]);

  if (loading) {
    return <p className="text-center py-4">Loading analytics...</p>;
  }

  if (error) {
    return <p className="text-red-500 text-center py-4">{error}</p>;
  }

  const COLORS = ['#0088FE', '#00C49F', '#FFBB28', '#FF8042'];

  return (
    <Card className="bg-pip-boy-panel shadow-pip-boy">
      <CardHeader>
        <CardTitle className="text-2xl font-bold text-pip-boy-green">Transaction Analytics</CardTitle>
      </CardHeader>
      <CardContent>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mb-6">
          <div className="bg-gray-800 p-4 rounded-lg">
            <h3 className="text-lg font-semibold mb-2 text-pip-boy-green">Total Transactions</h3>
            <p className="text-3xl text-pip-boy-text">{transactionData.totalTransactions}</p>
          </div>
          <div className="bg-gray-800 p-4 rounded-lg">
            <h3 className="text-lg font-semibold mb-2 text-pip-boy-green">Total Volume</h3>
            <p className="text-3xl text-pip-boy-text">{transactionData.totalVolume} TON</p>
          </div>
          <div className="bg-gray-800 p-4 rounded-lg">
            <h3 className="text-lg font-semibold mb-2 text-pip-boy-green">Average Transaction Size</h3>
            <p className="text-3xl text-pip-boy-text">{transactionData.averageTransactionSize} TON</p>
          </div>
        </div>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          <div>
            <h3 className="text-xl font-semibold mb-4 text-pip-boy-green">Monthly Transaction Volume</h3>
            <ResponsiveContainer width="100%" height={300}>
              <BarChart data={transactionData.volumeData}>
                <XAxis dataKey="date" stroke="#8884d8" />
                <YAxis stroke="#8884d8" />
                <Tooltip contentStyle={{ backgroundColor: '#1F2937', border: 'none' }} />
                <Legend />
                <Bar dataKey="volume" fill="#8884d8" />
              </BarChart>
            </ResponsiveContainer>
          </div>
          <div>
            <h3 className="text-xl font-semibold mb-4 text-pip-boy-green">Transaction Types</h3>
            <ResponsiveContainer width="100%" height={300}>
              <PieChart>
                <Pie
                  data={transactionData.typeData}
                  cx="50%"
                  cy="50%"
                  labelLine={false}
                  outerRadius={80}
                  fill="#8884d8"
                  dataKey="count"
                  label={({ name, percent }) => `${name} ${(percent * 100).toFixed(0)}%`}
                >
                  {transactionData.typeData.map((_: any, index: number) => (
                    <Cell key={`cell-${index}`} fill={COLORS[index % COLORS.length]} />
                  ))}
                </Pie>
                <Tooltip contentStyle={{ backgroundColor: '#1F2937', border: 'none' }} />
                <Legend />
              </PieChart>
            </ResponsiveContainer>
          </div>
        </div>
        <div className="mt-6">
          <h3 className="text-xl font-semibold mb-4 text-pip-boy-green">Transaction Volume Trend</h3>
          <ResponsiveContainer width="100%" height={300}>
            <LineChart data={transactionData.volumeData}>
              <XAxis dataKey="date" stroke="#8884d8" />
              <YAxis stroke="#8884d8" />
              <Tooltip contentStyle={{ backgroundColor: '#1F2937', border: 'none' }} />
              <Legend />
              <Line type="monotone" dataKey="volume" stroke="#8884d8" activeDot={{ r: 8 }} />
            </LineChart>
          </ResponsiveContainer>
        </div>
      </CardContent>
    </Card>
  );
};

export default AnalyticsReport;