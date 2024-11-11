import React, { useState } from 'react';

const CreateChannelForm = ({ onCreateChannel }) => {
  const [channelName, setChannelName] = useState('');

  const handleSubmit = (e) => {
    e.preventDefault();
    onCreateChannel(channelName);
    setChannelName('');
  };

  return (
    <form onSubmit={handleSubmit}>
      <input
        type="text"
        value={channelName}
        onChange={(e) => setChannelName(e.target.value)}
        placeholder="Enter channel name"
        required
      />
      <button type="submit">Create Channel</button>
    </form>
  );
};

export default CreateChannelForm;