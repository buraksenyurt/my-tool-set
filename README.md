# my-tool-set

Geliştirdiğim yararlı masaüstü ve komut satırı araçlarını **tek bir kabuktan** görmek, okumak, kurmak ve çalıştırmak için tasarlanmış çapraz platform bir masaüstü uygulaması. Eski MDI (Multiple Document Interface) uygulamalarının modern bir yorumu: solda araç kataloğu, ortada sekmeli çalışma alanı.

Uçtan uca yapay zeka araçları desteğinde, fazlar halinde geliştirilmektedir.

## Neler olacak

- **Katalog** — her araç bir `tool.toml` manifesti ile tanımlanır: ne işe yarar, hangi yığınla yazıldı, hangi platformda çalışır, nasıl başlatılır.
- **Bilgi** — aracın README'si (kurulu sürümün tag'inden) ve ekran görüntüleri kabuğun içinde okunur; çevrimdışıyken önbellekten.
- **Kur / güncelle** — araçlar kendi repolarının GitHub Releases'ından indirilir, SHA256 ile doğrulanır.
- **Başlat / izle** — araç ayrı bir süreç olarak açılır; durum ve log kabukta izlenir, kabuktan durdurulur. Yerel araçlar kabuktan derlenir.
- **Gömülü terminal** — CLI araçları (GitDash, Docky) manifestten oluşan bir parametre formuyla kabuğun içindeki terminal sekmesinde çalışır; renkler, etkileşimli menüler ve klavye girdisi desteklenir.
- **Modüller** *(MVP sonrası)* — Tauri ile yazılmış araçların (easy-git, java-atlas) kabuğun içinde kendi sekmesinde açılması değerlendirilecek.

## Kapsamdaki araçlar

| Araç | Tür | Yığın | Kaynak | Barındırma |
|---|---|---|---|---|
| easy-git | GUI | Rust · Tauri 2 · Svelte 5 | [buraksenyurt/easy-git](https://github.com/buraksenyurt/easy-git) releases | Dış süreç |
| java-atlas | GUI | Rust · Tauri 2 · Svelte 5 | [buraksenyurt/java-atlas](https://github.com/buraksenyurt/java-atlas) releases | Dış süreç |
| DockerCity | GUI | C# · WinUI 3 (yalnızca Windows) | [buraksenyurt/docker-city](https://github.com/buraksenyurt/docker-city) releases | Dış süreç |
| GitDash | CLI | Zig | `coding-vibes` (yerel) | Gömülü terminal |
| Docky | CLI | C# · .NET tool | `coding-vibes` (yerel) | Gömülü terminal |

## Yığın

Rust (workspace) · Tauri 2 · Svelte 5 + TypeScript + Vite · `toml`/`serde` · `reqwest` + `rustls` · `pulldown-cmark` + `ammonia` · `portable-pty` + xterm.js

## Klasör düzeni

```
catalog/              araç manifestleri (tool.toml)
crates/mts-core       domain: tool.toml, release.json, platformlar, sürümler (I/O yok)
crates/mts-catalog    katalog okuma, settings.toml, release.json indirme, kurulum, README işleme
crates/mts-runner     araçları ve derlemeleri başlatma, log toplama, süreç ağacını durdurma, terminal (pty) oturumları
src-tauri/            Tauri kabuğu: komutlar, DTO'lar, izinler
src/                  Svelte 5 ön yüz
  lib/api.ts          Rust ile konuşan tek dosya (tarayıcıda demo veriye düşer)
  lib/bindings/       Rust tiplerinden üretilen TypeScript tipleri (ts-rs)
  lib/shell/          kenar çubuğu, sekme çubuğu, çalışma alanı, durum çubuğu
  lib/views/          sekme içerikleri
docs/PLAN.md          plan ve yol haritası
```

## Çalıştırma

```bash
npm install
npm run tauri dev          # masaüstü uygulaması
npm run dev                # yalnızca ön yüz, tarayıcıda demo verilerle (http://localhost:1420)
cargo test --workspace     # testler; src/lib/bindings altındaki TypeScript tiplerini de yeniler
```

Geliştirme derlemesinde `MTS_RELEASE_BASE` ortam değişkeni, release'leri GitHub yerine aynı URL düzenindeki başka bir sunucudan okutur (yayınlamadan kurulum denemek için):

```bash
MTS_RELEASE_BASE=http://127.0.0.1:8765 npm run tauri dev
```

Gereksinimler easy-git ve java-atlas ile aynı: Rust (1.90+), Node.js LTS, Windows'ta WebView2 ve Visual Studio Build Tools (C++), Linux'ta `libwebkit2gtk-4.1-dev`. Windows'ta derleme sırasında linker hatası alınırsa komutları *Developer Command Prompt for Visual Studio* içinden çalıştır.

## Sürümler

Hazır derlemeler [Releases](https://github.com/buraksenyurt/my-tool-set/releases) sayfasında. Yeni sürüm için `package.json`, `src-tauri/tauri.conf.json` ve `Cargo.toml` içindeki sürümü güncelleyip commit'le, ardından tag'i push'la (`git tag v0.1.0 && git push origin v0.1.0`). [`release`](.github/workflows/release.yml) workflow'u araç repolarındakiyle aynı.

## Dokümanlar

- [Plan ve yol haritası](docs/PLAN.md)
- [Katalog manifest şeması](catalog/README.md)
