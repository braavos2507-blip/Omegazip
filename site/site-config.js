/**
 * Loads site/config.json + checksums.json; wires donate, downloads, contact, OS hints.
 */
(function () {
  /* Fallback: apex → www if edge 301 did not run (e.g. cached HTML). */
  if (location.hostname === "omegazip.ru") {
    location.replace(
      "https://www.omegazip.ru" + location.pathname + location.search + location.hash
    );
    return;
  }

  const PLACEHOLDER_PREFIX = "REPLACE_ME_";

  function isPlaceholder(value) {
    return typeof value !== "string" || value.startsWith(PLACEHOLDER_PREFIX);
  }

  function joinUrl(base, path) {
    if (!base || !path) return null;
    const b = base.replace(/\/+$/, "");
    const p = path.replace(/^\/+/, "");
    return `${b}/${p}`;
  }

  function detectOs() {
    const ua = navigator.userAgent || "";
    const plat = (navigator.userAgentData && navigator.userAgentData.platform) || navigator.platform || "";
    if (/Win/i.test(plat) || /Windows/i.test(ua)) return "windows";
    if (/Mac/i.test(plat) || /Macintosh/i.test(ua)) return "macos";
    if (/Linux/i.test(plat) || /Linux/i.test(ua)) return "linux";
    return null;
  }

  function buildDownloadUrl(base, file, campaign) {
    const url = joinUrl(base, file);
    if (!url) return null;
    const u = new URL(url);
    u.searchParams.set("utm_source", "omegazip_site");
    u.searchParams.set("utm_medium", "download");
    if (campaign) u.searchParams.set("utm_campaign", campaign);
    return u.toString();
  }

  function applyDonate(cfg) {
    const payment = cfg.donatePaymentUrl;
    if (isPlaceholder(payment)) return;
    document.querySelectorAll("[data-donate-payment]").forEach((el) => {
      el.href = payment;
      if (el.tagName === "A") {
        el.target = "_blank";
        el.rel = "noopener noreferrer";
      }
    });
  }

  function applyContact(cfg) {
    if (!cfg.contactEmail || isPlaceholder(cfg.contactEmail)) return;
    document.querySelectorAll("[data-contact-email]").forEach((el) => {
      if (el.tagName === "A") el.href = `mailto:${cfg.contactEmail}`;
      else el.textContent = cfg.contactEmail;
    });
  }

  function applyChecksums(checksums, cfg) {
    const files = cfg.downloads || {};
    const map = { windows: files.windows, macos: files.macos, linux: files.linux };
    for (const [os, filename] of Object.entries(map)) {
      if (!filename) continue;
      const hash = checksums[filename];
      document.querySelectorAll(`[data-checksum-for="${os}"]`).forEach((el) => {
        if (hash) {
          el.textContent = `SHA-256: ${hash}`;
          el.hidden = false;
        }
      });
    }
  }

  function applyDownloads(cfg, checksums) {
    const base = cfg.downloadsBaseUrl;
    const os = detectOs();
    const urls = {};

    const map = [
      ["dl-windows", "windows", cfg.downloads?.windows],
      ["dl-macos", "macos", cfg.downloads?.macos],
      ["dl-linux", "linux", cfg.downloads?.linux],
    ];

    for (const [id, osKey, file] of map) {
      const el = document.getElementById(id);
      if (!el) continue;
      if (!file || isPlaceholder(base)) {
        el.classList.add("dl-pending");
        el.setAttribute("aria-disabled", "true");
        el.title = "Ссылка появится после публикации релиза";
        continue;
      }
      const url = buildDownloadUrl(base, file, id.replace("dl-", ""));
      if (url) {
        el.href = url;
        urls[osKey] = url;
        el.removeAttribute("aria-disabled");
        el.classList.remove("dl-pending");
      }
    }

    const hero = document.getElementById("dl-hero");
    if (hero && os && urls[os]) {
      hero.href = urls[os];
      const labels = { windows: "Windows", macos: "macOS", linux: "Linux" };
      hero.textContent = `Скачать для ${labels[os]}`;
      const hint = document.getElementById("dl-hero-hint");
      if (hint) hint.textContent = `Определена ваша ОС: ${labels[os]}. Ниже — все варианты.`;
    }

    document.querySelectorAll("[data-dl-os]").forEach((card) => {
      const key = card.getAttribute("data-dl-os");
      const on = os !== null && key === os;
      card.classList.toggle("dl-card-recommended", on);
      const badge = card.querySelector("[data-dl-badge]");
      if (badge) badge.hidden = !on;
    });

    applyChecksums(checksums, cfg);
  }

  function applyVersion(cfg) {
    const v = cfg.productVersion;
    if (!v) return;
    document.querySelectorAll("[data-product-version]").forEach((el) => {
      el.textContent = v;
    });
  }

  function applyAnalytics(cfg) {
    const id = cfg.analyticsMetrikaId;
    if (!id || isPlaceholder(id)) return;
    (function (m, e, t, r, i, k, a) {
      m[i] = m[i] || function () { (m[i].a = m[i].a || []).push(arguments); };
      m[i].l = 1 * new Date();
      for (var j = 0; j < document.scripts.length; j++) {
        if (document.scripts[j].src === r) return;
      }
      k = e.createElement(t);
      a = e.getElementsByTagName(t)[0];
      k.async = 1;
      k.src = r;
      a.parentNode.insertBefore(k, a);
    })(window, document, "script", "https://mc.yandex.ru/metrika/tag.js", "ym");
    window.ym(Number(id), "init", {
      clickmap: true,
      trackLinks: true,
      accurateTrackBounce: true,
    });
  }

  Promise.all([
    fetch("config.json", { cache: "no-store" }).then((r) => {
      if (!r.ok) throw new Error(`config.json ${r.status}`);
      return r.json();
    }),
    fetch("checksums.json", { cache: "no-store" })
      .then((r) => (r.ok ? r.json() : {}))
      .catch(() => ({})),
  ])
    .then(([cfg, checksums]) => {
      applyDonate(cfg);
      applyDownloads(cfg, checksums);
      applyContact(cfg);
      applyVersion(cfg);
      applyAnalytics(cfg);
    })
    .catch(() => {
      /* static preview */
    });
})();
