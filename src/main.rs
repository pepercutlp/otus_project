use rustblockchain::{Blockchain, FixedPeerConsensus, Peer, serialize_block};

fn main() {
    let mut blockchain = Blockchain::new(); // Создание блокчейна

    // Фиксированный список пиров сети
    let peers = vec![
        Peer::new(1),
        Peer::new(2),
        Peer::new(3),
        Peer::new(4),
        Peer::new(5),
    ];

    // Создание сети пиров
    let consensus = FixedPeerConsensus::new(peers);

    let transactions = vec![
        "Транзакция 1,2,3",
        "Транзакция 4",
        "Транзакция 5,6",
        "Транзакция 7",
        "Транзакция 8,9,10",
        "Транзакция 11",
        "Транзакция 12,13",
        "Транзакция 14",
        "Транзакция 15",
        "Транзакция 16,17,18",
    ];

    // Предложение блоков через механизм консенсуса
    for tx in transactions {
        let added = consensus.propose_block(tx.to_string(), &mut blockchain);
        println!(
            "Транзакция \"{}\" {} в блокчейн (пиров: {}).",
            tx,
            if added {
                "добавлена"
            } else {
                "НЕ добавлена"
            },
            consensus.peer_count()
        );
    }

    // Вывод цепочки и проверка валидности
    blockchain.print_chain();

    println!("Цепочка валидна: {}", blockchain.is_valid());

    // Вывод информации по определенному блоку
    if let Some(block) = blockchain.get_block(7) {
        println!("Блок 7: {}", block.data);
    } else {
        println!("Блок не найден");
    }
    if let Some(block) = blockchain.get_block(11) {
        println!("Блок 11: {}", block.data);
    } else {
        println!("Блок 11: Блок не найден");
    }

    //Вывод информации о блокчейне
    println!("{}", blockchain.get_chain_info());

    // Имитация атаки (подмена данных в одном блоке)
    println!("Попытка внесения изменений в блок #6");
    blockchain.blocks[6].data = "изменения_в_блок".to_string();

    println!("Цепочка валидна? {}", blockchain.is_valid());

    // Пример использования bincode: сериализация и десериализация последнего блока
    if let Some(last_block) = blockchain.blocks.last() {
        let encoded = serialize_block(last_block).expect("Ошибка сериализации блока");
        println!("Сериализованный блок занимает {} байт", encoded.len());

        let decoded_block =
            rustblockchain::deserialize_block(&encoded).expect("Ошибка десериализации блока");
        println!(
            "Восстановленный блок: index={}, hash={}",
            decoded_block.index,
            &decoded_block.hash[..10]
        );
    }
}
