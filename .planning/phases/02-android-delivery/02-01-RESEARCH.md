# Phase 2 (v1.0) — RESEARCH: Android delivery

**Requirements:** AND-01 — AND-03  
**Ретроспектива:** отдельная линия `ui-android/`, `tauri.android.conf.json`, заглушки rclone/7-Zip на Android.

## Состояние

| REQ | Доказательство |
|-----|----------------|
| **AND-01** | Заголовок «OmegaZip Android» в `ui-android/index.html`; `tauri.android.conf.json` — имя и bundle. |
| **AND-02** | Баннер без 7-Zip; `rclone_*` / облако возвращают ошибку «не поддерживается» в `src-tauri/src/lib.rs` под `target_os = "android"`. |
| **AND-03** | `ANDROID_BUILD.md` — SDK, init, `npm run android:dev` / `android:build`. |
