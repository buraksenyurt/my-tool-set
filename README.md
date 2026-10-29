# my-tool-set

Geliştirdiğim yararlı masaüstü ve komut satırı araçlarını **tek bir kabuktan** görmek, okumak, kurmak ve çalıştırmak için tasarlanmış çapraz platform bir masaüstü uygulaması. Eski MDI (Multiple Document Interface) uygulamalarının modern bir yorumu: solda araç kataloğu, ortada sekmeli çalışma alanı.

Uçtan uca yapay zeka araçları desteğinde, fazlar halinde geliştirilmektedir.

## Neler olacak

- **Katalog** — her araç bir `tool.toml` manifesti ile tanımlanır: ne işe yarar, hangi yığınla yazıldı, hangi platformda çalışır, nasıl başlatılır.
- **Bilgi** — aracın README'si ve ekran görüntüleri kabuğun içinde okunur.
- **Kur / güncelle** — araçlar kendi repolarının GitHub Releases'ından indirilir, SHA256 ile doğrulanır.
- **Başlat / izle** — araç ayrı bir süreç olarak açılır; durum ve log kabukta izlenir.
- **Gömülü terminal** — CLI araçları (GitDash, Docky) kabuğun içindeki terminal sekmesinde çalışır.
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

Rust (workspace) · Tauri 2 · Svelte 5 + TypeScript + Vite · `toml`/`serde` · `portable-pty` + xterm.js

## Klasör düzeni (hedef)

```
catalog/        araç manifestleri (tool.toml)
crates/         Rust çekirdeği (katalog, süreç yönetimi)
src-tauri/      Tauri kabuğu
src/            Svelte ön yüz
docs/           plan
```

## Dokümanlar

- [Plan ve yol haritası](docs/PLAN.md)
- [Katalog manifest şeması](catalog/README.md)
