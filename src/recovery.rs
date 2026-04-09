//! Восстановительные записи: Reed-Solomon (как в WinRAR). Группы блоков + parity shards.

use reed_solomon_erasure::galois_8::ReedSolomon;

/// Размер полосы: столько блоков объединяем для расчёта parity.
pub const STRIPE_DATA_SHARDS: usize = 16;
/// Количество parity-блоков на полосу (можно восстановить до 2 повреждённых блоков в полосе).
pub const STRIPE_PARITY_SHARDS: usize = 2;

/// По списку блоков (переменной длины) считает parity-блоки для одной полосы.
/// Блоки дополняются нулями до максимальной длины в полосе.
/// Возвращает `parity_shards` блоков той же длины.
pub fn encode_stripe(blocks: &[Vec<u8>]) -> Result<Vec<Vec<u8>>, String> {
    if blocks.len() != STRIPE_DATA_SHARDS {
        return Err(format!(
            "expected {} data shards, got {}",
            STRIPE_DATA_SHARDS,
            blocks.len()
        ));
    }
    let max_len = blocks.iter().map(|b| b.len()).max().unwrap_or(0);
    if max_len == 0 {
        return Ok(vec![vec![]; STRIPE_PARITY_SHARDS]);
    }
    let rs = ReedSolomon::new(STRIPE_DATA_SHARDS, STRIPE_PARITY_SHARDS)
        .map_err(|e| e.to_string())?;
    let data_shards: Vec<Vec<u8>> = blocks
        .iter()
        .map(|b| {
            let mut s = b.clone();
            s.resize(max_len, 0);
            s
        })
        .collect();
    let mut parity_shards = vec![vec![0u8; max_len]; STRIPE_PARITY_SHARDS];
    let data_refs: Vec<&[u8]> = data_shards.iter().map(|s| s.as_slice()).collect();
    rs.encode_sep(data_refs.as_slice(), parity_shards.as_mut_slice())
        .map_err(|e| e.to_string())?;
    Ok(parity_shards)
}

/// Восстанавливает недостающие/повреждённые шарды в полосе (16 data + 2 parity).
/// shards: 18 элементов; отсутствующие — None. Все присутствующие должны быть длины shard_len.
pub fn decode_stripe(
    shards: &mut [Option<Vec<u8>>],
    shard_len: usize,
) -> Result<(), String> {
    const TOTAL: usize = STRIPE_DATA_SHARDS + STRIPE_PARITY_SHARDS;
    if shards.len() != TOTAL {
        return Err(format!("expected {} shards, got {}", TOTAL, shards.len()));
    }
    let rs = ReedSolomon::new(STRIPE_DATA_SHARDS, STRIPE_PARITY_SHARDS)
        .map_err(|e| e.to_string())?;
    let present: Vec<bool> = shards.iter().map(|s| s.is_some()).collect();
    if present.iter().filter(|&&p| p).count() < STRIPE_DATA_SHARDS {
        return Err("not enough shards to reconstruct".into());
    }
    let mut with_flag: Vec<(Vec<u8>, bool)> = (0..TOTAL)
        .map(|i| {
            let buf = shards[i]
                .as_ref()
                .map(|v| {
                    let mut b = v.clone();
                    b.resize(shard_len, 0);
                    b
                })
                .unwrap_or_else(|| vec![0u8; shard_len]);
            (buf, present[i])
        })
        .collect();
    rs.reconstruct(&mut with_flag).map_err(|e| e.to_string())?;
    for i in 0..TOTAL {
        if shards[i].is_none() {
            shards[i] = Some(with_flag[i].0.clone());
        }
    }
    Ok(())
}

