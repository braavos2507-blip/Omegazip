//! Разбиение на чанки для дедупликации (как Borg/restic).

/// Рекомендуемый размер чанка (64 KiB) — баланс дедупа и накладных расходов.
pub const DEFAULT_CHUNK_SIZE: usize = 64 * 1024;

/// Разбивает данные на чанки фиксированного размера; последний чанк может быть меньше.
#[inline]
pub fn chunks(data: &[u8], chunk_size: usize) -> impl Iterator<Item = &[u8]> {
    let size = chunk_size.max(1);
    (0..data.len()).step_by(size).map(move |i| {
        let end = (i + size).min(data.len());
        &data[i..end]
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunks() {
        let data: Vec<u8> = (0..200).collect();
        let c: Vec<&[u8]> = chunks(&data, 64).collect();
        assert_eq!(c.len(), 4);
        assert_eq!(c[0].len(), 64);
        assert_eq!(c[3].len(), 8);
    }
}
