import React from 'react';

interface Member {
  id: string;
  name: string;
}

interface ChannelMembersProps {
  members: Member[];
}

const ChannelMembers: React.FC<ChannelMembersProps> = ({ members }) => {
  return (
    <div>
      <h3>Channel Members</h3>
      <ul>
        {members.map((member) => (
          <li key={member.id}>{member.name}</li>
        ))}
      </ul>
    </div>
  );
};

export default ChannelMembers;
