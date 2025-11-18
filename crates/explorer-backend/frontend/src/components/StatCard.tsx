import { ReactNode } from 'react';
import './StatCard.css';

interface StatCardProps {
  label: string;
  value: string | number;
  icon?: ReactNode;
  loading?: boolean;
}

export function StatCard({ label, value, icon, loading }: StatCardProps) {
  return (
    <div className="stat-card">
      {icon && <div className="stat-icon">{icon}</div>}
      <div className="stat-content">
        <div className="stat-label">{label}</div>
        <div className={`stat-value ${loading ? 'loading' : ''}`}>
          {loading ? '...' : value}
        </div>
      </div>
    </div>
  );
}
