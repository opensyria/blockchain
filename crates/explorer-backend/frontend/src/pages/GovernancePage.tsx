import { useState } from 'react';
import { Layout } from '@/components/Layout';
import { formatDistanceToNow } from 'date-fns';
import './GovernancePage.css';

interface Proposal {
  id: number;
  title: string;
  description: string;
  proposer: string;
  type: 'text' | 'min-fee' | 'block-size' | 'reward' | 'param' | 'upgrade' | 'emergency';
  status: 'active' | 'passed' | 'rejected' | 'executed' | 'expired';
  votesFor: number;
  votesAgainst: number;
  votesAbstain: number;
  totalStake: number;
  startTime: Date;
  endTime: Date;
  executionTime?: Date;
}

// Mock data - would come from API
const proposals: Proposal[] = [
  {
    id: 1,
    title: 'Reduce Minimum Transaction Fee',
    description: 'Proposal to reduce the minimum transaction fee from 0.001 SYL to 0.0005 SYL to encourage more transactions and improve network accessibility.',
    proposer: 'syria1qz3r...4k8p',
    type: 'min-fee',
    status: 'active',
    votesFor: 125000,
    votesAgainst: 45000,
    votesAbstain: 12000,
    totalStake: 200000,
    startTime: new Date(Date.now() - 2 * 24 * 60 * 60 * 1000),
    endTime: new Date(Date.now() + 5 * 24 * 60 * 60 * 1000),
  },
  {
    id: 2,
    title: 'Increase Block Size Limit',
    description: 'Increase maximum block size from 1MB to 2MB to accommodate growing transaction volume while maintaining decentralization.',
    proposer: 'syria1ak7w...9m2v',
    type: 'block-size',
    status: 'active',
    votesFor: 89000,
    votesAgainst: 78000,
    votesAbstain: 8000,
    totalStake: 200000,
    startTime: new Date(Date.now() - 1 * 24 * 60 * 60 * 1000),
    endTime: new Date(Date.now() + 6 * 24 * 60 * 60 * 1000),
  },
  {
    id: 3,
    title: 'Damascus Heritage NFT Initiative',
    description: 'Allocate 10,000 SYL from treasury to fund digitization of Damascus historical sites as heritage NFTs for cultural preservation.',
    proposer: 'syria1mp4x...7t3n',
    type: 'text',
    status: 'passed',
    votesFor: 156000,
    votesAgainst: 21000,
    votesAbstain: 5000,
    totalStake: 200000,
    startTime: new Date(Date.now() - 10 * 24 * 60 * 60 * 1000),
    endTime: new Date(Date.now() - 3 * 24 * 60 * 60 * 1000),
    executionTime: new Date(Date.now() - 2 * 24 * 60 * 60 * 1000),
  },
  {
    id: 4,
    title: 'Network Upgrade to v2.0',
    description: 'Major protocol upgrade introducing smart contract support and improved consensus mechanism. Requires coordinated network-wide upgrade.',
    proposer: 'syria1ct9b...5w1k',
    type: 'upgrade',
    status: 'active',
    votesFor: 142000,
    votesAgainst: 34000,
    votesAbstain: 18000,
    totalStake: 200000,
    startTime: new Date(Date.now() - 4 * 24 * 60 * 60 * 1000),
    endTime: new Date(Date.now() + 10 * 24 * 60 * 60 * 1000),
  },
  {
    id: 5,
    title: 'Adjust Mining Rewards',
    description: 'Reduce block reward from 50 SYL to 40 SYL to slow inflation rate while maintaining miner incentives.',
    proposer: 'syria1vr2s...6h4j',
    type: 'reward',
    status: 'rejected',
    votesFor: 67000,
    votesAgainst: 112000,
    votesAbstain: 9000,
    totalStake: 200000,
    startTime: new Date(Date.now() - 15 * 24 * 60 * 60 * 1000),
    endTime: new Date(Date.now() - 8 * 24 * 60 * 60 * 1000),
  },
];

const statusColors = {
  active: '#667eea',
  passed: '#6a994e',
  rejected: '#e63946',
  executed: '#457b9d',
  expired: '#6b7280',
};

const typeIcons = {
  text: '📜',
  'min-fee': '💰',
  'block-size': '📦',
  reward: '🎁',
  param: '⚙️',
  upgrade: '🚀',
  emergency: '🚨',
};

export function GovernancePage() {
  const [selectedStatus, setSelectedStatus] = useState<string>('all');
  const [selectedProposal, setSelectedProposal] = useState<Proposal | null>(null);
  const [voteChoice, setVoteChoice] = useState<'for' | 'against' | 'abstain'>('for');

  const filteredProposals = proposals.filter(proposal => {
    if (selectedStatus === 'all') return true;
    return proposal.status === selectedStatus;
  });

  const getQuorum = (proposal: Proposal) => {
    const totalVotes = proposal.votesFor + proposal.votesAgainst + proposal.votesAbstain;
    const participation = (totalVotes / proposal.totalStake) * 100;
    return participation.toFixed(1);
  };

  const getApprovalRate = (proposal: Proposal) => {
    const totalVotes = proposal.votesFor + proposal.votesAgainst + proposal.votesAbstain;
    if (totalVotes === 0) return 0;
    return ((proposal.votesFor / totalVotes) * 100).toFixed(1);
  };

  return (
    <Layout>
      <div className="container">
        <div className="governance-page animate-fade-in-up">
          <div className="page-header corner-ornament">
            <div>
              <h1 className="page-title">Governance Proposals</h1>
              <p className="page-subtitle">Community-driven decision making</p>
            </div>
            <button className="btn-primary">Create Proposal</button>
          </div>

          <div className="status-filter card-cultural">
            <button
              className={`status-btn ${selectedStatus === 'all' ? 'active' : ''}`}
              onClick={() => setSelectedStatus('all')}
            >
              All
            </button>
            <button
              className={`status-btn ${selectedStatus === 'active' ? 'active' : ''}`}
              onClick={() => setSelectedStatus('active')}
            >
              🟢 Active
            </button>
            <button
              className={`status-btn ${selectedStatus === 'passed' ? 'active' : ''}`}
              onClick={() => setSelectedStatus('passed')}
            >
              ✅ Passed
            </button>
            <button
              className={`status-btn ${selectedStatus === 'rejected' ? 'active' : ''}`}
              onClick={() => setSelectedStatus('rejected')}
            >
              ❌ Rejected
            </button>
            <button
              className={`status-btn ${selectedStatus === 'executed' ? 'active' : ''}`}
              onClick={() => setSelectedStatus('executed')}
            >
              🎯 Executed
            </button>
          </div>

          <div className="proposals-list stagger-children">
            {filteredProposals.map((proposal) => (
              <div
                key={proposal.id}
                className="proposal-card card-cultural hover-lift"
                onClick={() => setSelectedProposal(proposal)}
              >
                <div className="proposal-header">
                  <div className="proposal-number">#{proposal.id}</div>
                  <div
                    className="proposal-status"
                    style={{ backgroundColor: statusColors[proposal.status] }}
                  >
                    {proposal.status}
                  </div>
                  <div className="proposal-type">
                    <span>{typeIcons[proposal.type]}</span>
                    <span>{proposal.type}</span>
                  </div>
                </div>

                <h3 className="proposal-title">{proposal.title}</h3>
                <p className="proposal-description">{proposal.description}</p>

                <div className="proposal-meta">
                  <div className="meta-item">
                    <span className="meta-label">Proposer:</span>
                    <span className="meta-value monospace">{proposal.proposer}</span>
                  </div>
                  <div className="meta-item">
                    <span className="meta-label">Started:</span>
                    <span className="meta-value">{formatDistanceToNow(proposal.startTime)} ago</span>
                  </div>
                  <div className="meta-item">
                    <span className="meta-label">Ends:</span>
                    <span className="meta-value">
                      {proposal.status === 'active'
                        ? `in ${formatDistanceToNow(proposal.endTime)}`
                        : formatDistanceToNow(proposal.endTime) + ' ago'}
                    </span>
                  </div>
                </div>

                <div className="vote-progress">
                  <div className="progress-bar">
                    <div
                      className="progress-fill progress-for"
                      style={{ width: `${(proposal.votesFor / proposal.totalStake) * 100}%` }}
                    />
                    <div
                      className="progress-fill progress-against"
                      style={{
                        width: `${(proposal.votesAgainst / proposal.totalStake) * 100}%`,
                        marginLeft: `${(proposal.votesFor / proposal.totalStake) * 100}%`,
                      }}
                    />
                  </div>
                  <div className="vote-stats">
                    <div className="vote-stat">
                      <span className="vote-label">For:</span>
                      <span className="vote-value">{proposal.votesFor.toLocaleString()} SYL</span>
                    </div>
                    <div className="vote-stat">
                      <span className="vote-label">Against:</span>
                      <span className="vote-value">{proposal.votesAgainst.toLocaleString()} SYL</span>
                    </div>
                    <div className="vote-stat">
                      <span className="vote-label">Quorum:</span>
                      <span className="vote-value">{getQuorum(proposal)}%</span>
                    </div>
                  </div>
                </div>
              </div>
            ))}
          </div>

          {filteredProposals.length === 0 && (
            <div className="empty-state card-cultural">
              <p>No proposals found with status: {selectedStatus}</p>
            </div>
          )}

          {selectedProposal && (
            <div className="modal-overlay" onClick={() => setSelectedProposal(null)}>
              <div className="modal-content card-cultural animate-scale-in" onClick={(e) => e.stopPropagation()}>
                <button className="modal-close" onClick={() => setSelectedProposal(null)}>
                  ✕
                </button>

                <div className="modal-header">
                  <div className="modal-badges">
                    <span className="badge-heritage">#{selectedProposal.id}</span>
                    <span
                      className="badge-heritage"
                      style={{ backgroundColor: statusColors[selectedProposal.status] }}
                    >
                      {selectedProposal.status}
                    </span>
                    <span className="badge-heritage">
                      {typeIcons[selectedProposal.type]} {selectedProposal.type}
                    </span>
                  </div>
                  <h2 className="modal-title">{selectedProposal.title}</h2>
                  <p className="modal-description">{selectedProposal.description}</p>
                </div>

                <div className="modal-stats">
                  <div className="stat-box">
                    <div className="stat-label">Approval Rate</div>
                    <div className="stat-value">{getApprovalRate(selectedProposal)}%</div>
                  </div>
                  <div className="stat-box">
                    <div className="stat-label">Participation</div>
                    <div className="stat-value">{getQuorum(selectedProposal)}%</div>
                  </div>
                  <div className="stat-box">
                    <div className="stat-label">Total Votes</div>
                    <div className="stat-value">
                      {(selectedProposal.votesFor + selectedProposal.votesAgainst + selectedProposal.votesAbstain).toLocaleString()}
                    </div>
                  </div>
                </div>

                <div className="vote-breakdown">
                  <h3>Vote Breakdown</h3>
                  <div className="breakdown-item">
                    <div className="breakdown-label">
                      <span className="dot dot-for"></span>
                      For
                    </div>
                    <div className="breakdown-value">{selectedProposal.votesFor.toLocaleString()} SYL</div>
                    <div className="breakdown-percent">
                      {((selectedProposal.votesFor / selectedProposal.totalStake) * 100).toFixed(1)}%
                    </div>
                  </div>
                  <div className="breakdown-item">
                    <div className="breakdown-label">
                      <span className="dot dot-against"></span>
                      Against
                    </div>
                    <div className="breakdown-value">{selectedProposal.votesAgainst.toLocaleString()} SYL</div>
                    <div className="breakdown-percent">
                      {((selectedProposal.votesAgainst / selectedProposal.totalStake) * 100).toFixed(1)}%
                    </div>
                  </div>
                  <div className="breakdown-item">
                    <div className="breakdown-label">
                      <span className="dot dot-abstain"></span>
                      Abstain
                    </div>
                    <div className="breakdown-value">{selectedProposal.votesAbstain.toLocaleString()} SYL</div>
                    <div className="breakdown-percent">
                      {((selectedProposal.votesAbstain / selectedProposal.totalStake) * 100).toFixed(1)}%
                    </div>
                  </div>
                </div>

                {selectedProposal.status === 'active' && (
                  <div className="voting-section">
                    <h3>Cast Your Vote</h3>
                    <div className="vote-options">
                      <label className={`vote-option ${voteChoice === 'for' ? 'selected' : ''}`}>
                        <input
                          type="radio"
                          value="for"
                          checked={voteChoice === 'for'}
                          onChange={(e) => setVoteChoice(e.target.value as 'for')}
                        />
                        <span>👍 Vote For</span>
                      </label>
                      <label className={`vote-option ${voteChoice === 'against' ? 'selected' : ''}`}>
                        <input
                          type="radio"
                          value="against"
                          checked={voteChoice === 'against'}
                          onChange={(e) => setVoteChoice(e.target.value as 'against')}
                        />
                        <span>👎 Vote Against</span>
                      </label>
                      <label className={`vote-option ${voteChoice === 'abstain' ? 'selected' : ''}`}>
                        <input
                          type="radio"
                          value="abstain"
                          checked={voteChoice === 'abstain'}
                          onChange={(e) => setVoteChoice(e.target.value as 'abstain')}
                        />
                        <span>🤷 Abstain</span>
                      </label>
                    </div>
                    <button className="btn-primary btn-vote">Submit Vote</button>
                    <p className="vote-note">Voting power is based on your staked SYL balance</p>
                  </div>
                )}

                {selectedProposal.executionTime && (
                  <div className="execution-info">
                    <span className="execution-badge">✅ Executed</span>
                    <span>Executed {formatDistanceToNow(selectedProposal.executionTime)} ago</span>
                  </div>
                )}
              </div>
            </div>
          )}
        </div>
      </div>
    </Layout>
  );
}
