//! Демонстрация работы блокчейна: создание цепочки, консенсус, валидация, сериализация.

// Импорт компонентов из библиотеки
use rustblockchain::{
    Blockchain,             // Основная структура блокчейна
    FixedPeerConsensus,     // Механизм консенсуса
    Peer,                   // Участник пиринговой сети
    Transaction,            // Структура транзакции
    deserialize_block,      // Функция десериализации блока
    deserialize_blockchain, // Функция десериализации блокчейна
    serialize_block,        // Функция сериализации блока
    serialize_blockchain,   // Функция сериализации блокчейна
};

fn main() {
    // 1: Инициализация блокчейна
    println!("Запуск демонстрации блокчейна...\n");

    // Создаём новую цепочку с генезис-блоком
    let mut blockchain = Blockchain::new();
    println!("Блокчейн создан!");
    // Выводим краткую информацию о состоянии блокчейна
    println!("   {}", blockchain.get_chain_info());
    println!();

    // 2: Настройка пиринговой сети
    // Создаём 5 участников сети (пиров), все изначально "честные"
    let peers = (1..=5).map(Peer::new).collect::<Vec<_>>();
    // Инициализируем механизм консенсуса на основе фиксированного списка пиров
    let consensus = FixedPeerConsensus::new(peers);
    println!(
        "Сеть пиров инициализирована ({} участников).",
        consensus.peer_count()
    );
    println!();

    // 3: Предложение блоков через консенсус
    // Формируем несколько пакетов транзакций (каждый пакет - один блок)
    let transaction_batches = vec![
        vec![
            Transaction {
                from: "Address1".to_string(),
                to: "Address2".to_string(),
                amount: 52,
            },
            Transaction {
                from: "Address3".to_string(),
                to: "Address4".to_string(),
                amount: 69,
            },
        ],
        vec![Transaction {
            from: "Address5".to_string(),
            to: "Address6".to_string(),
            amount: 111,
        }],
        vec![
            Transaction {
                from: "Address7".to_string(),
                to: "Address8".to_string(),
                amount: 25,
            },
            Transaction {
                from: "Address9".to_string(),
                to: "Address10".to_string(),
                amount: 90,
            },
        ],
        vec![Transaction {
            from: "Address11".to_string(),
            to: "Address12".to_string(),
            amount: 11,
        }],
        vec![
            Transaction {
                from: "Address13".to_string(),
                to: "Address14".to_string(),
                amount: 250,
            },
            Transaction {
                from: "Address15".to_string(),
                to: "Address16".to_string(),
                amount: 159,
            },
        ],
    ];

    // Предлагаем каждый пакет транзакций как новый блок
    for (i, txs) in transaction_batches.into_iter().enumerate() {
        println!("Предложение блока #{} ({} транзакций):", i + 1, txs.len());
        // Добавление блока через консенсус
        let added = consensus.propose_block(txs, &mut blockchain);

        // Вывод результата голосования
        if added {
            println!("  • Блок принят и добавлен.");
        } else {
            println!("  • Блок отклонён (недостаточно голосов).");
        }
    }
    println!();

    // 4: Вывод всего блокчейна и количества транзакций в блоках
    println!("Текущее состояние блокчейна:");
    blockchain.print_chain();

    // 5: Проверка целостности блокчейна
    // Метод is_valid() пересчитывает хеш каждого блока и проверяет ссылки
    println!("Проверка целостности цепочки...");
    if blockchain.is_valid() {
        println!("Цепочка валидна.");
    } else {
        println!("Цепочка повреждена!");
    }
    println!();

    // 6: Генерация отчёта о сети
    // Рассчитываем суммарный и средний размер блоков
    let serialized_total: usize = blockchain
        .blocks
        .iter()
        .map(|block| serialize_block(block).unwrap().len())
        .sum();

    let block_count = blockchain.blocks.len();
    let average_size = if block_count > 0 {
        serialized_total / block_count
    } else {
        0
    };

    // Вывод статистики
    println!("Отчёт о сети:");
    println!("• Всего блоков: {}", block_count);
    println!(
        "• Всего транзакций: {}",
        blockchain
            .blocks
            .iter()
            .map(|b| b.transactions.len())
            .sum::<usize>()
    );
    println!("  Средний размер блока: {} байт", average_size);

    // 7. Поиск блока по индексу
    println!("\nПоиск блока по индексу...");
    let block_index = 3;
    if let Some(block) = blockchain.get_block(block_index) {
        println!("   Найден блок #{}:", block_index);
        println!("   Хеш: {}", hex::encode(block.hash));
        println!("   Транзакций: {}", block.transactions.len());
        for tx in &block.transactions {
            println!("     • {} → {} : {}", tx.from, tx.to, tx.amount);
        }
    } else {
        println!(".  Блок #{} не найден.", block_index);
    }

    // 8: Сериализация и десериализация (bincode)
    // Один блок
    println!("\nСериализация последнего блока через bincode...");
    if let Some(last_block) = blockchain.blocks.last() {
        // Сериализуем блок в вектор байтов
        match serialize_block(last_block) {
            Ok(encoded) => {
                println!("   Успешно! Размер: {} байт.", encoded.len());
                // Восстанавливаем блок из байтов
                match deserialize_block(&encoded) {
                    Ok(decoded) => {
                        println!("\nДесериализация прошла успешно.");
                        // Сравниваем хеши: оригинальный и восстановленного блока
                        if decoded.hash == last_block.hash {
                            println!(" - Хеш совпадает — данные не повреждены.");
                        } else {
                            println!(" - Ошибка: хеш не совпадает!");
                        }
                    }
                    Err(e) => println!("    Ошибка десериализации: {}", e),
                }
            }
            Err(e) => println!("    Ошибка сериализации: {}", e),
        }
    }

    // Весь блокчейн
    println!("\nСериализация всего блокчейна через bincode...");
    match serialize_blockchain(&blockchain) {
        Ok(encoded_chain) => {
            println!("   Успешно! Размер: {} байт.", encoded_chain.len());

            match deserialize_blockchain(&encoded_chain) {
                Ok(deserialized_chain) => {
                    println!("   Десериализация прошла успешно.");
                    if deserialized_chain.is_valid() {
                        println!("   - Восстановленная цепочка валидна.");
                    } else {
                        println!("    Восстановленная цепочка повреждена!");
                    }
                }
                Err(e) => println!("    Ошибка десериализации цепочки: {}", e),
            }
        }
        Err(e) => println!("    Ошибка сериализации цепочки: {}", e),
    }

    // 9: Имитация атаки
    // Подменяем данные в блоке #2
    println!("\nИмитация атаки: подмена данных в блоке #2...");
    if let Some(block) = blockchain.blocks.get_mut(1) {
        // Удаляем все транзакции
        block.transactions.clear();
        println!(" • Данные блока #2 подменены.");
    }

    // Повторная проверка целостности
    println!("\nПовторная проверка целостности...");
    if blockchain.is_valid() {
        // Если это происходит — есть ошибка в логике валидации!
        println!(" - Цепочка валидна. (ЭТОГО НЕ ДОЛЖНО БЫТЬ!)");
    } else {
        // Ожидаемое поведение: цепочка признана повреждённой
        println!(" - Атака обнаружена! Цепочка признана невалидной.");
    }
    println!();

    // Повторная сериализация после успешной атаки
    println!("Сериализация всего блокчейна через bincode...");
    match serialize_blockchain(&blockchain) {
        Ok(encoded_chain) => {
            println!("   Успешно! Размер: {} байт.", encoded_chain.len());

            match deserialize_blockchain(&encoded_chain) {
                Ok(deserialized_chain) => {
                    println!("   Десериализация прошла успешно.");
                    if deserialized_chain.is_valid() {
                        println!("   - Восстановленная цепочка валидна.");
                    } else {
                        println!("   - Восстановленная цепочка повреждена!");
                    }
                }
                Err(e) => println!("    Ошибка десериализации цепочки: {}", e),
            }
        }
        Err(e) => println!("    Ошибка сериализации цепочки: {}", e),
    }

    // Завершение демонстрации
    println!("\nДемонстрация завершена.");
}
