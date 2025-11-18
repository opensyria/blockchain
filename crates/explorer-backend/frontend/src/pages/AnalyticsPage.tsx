import { useState } from 'react';
import { Layout } from '@/components/Layout';
import {
  LineChart,
  Line,
  AreaChart,
  Area,
  BarChart,
  Bar,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ResponsiveContainer,
} from 'recharts';
import './AnalyticsPage.css';

// Mock data - would come from API
const blockTimeData = Array.from({ length: 24 }, (_, i) => ({
  hour: `${i}:00`,
  blockTime: 55 + Math.random() * 10,
  avgBlockTime: 60,
}));

const hashRateData = Array.from({ length: 7 }, (_, i) => ({
  day: ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'][i],
  hashRate: 1.5 + Math.random() * 0.3,
}));

const transactionVolumeData = Array.from({ length: 12 }, (_, i) => ({
  month: ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'][i],
  transactions: Math.floor(5000 + Math.random() * 3000),
  value: Math.floor(100000 + Math.random() * 50000),
}));

const difficultyData = Array.from({ length: 30 }, (_, i) => ({
  block: i * 100,
  difficulty: 16 + Math.floor(Math.random() * 8),
}));

export function AnalyticsPage() {
  const [timeRange, setTimeRange] = useState<'24h' | '7d' | '30d' | 'all'>('24h');

  return (
    <Layout>
      <div className="container">
        <div className="analytics-page animate-fade-in-up">
          <div className="page-header">
            <h1 className="page-title">Blockchain Analytics</h1>
            <div className="time-range-selector">
              <button
                className={`range-btn ${timeRange === '24h' ? 'active' : ''}`}
                onClick={() => setTimeRange('24h')}
              >
                24H
              </button>
              <button
                className={`range-btn ${timeRange === '7d' ? 'active' : ''}`}
                onClick={() => setTimeRange('7d')}
              >
                7D
              </button>
              <button
                className={`range-btn ${timeRange === '30d' ? 'active' : ''}`}
                onClick={() => setTimeRange('30d')}
              >
                30D
              </button>
              <button
                className={`range-btn ${timeRange === 'all' ? 'active' : ''}`}
                onClick={() => setTimeRange('all')}
              >
                All
              </button>
            </div>
          </div>

          <div className="charts-grid stagger-children">
            {/* Block Time Trend */}
            <div className="card-cultural chart-card hover-lift">
              <h2 className="chart-title">Block Time Trend</h2>
              <p className="chart-subtitle">Average time between blocks (seconds)</p>
              <ResponsiveContainer width="100%" height={300}>
                <LineChart data={blockTimeData}>
                  <CartesianGrid strokeDasharray="3 3" stroke="#e5e7eb" />
                  <XAxis dataKey="hour" stroke="#6b7280" />
                  <YAxis stroke="#6b7280" />
                  <Tooltip
                    contentStyle={{
                      backgroundColor: '#fff',
                      border: '1px solid #e5e7eb',
                      borderRadius: '8px',
                    }}
                  />
                  <Legend />
                  <Line
                    type="monotone"
                    dataKey="blockTime"
                    stroke="#667eea"
                    strokeWidth={2}
                    dot={{ fill: '#667eea', r: 4 }}
                    name="Block Time"
                  />
                  <Line
                    type="monotone"
                    dataKey="avgBlockTime"
                    stroke="#f4a261"
                    strokeWidth={2}
                    strokeDasharray="5 5"
                    dot={false}
                    name="Target (60s)"
                  />
                </LineChart>
              </ResponsiveContainer>
            </div>

            {/* Hash Rate */}
            <div className="card-cultural chart-card hover-lift">
              <h2 className="chart-title">Network Hash Rate</h2>
              <p className="chart-subtitle">Mining power over time (MH/s)</p>
              <ResponsiveContainer width="100%" height={300}>
                <AreaChart data={hashRateData}>
                  <defs>
                    <linearGradient id="hashRateGradient" x1="0" y1="0" x2="0" y2="1">
                      <stop offset="5%" stopColor="#6a994e" stopOpacity={0.8} />
                      <stop offset="95%" stopColor="#6a994e" stopOpacity={0.1} />
                    </linearGradient>
                  </defs>
                  <CartesianGrid strokeDasharray="3 3" stroke="#e5e7eb" />
                  <XAxis dataKey="day" stroke="#6b7280" />
                  <YAxis stroke="#6b7280" />
                  <Tooltip
                    contentStyle={{
                      backgroundColor: '#fff',
                      border: '1px solid #e5e7eb',
                      borderRadius: '8px',
                    }}
                  />
                  <Area
                    type="monotone"
                    dataKey="hashRate"
                    stroke="#6a994e"
                    strokeWidth={2}
                    fillOpacity={1}
                    fill="url(#hashRateGradient)"
                    name="Hash Rate"
                  />
                </AreaChart>
              </ResponsiveContainer>
            </div>

            {/* Transaction Volume */}
            <div className="card-cultural chart-card hover-lift chart-card-wide">
              <h2 className="chart-title">Transaction Volume</h2>
              <p className="chart-subtitle">Monthly transaction count and value</p>
              <ResponsiveContainer width="100%" height={300}>
                <BarChart data={transactionVolumeData}>
                  <CartesianGrid strokeDasharray="3 3" stroke="#e5e7eb" />
                  <XAxis dataKey="month" stroke="#6b7280" />
                  <YAxis stroke="#6b7280" />
                  <Tooltip
                    contentStyle={{
                      backgroundColor: '#fff',
                      border: '1px solid #e5e7eb',
                      borderRadius: '8px',
                    }}
                  />
                  <Legend />
                  <Bar dataKey="transactions" fill="#e63946" name="Transactions" />
                  <Bar dataKey="value" fill="#457b9d" name="Value (SYL)" />
                </BarChart>
              </ResponsiveContainer>
            </div>

            {/* Difficulty Adjustment */}
            <div className="card-cultural chart-card hover-lift">
              <h2 className="chart-title">Difficulty Adjustment</h2>
              <p className="chart-subtitle">Mining difficulty over blocks</p>
              <ResponsiveContainer width="100%" height={300}>
                <LineChart data={difficultyData}>
                  <CartesianGrid strokeDasharray="3 3" stroke="#e5e7eb" />
                  <XAxis dataKey="block" stroke="#6b7280" />
                  <YAxis stroke="#6b7280" />
                  <Tooltip
                    contentStyle={{
                      backgroundColor: '#fff',
                      border: '1px solid #e5e7eb',
                      borderRadius: '8px',
                    }}
                  />
                  <Line
                    type="stepAfter"
                    dataKey="difficulty"
                    stroke="#764ba2"
                    strokeWidth={2}
                    dot={{ fill: '#764ba2', r: 3 }}
                    name="Difficulty"
                  />
                </LineChart>
              </ResponsiveContainer>
            </div>
          </div>

          <div className="divider-calligraphic">
            <span>✦</span>
          </div>

          <div className="stats-summary card-cultural">
            <h2 className="section-title">Network Statistics</h2>
            <div className="stats-row">
              <div className="stat-item">
                <div className="stat-label">Total Blocks</div>
                <div className="stat-value">12,345</div>
              </div>
              <div className="stat-item">
                <div className="stat-label">Total Transactions</div>
                <div className="stat-value">98,765</div>
              </div>
              <div className="stat-item">
                <div className="stat-label">Active Addresses</div>
                <div className="stat-value">4,567</div>
              </div>
              <div className="stat-item">
                <div className="stat-label">Avg Block Time</div>
                <div className="stat-value">58.3s</div>
              </div>
              <div className="stat-item">
                <div className="stat-label">Network Hash Rate</div>
                <div className="stat-value">1.62 MH/s</div>
              </div>
              <div className="stat-item">
                <div className="stat-label">Current Difficulty</div>
                <div className="stat-value">18</div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </Layout>
  );
}
