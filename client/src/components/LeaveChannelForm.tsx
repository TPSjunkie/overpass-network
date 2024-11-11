import React from 'react';

interface LeaveChannelFormProps {
  onLeaveChannel: () => void;
  channelName: string;
}

const LeaveChannelForm: React.FC<LeaveChannelFormProps> = ({ onLeaveChannel, channelName }) => {
  const handleLeave = () => {
    onLeaveChannel();
  };

  return (
    <div>
      <p>Are you sure you want to leave the channel "{channelName}"?</p>
      <button onClick={handleLeave}>Leave Channel</button>
    </div>
  );
};

export default LeaveChannelForm;
