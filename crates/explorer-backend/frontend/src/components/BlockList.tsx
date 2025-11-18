import { Link } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { BlockSummary } from '@/types/api';
import { formatDistanceToNow } from 'date-fns';
import './BlockList.css';

interface BlockListProps {
  blocks: BlockSummary[];
  loading?: boolean;
}

export function BlockList({ blocks, loading }: BlockListProps) {
  const { t } = useTranslation();

  if (loading) {
    return (
      <div className="block-list-loading">
        <div className="loading-spinner"></div>
        <p>{t('common.loading')}</p>
      </div>
    );
  }

  return (
    <div className="block-list">
      {blocks.map((block) => (
        <Link
          key={block.height}
          to={`/block/${block.height}`}
          className="block-item"
        >
          <div className="block-header">
            <div className="block-height">
              #{block.height.toLocaleString()}
            </div>
            <div className="block-time">
              {formatDistanceToNow(new Date(block.timestamp * 1000), {
                addSuffix: true,
              })}
            </div>
          </div>

          <div className="block-hash monospace">
            {block.hash}
          </div>

          <div className="block-info">
            <span>
              {block.transactions_count} {t('block.transactions')}
            </span>
            <span className="block-miner monospace">
              {t('block.miner')}: {block.miner.slice(0, 16)}...
            </span>
          </div>
        </Link>
      ))}
    </div>
  );
}
