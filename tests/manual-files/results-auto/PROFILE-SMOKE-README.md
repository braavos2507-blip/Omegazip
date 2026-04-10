# profile-smoke-last.txt

Генерируется скриптом:

```bash
bash scripts/profile-compress-heavy-smoke.sh
# или
npm run measure:profile-smoke
```

Файл **`profile-smoke-last.txt`** в этом каталоге по умолчанию в `.gitignore` (локальный артефакт). Содержит время и размеры ZIP vs chunked `.oz` на ~30 MiB синтетики с сильным дедупом.

Полноценный профиль CPU — [docs/MEASURABLE-QUALITY.md](../../../docs/MEASURABLE-QUALITY.md) §C и `scripts/profile-compress-local.sh`.
