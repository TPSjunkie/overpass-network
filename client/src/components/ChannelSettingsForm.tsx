import React, { useState, FormEvent } from 'react';

interface Channel {
  name: string;
  description: string;
}

interface ChannelSettingsFormProps {
  channel: Channel;
  onUpdateSettings: (settings: Channel) => void;
}

const ChannelSettingsForm: React.FC<ChannelSettingsFormProps> = ({ channel, onUpdateSettings }) => {
  const [name, setName] = useState<string>(channel.name);
  const [description, setDescription] = useState<string>(channel.description);

  const handleSubmit = (e: FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    onUpdateSettings({ name, description });
  };

  return (
    <form onSubmit={handleSubmit}>
      <div>
        <label htmlFor="channelName">Channel Name:</label>
        <input
          type="text"
          id="channelName"
          value={name}
          onChange={(e) => setName(e.target.value)}
          required
        />
      </div>
      <div>
        <label htmlFor="channelDescription">Channel Description:</label>
        <textarea
          id="channelDescription"
          value={description}
          onChange={(e) => setDescription(e.target.value)}
        />
      </div>
      <button type="submit">Update Settings</button>
    </form>
  );
};

export default ChannelSettingsForm;
