//! Функциональная реализация блокчейна на Rust.
//!
//! Этот модуль предоставляет базовые компоненты блокчейна:
//! - структуру транзакции (`Transaction`),
//! - структуру блока (`Block`),
//! - цепочку блоков (`Blockchain`),
//! - механизм консенсуса на основе фиксированного списка пиров,
//! - сериализацию через `bincode`.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

/// Максимальное количество транзакций в одном блоке.
pub const MAX_TRANSACTIONS_PER_BLOCK: usize = 10;

/// Структура транзакции.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Transaction {
    /// Отправитель (публичный ключ, 32 байта).
    pub from: [u8; 32],
    /// Получатель (публичный ключ, 32 байта).
    pub to: [u8; 32],
    /// Сумма в минимальных единицах.
    pub amount: u64,
}

/// Структура блока.
///
/// Каждый блок содержит:
/// - `index` — порядковый номер,
/// - `timestamp` — время создания в секундах с Unix-эпохи,
/// - `transactions` — список транзакций,
/// - `previous_hash` — хеш предыдущего блока (32 байта),
/// - `hash` — хеш текущего блока (32 байта, SHA-256).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub index: u64,
    pub timestamp: u64,
    pub transactions: Vec<Transaction>,
    pub previous_hash: [u8; 32],
    pub hash: [u8; 32],
}

/// Вспомогательная структура для хеширования — содержит всё, кроме `hash`.
#[derive(Serialize)]
struct BlockContent<'a> {
    index: u64,
    timestamp: u64,
    transactions: &'a [Transaction],
    previous_hash: [u8; 32],
}

impl Block {
    /// Функция вычесления хеша блока на основе его содержимого (исключая поле `hash`).
    pub fn calculate_hash(&self) -> [u8; 32] {
        let content = BlockContent {
            index: self.index,
            timestamp: self.timestamp,
            transactions: &self.transactions,
            previous_hash: self.previous_hash,
        };
        let bytes =
            bincode::serialize(&content).expect("Не удалось сериализовать содержимое блока");
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        hasher.finalize().into()
    }
}

/// Функция возвращает текущее время в секундах с Unix-эпохи.
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Системное время установлено до Unix-эпохи")
        .as_nanos() as u64
}

/// Функция создания нового блока на основе предыдущего.
fn create_block(transactions: Vec<Transaction>, previous_block: &Block) -> Block {
    let index = previous_block.index + 1;
    let timestamp = current_timestamp();

    // Проверка: новый timestamp должен быть строго больше предыдущего
    if timestamp <= previous_block.timestamp {
        panic!(
            "Некорректный timestamp: {} <= {} (предыдущий блок)",
            timestamp, previous_block.timestamp
        );
    }

    let previous_hash = previous_block.hash;
    let mut block = Block {
        index,
        timestamp,
        transactions,
        previous_hash,
        hash: [0u8; 32],
    };
    block.hash = block.calculate_hash();
    block
}

/// Функция создания генезиз-блока.
///
/// Генезис-блок определяется как блок с `index == 0` и `previous_hash == [0u8; 32]` и не содержит транзакций.
fn create_genesis_block() -> Block {
    let mut block = Block {
        index: 0,
        timestamp: current_timestamp(),
        transactions: vec![],
        previous_hash: [0u8; 32],
        hash: [0u8; 32],
    };
    block.hash = block.calculate_hash();
    block
}

/// Структура блокчейна.
#[derive(Debug, Serialize, Deserialize)]
pub struct Blockchain {
    pub blocks: Vec<Block>,
}

impl Default for Blockchain {
    fn default() -> Self {
        Self::new()
    }
}

impl Blockchain {
    /// Создание новой цепочки с добавлением генезис-блока.
    pub fn new() -> Self {
        let mut chain = Blockchain { blocks: vec![] };
        chain.blocks.push(create_genesis_block());
        chain
    }

    /// Добавляет новый блок с заданными транзакциями.
    pub fn add_block(&mut self, transactions: Vec<Transaction>) {
        if transactions.len() > MAX_TRANSACTIONS_PER_BLOCK {
            panic!(
                "Превышено максимальное число транзакций в блоке: {} > {}",
                transactions.len(),
                MAX_TRANSACTIONS_PER_BLOCK
            );
        }
        let last_block = self.blocks.last().unwrap();
        let new_block = create_block(transactions, last_block);
        self.blocks.push(new_block);
    }

    /// Метод вывода информации о блоках.
    pub fn print_chain(&self) {
        for block in &self.blocks {
            println!("--- Block {} ---", block.index);
            println!("Timestamp: {}", block.timestamp);
            println!("Hash: {}", hex::encode(block.hash));
            println!("Transactions:");
            if block.transactions.is_empty() {
                println!("  (нет транзакций)");
            } else {
                for tx in &block.transactions {
                    println!(
                        "  {} → {} : {}",
                        hex::encode(tx.from),
                        hex::encode(tx.to),
                        tx.amount
                    );
                }
            }
            println!("Prev: {}", hex::encode(block.previous_hash));
            println!();
        }
    }

    /// Метод вывода информации о блоке по номеру.
    pub fn get_block(&self, index: usize) -> Option<&Block> {
        self.blocks.get(index)
    }

    /// Метод вывода общей информации о блокчейне.
    pub fn get_chain_info(&self) -> String {
        format!(
            "Блоков: {}, Валидно: {}, Последний хеш: {}",
            self.blocks.len(),
            self.is_valid(),
            &hex::encode(self.blocks.last().unwrap().hash)[..10]
        )
    }

    /// Проверка целостности всей цепочки.
    pub fn is_valid(&self) -> bool {
        if self.blocks.is_empty() {
            return false;
        }
        // Проверка генезис-блока
        let genesis = &self.blocks[0];
        if genesis.index != 0 {
            return false;
        }
        if genesis.previous_hash != [0u8; 32] {
            return false;
        }
        if genesis.hash != genesis.calculate_hash() {
            return false;
        }
        // Проверка остальных блоков
        for i in 1..self.blocks.len() {
            let current = &self.blocks[i];
            let previous = &self.blocks[i - 1];
            if current.index != previous.index + 1 {
                return false;
            }
            if current.previous_hash != previous.hash {
                return false;
            }
            if current.hash != current.calculate_hash() {
                return false;
            }
        }
        true
    }
}

/// Модель участников сети (пиров) и консенсуса.
///
/// Идентификатор пира.
pub type PeerId = u32;

/// Моделирование пира.
#[derive(Debug, Clone)]
pub struct Peer {
    pub id: PeerId,
    pub is_honest: bool,
}

impl Peer {
    pub fn new(id: PeerId) -> Self {
        Self {
            id,
            is_honest: true,
        }
    }

    pub fn vote_for_transaction(&self, _transactions: &[Transaction]) -> bool {
        true
    }
}

/// Консенсус с фиксированным списком пиров.
pub struct FixedPeerConsensus {
    pub peers: Vec<Peer>,
}

impl FixedPeerConsensus {
    pub fn new(peers: Vec<Peer>) -> Self {
        Self { peers }
    }

    pub fn peer_count(&self) -> usize {
        self.peers.len()
    }

    fn majority_threshold(&self) -> usize {
        self.peers.len().div_ceil(2)
    }

    /// Предлагает добавить блок с транзакциями.
    pub fn propose_block(
        &self,
        transactions: Vec<Transaction>,
        blockchain: &mut Blockchain,
    ) -> bool {
        if self.peers.is_empty() {
            return false;
        }
        let approvals = self
            .peers
            .iter()
            .filter(|peer| peer.vote_for_transaction(&transactions))
            .count();
        let threshold = self.majority_threshold();
        if approvals > threshold {
            blockchain.add_block(transactions);
            true
        } else {
            false
        }
    }
}

/// Сериализация
pub fn serialize_block(block: &Block) -> Result<Vec<u8>, bincode::Error> {
    bincode::serialize(block)
}

pub fn deserialize_block(bytes: &[u8]) -> Result<Block, bincode::Error> {
    bincode::deserialize(bytes)
}

pub fn serialize_blockchain(chain: &Blockchain) -> Result<Vec<u8>, bincode::Error> {
    bincode::serialize(chain)
}

pub fn deserialize_blockchain(bytes: &[u8]) -> Result<Blockchain, bincode::Error> {
    bincode::deserialize(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_tx(from: [u8; 32], to: [u8; 32], amount: u64) -> Transaction {
        Transaction { from, to, amount }
    }

    #[test]
    fn test_genesis_block_has_correct_properties() {
        let chain = Blockchain::new();
        let genesis = &chain.blocks[0];
        assert_eq!(genesis.index, 0);
        assert_eq!(genesis.previous_hash, [0u8; 32]);
        assert!(genesis.transactions.is_empty());
        assert_eq!(genesis.hash, genesis.calculate_hash());
    }

    #[test]
    fn test_chain_validity_with_real_transactions() {
        let mut chain = Blockchain::new();
        chain.add_block(vec![dummy_tx([1; 32], [2; 32], 100)]);
        chain.add_block(vec![dummy_tx([3; 32], [4; 32], 50)]);
        assert!(chain.is_valid());
    }

    #[test]
    fn test_chain_becomes_invalid_after_tampering() {
        let mut chain = Blockchain::new();
        chain.add_block(vec![dummy_tx([1; 32], [2; 32], 1)]);
        chain.blocks[1].transactions.clear();
        assert!(!chain.is_valid());
    }

    #[test]
    fn test_block_serialization_roundtrip() {
        let mut block = Block {
            index: 1,
            timestamp: 1700000000,
            transactions: vec![dummy_tx([1; 32], [2; 32], 10)],
            previous_hash: [2u8; 32],
            hash: [0u8; 32],
        };
        block.hash = block.calculate_hash();

        let serialized = serialize_block(&block).unwrap();
        let deserialized: Block = deserialize_block(&serialized).unwrap();
        assert_eq!(block.hash, deserialized.hash);
        assert_eq!(block.transactions, deserialized.transactions);
        assert_eq!(deserialized.hash, deserialized.calculate_hash());
    }

    #[test]
    fn test_blockchain_serialization_roundtrip() {
        let mut chain = Blockchain::new();
        chain.add_block(vec![dummy_tx([5; 32], [6; 32], 42)]);
        let serialized = serialize_blockchain(&chain).unwrap();
        let deserialized: Blockchain = deserialize_blockchain(&serialized).unwrap();
        assert_eq!(chain.blocks.len(), deserialized.blocks.len());
        assert!(deserialized.is_valid());
        assert_eq!(chain.blocks[1].hash, deserialized.blocks[1].hash);
    }

    #[test]
    fn test_consensus_approves_block_with_majority() {
        let peers = vec![Peer::new(1), Peer::new(2), Peer::new(3)];
        let consensus = FixedPeerConsensus::new(peers);
        let mut chain = Blockchain::new();
        let approved = consensus.propose_block(vec![dummy_tx([1; 32], [2; 32], 100)], &mut chain);
        assert!(approved);
    }

    #[test]
    fn test_consensus_rejects_block_without_majority() {
        let peers = vec![Peer::new(1)];
        let consensus = FixedPeerConsensus::new(peers);
        let mut chain = Blockchain::new();
        let approved = consensus.propose_block(vec![dummy_tx([1; 32], [2; 32], 1)], &mut chain);
        assert!(!approved);
    }
}
