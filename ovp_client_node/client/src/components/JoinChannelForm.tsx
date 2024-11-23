import React, { useState, type FormEvent } from 'react';

interface JoinChannelFormProps {
  onJoinChannel: (channelId: string) => void;
}

const JoinChannelForm: React.FC<JoinChannelFormProps> = ({ onJoinChannel }) => {
  const [channelId, setChannelId] = useState<string>('');

  const handleSubmit = (e: FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    onJoinChannel(channelId);
    setChannelId('');
  };

  return (
    <form onSubmit={handleSubmit}>
      <input
        type="text"
        value={channelId}
        onChange={(e) => setChannelId(e.target.value)}
        placeholder="Enter channel ID"
        required
      />
      <button type="submit">Join Channel</button>
    </form>
  );
};

export default JoinChannelForm;
