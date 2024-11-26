import { useMemo } from 'react';

const MemoizedComponent = ({ channels }) => {
  const totalBalance = useMemo(() => {
    return channels.reduce((total, channel) => total + channel.balance, 0n);
  }, [channels]);

  return (
    <div>
      <h2>Total Balance: {totalBalance}</h2>
      <ul>
        {channels.map((channel) => (
          <li key={channel.id}>
            <h3>{channel.name}</h3>
            <p>Balance: {channel.balance}</p>
          </li>
        ))}
      </ul>
    </div>
  );
};
export default MemoizedComponent;
