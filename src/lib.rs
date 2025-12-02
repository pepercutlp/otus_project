use chrono::Utc; // Получение текущего времени в UTC
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256}; // Хеширование SHA-256 // Сериализация

// Структура блока
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub index: u64,            // Номер блока в цепочке
    pub timestamp: String,     // Время создания
    pub data: String,          // Транзакции
    pub previous_hash: String, // Хеш предыдущего блока
    pub hash: String,          // Хеш текущего блока
}

// Функция вычесления хеша (вычисляет хеш блока на основе всех его полей и возвращает хеш)
fn calculate_hash(index: u64, timestamp: &str, data: &str, previous_hash: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(format!("{}{}{}{}", index, timestamp, data, previous_hash).as_bytes());
    format!("{:x}", hasher.finalize())
}

// Функция создания нового блока на основе предыдущего
fn create_block(data: String, previous_block: &Block) -> Block {
    let index = previous_block.index + 1;
    let timestamp = Utc::now().to_rfc3339();
    let previous_hash = previous_block.hash.clone();
    let hash = calculate_hash(index, &timestamp, &data, &previous_hash);

    Block {
        index,
        timestamp,
        data,
        previous_hash,
        hash,
    }
}

//Функция создания генезиз-блока
fn create_genesis_block() -> Block {
    Block {
        index: 0,
        timestamp: Utc::now().to_rfc3339(),
        data: "Genesis Block".to_string(),
        previous_hash: "0".to_string(),
        hash: calculate_hash(0, &Utc::now().to_rfc3339(), "Genesis Block", "0"),
    }
}

//Струтура блокчейна
#[derive(Debug, Serialize, Deserialize)]
pub struct Blockchain {
    pub blocks: Vec<Block>, // Хранит вектор блоков (основную цепочку)
}

impl Blockchain {
    // Создание новой цепочки с добавлением генезис-блока
    pub fn new() -> Self {
        let mut chain = Blockchain { blocks: vec![] };
        chain.blocks.push(create_genesis_block());
        chain
    }

    // Добавление нового блока в конец цепочки на основе предыдущего блока
    pub fn add_block(&mut self, data: String) {
        let last_block = self.blocks.last().unwrap();
        let new_block = create_block(data, last_block);
        self.blocks.push(new_block);
    }

    // Метод вывода информации о блоках
    pub fn print_chain(&self) {
        for block in &self.blocks {
            println!("--- Block {} ---", block.index);
            println!("Hash: {}", &block.hash);
            println!("Data: {}", block.data);
            println!("Prev: {}", &block.previous_hash);
            println!();
        }
    }

    // Метод вывода информации о блоке по номеру
    pub fn get_block(&self, index: usize) -> Option<&Block> {
        self.blocks.get(index)
    }

    // Метод вывода общей информации о блокчейне
    pub fn get_chain_info(&self) -> String {
        format!(
            "Блоков: {}, Валидно: {}, Последний хеш: {}",
            self.blocks.len(),
            self.is_valid(),
            &self.blocks.last().unwrap().hash[..10]
        )
    }

    // Проверка целостности всей цепочки
    pub fn is_valid(&self) -> bool {
        for i in 1..self.blocks.len() {
            let current = &self.blocks[i];
            let previous = &self.blocks[i - 1];

            // Проверяем, что хеш совпадает
            let recalculated_hash = calculate_hash(
                current.index,
                &current.timestamp,
                &current.data,
                &current.previous_hash,
            );
            if current.hash != recalculated_hash {
                return false;
            }

            // Проверяем, что ссылка на предыдущий блок корректна
            if current.previous_hash != previous.hash {
                return false;
            }
        }
        true
    }
}

// Модель участников сети (пиров) и консенсуса
pub type PeerId = u32; // Идентификатор пира

// Моделирование пира
#[derive(Debug, Clone)]
pub struct Peer {
    pub id: PeerId,
    pub is_honest: bool, // Флаг, который можно использовать для моделирования "честных" и "нечестных" пиров
}

impl Peer {
    pub fn new(id: PeerId) -> Self {
        Self {
            id,
            is_honest: true,
        }
    }

    // Упрощённое голосование пира по транзакции
    pub fn vote_for_transaction(&self, _data: &str) -> bool {
        if self.is_honest { true } else { true }
    }
}

// Консенсус с фиксированным списком пиров
pub struct FixedPeerConsensus {
    pub peers: Vec<Peer>,
}

impl FixedPeerConsensus {
    pub fn new(peers: Vec<Peer>) -> Self {
        Self { peers }
    }

    /// Возвращает количество пиров в сети
    pub fn peer_count(&self) -> usize {
        self.peers.len()
    }

    // Блок добавляется в цепочку, если больше, чем (N+1)/2 пиров подтвердили транзакцию
    fn majority_threshold(&self) -> usize {
        (self.peers.len() + 1) / 2
    }

    // Предложение включить транзакцию в цепочку
    pub fn propose_block(&self, data: String, blockchain: &mut Blockchain) -> bool {
        if self.peers.is_empty() {
            return false;
        }

        let mut approvals = 0usize;
        for peer in &self.peers {
            if peer.vote_for_transaction(&data) {
                approvals += 1;
            }
        }

        // Добавление в цепочку если больше, чем (N+1)/2 пиров подтвердили транзакцию
        let threshold = self.majority_threshold();
        if approvals > threshold {
            blockchain.add_block(data);
            true
        } else {
            false
        }
    }
}

// Сериализация через bincode (сохранение/восстановление блоков и цепочки в компактном бинарном виде)
pub fn serialize_block(block: &Block) -> Result<Vec<u8>, bincode::Error> {
    bincode::serialize(block) // Сериализация блока в бинарный формат
}

pub fn deserialize_block(bytes: &[u8]) -> Result<Block, bincode::Error> {
    bincode::deserialize(bytes) // Десериализация блока из бинарного формата
}

pub fn serialize_blockchain(chain: &Blockchain) -> Result<Vec<u8>, bincode::Error> {
    bincode::serialize(chain) // Сериализация всей цепочки
}

pub fn deserialize_blockchain(bytes: &[u8]) -> Result<Blockchain, bincode::Error> {
    bincode::deserialize(bytes) // Десериализация всей цепочки
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    // Проверка корректности генезис-блока
    fn test_genesis_block() {
        let chain = Blockchain::new();
        assert_eq!(chain.blocks[0].index, 0);
        assert_eq!(chain.blocks[0].data, "Genesis Block");
    }

    // Проверка валидности после добавления блока
    #[test]
    fn test_chain_validity() {
        let mut chain = Blockchain::new();
        chain.add_block("Test 1".to_string());
        chain.add_block("Test 2".to_string());
        assert!(chain.is_valid());
    }

    // Проверка валидности после внения изменений в блок
    #[test]
    fn test_chain_invalid_after_hack() {
        let mut chain = Blockchain::new();
        chain.add_block("Test 1".to_string());
        chain.add_block("Test 2".to_string());

        chain.blocks[1].data = "изменения_в_блок".to_string();
        assert!(!chain.is_valid());
    }
}
