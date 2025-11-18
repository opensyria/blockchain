#[cfg(test)]
mod tests {
    use opensyria_core::{Block};
    use opensyria_consensus::ProofOfWork;
    use opensyria_storage::{BlockchainStorage, StateStorage};
    use std::path::PathBuf;

    fn setup_test_blockchain() -> PathBuf {
        let test_dir = std::env::temp_dir()
            .join(format!("explorer_test_{}", std::process::id()));
        
        let _ = std::fs::remove_dir_all(&test_dir);
        
        // Create blockchain with a few blocks
        let blocks_dir = test_dir.join("blocks");
        let state_dir = test_dir.join("state");
        
        let blockchain = BlockchainStorage::open(blocks_dir).unwrap();
        let _state = StateStorage::open(state_dir).unwrap();
        
        let pow = ProofOfWork::new(16);
        
        // Mine genesis
        let genesis = Block::genesis(16);
        let (mined_genesis, _) = pow.mine(genesis);
        blockchain.append_block(&mined_genesis).unwrap();
        
        // Mine a few more blocks
        for _ in 0..3 {
            let tip_hash = blockchain.get_chain_tip().unwrap().unwrap();
            let prev_block = blockchain.get_block(&tip_hash).unwrap().unwrap();
            let new_block = Block::new(prev_block.hash(), vec![], 16);
            let (mined_block, _) = pow.mine(new_block);
            blockchain.append_block(&mined_block).unwrap();
        }
        
        drop(blockchain);
        test_dir
    }

    #[tokio::test]
    async fn test_explorer_stats() {
        use crate::handlers::AppState;
        use crate::handlers::get_chain_stats;
        use axum::extract::State;
        use std::sync::Arc;
        use tokio::sync::RwLock;
        
        let test_dir = setup_test_blockchain();
        
        let blockchain = BlockchainStorage::open(test_dir.join("blocks")).unwrap();
        let state = StateStorage::open(test_dir.join("state")).unwrap();
        
        let app_state = AppState {
            blockchain: Arc::new(RwLock::new(blockchain)),
            state: Arc::new(RwLock::new(state)),
        };
        
        let result = get_chain_stats(State(app_state)).await;
        assert!(result.is_ok());
        
        let stats = result.unwrap().0;
        assert_eq!(stats.height, 4); // Genesis + 3 blocks
        assert_eq!(stats.total_blocks, 5); // height + 1
        assert_eq!(stats.difficulty, 16);
        
        std::fs::remove_dir_all(&test_dir).ok();
    }

    #[tokio::test]
    async fn test_get_block_by_height() {
        use crate::handlers::{AppState, get_block_by_height};
        use axum::extract::{Path, State};
        use std::sync::Arc;
        use tokio::sync::RwLock;
        
        let test_dir = setup_test_blockchain();
        
        let blockchain = BlockchainStorage::open(test_dir.join("blocks")).unwrap();
        let state = StateStorage::open(test_dir.join("state")).unwrap();
        
        let app_state = AppState {
            blockchain: Arc::new(RwLock::new(blockchain)),
            state: Arc::new(RwLock::new(state)),
        };
        
        // Test genesis block (height 1)
        let result = get_block_by_height(Path(1), State(app_state.clone())).await;
        assert!(result.is_ok());
        
        let block_info = result.unwrap().0;
        assert_eq!(block_info.height, 1);
        assert_eq!(block_info.difficulty, 16);
        assert_eq!(block_info.transaction_count, 0);
        
        // Test non-existent block
        let result = get_block_by_height(Path(100), State(app_state)).await;
        assert!(result.is_err());
        
        std::fs::remove_dir_all(&test_dir).ok();
    }

    #[tokio::test]
    async fn test_get_recent_blocks() {
        use crate::handlers::{AppState, get_recent_blocks, Pagination};
        use axum::extract::{Query, State};
        use std::sync::Arc;
        use tokio::sync::RwLock;
        
        let test_dir = setup_test_blockchain();
        
        let blockchain = BlockchainStorage::open(test_dir.join("blocks")).unwrap();
        let state = StateStorage::open(test_dir.join("state")).unwrap();
        
        let app_state = AppState {
            blockchain: Arc::new(RwLock::new(blockchain)),
            state: Arc::new(RwLock::new(state)),
        };
        
        let pagination = Pagination { page: 1, per_page: 10 };
        let result = get_recent_blocks(Query(pagination), State(app_state)).await;
        
        assert!(result.is_ok());
        
        let paginated = result.unwrap().0;
        assert_eq!(paginated.total, 5); // Total blocks
        assert_eq!(paginated.page, 1);
        assert!(paginated.items.len() <= 5);
        
        // Most recent block should be first
        if !paginated.items.is_empty() {
            assert_eq!(paginated.items[0].height, 4);
        }
        
        std::fs::remove_dir_all(&test_dir).ok();
    }
}
