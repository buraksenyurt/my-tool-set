# my-tool-set — Proje Plan Dokümanı

> **Atölye:** Fikirler Atölyesi
> **Doküman sürümü:** v0.2 — 29 Ekim 2026
> **Kod:** `my-tool-set` (araçlar kendi repolarında, kabuk onları GitHub Releases'tan alır)
> **Durum:** Araç repoları ve release workflow'ları hazır — sıradaki: ilk release'ler, ardından Faz 0

---

## 1. Tek Cümlelik Tanım

Kendi geliştirdiğim masaüstü ve komut satırı araçlarını tek bir kabukta listeleyen, ne işe yaradıklarını anlatan, GitHub Releases'tan indirip güncel tutan, başlatıp izleyen ve zamanla kendi sekmelerinde barındıran çapraz platform bir masaüstü uygulaması.

## 2. Atölyenin Amacı

| Katman | Hedef |
|---|---|
| **Ürün** | "Hangi aracım vardı, ne işe yarıyordu, nasıl çalıştırıyordum?" sorusunu ortadan kaldırmak. Araçları her seferinde kendi klasörlerinde aramak yerine tek bir kabuktan açmak. |
| **Pratik** | Tauri 2'nin ileri konuları: **plugin yazımı**, çoklu pencere/webview, süreç yönetimi ve log akışı (channel), pty ile gömülü terminal, asset protocol ile yerel dosya sunma. Rust tarafında manifest tasarımı (`serde` + `toml`), indirme/doğrulama, platform soyutlaması, süreç yaşam döngüsü. Svelte 5 ile sekmeli/bölünebilir çalışma alanı. |

Bu projede öğreti dokümanı yazılmıyor; doğrudan fazlar halinde kodlanıyor. Kararlar bu planda tutuluyor.

**Kapsam kararları:**

- Olgunlaşan araçlar kendi repolarında yaşar ve GitHub Releases ile dağıtılır; `my-tool-set` yalnızca manifest tutar, release'leri indirir ve çağırır. Yerel klasör bağımlılığı yok (bkz. §5.1).
- Kabuk araçları **değiştirmez**; modül entegrasyonu (Faz 5–6) gerektiğinde araç tarafında geriye uyumlu bir refactor yapılır.
- Oyunlar (MissionControl, LunarRace, Walkure) ve web uygulamaları (DoDi, Friendly) ilk kapsam dışında. Manifest şeması bunları sonradan eklemeye engel olmayacak.

## 3. Envanter ve Seçim

| Proje | Tür | Yığın | Platform | Karar |
|---|---|---|---|---|
| easy-git | GUI | Rust · Tauri 2 · Svelte 5 | Windows (diğerleri denenmedi) | ✅ dış süreç → modül · [kendi reposu](https://github.com/buraksenyurt/easy-git) |
| java-atlas | GUI | Rust · Tauri 2 · Svelte 5 | Windows · Linux · macOS | ✅ dış süreç → modül · [kendi reposu](https://github.com/buraksenyurt/java-atlas) |
| DockerCity | GUI | C# · WinUI 3 | Yalnızca Windows | ✅ dış süreç · [kendi reposu](https://github.com/buraksenyurt/docker-city) |
| GitDash | CLI | Zig | Windows · Linux (WSL) | ✅ gömülü terminal · `coding-vibes`'ta |
| Docky | CLI | C# · .NET tool | Windows | ✅ gömülü terminal · `coding-vibes`'ta |
| MissionControl, LunarRace, Walkure | Oyun | Rust/Bevy · Python · Go | — | ⏸ sonraya ("Eğlence" kategorisi) |
| DoDi, Friendly | Web | ASP.NET · EF · Docker | — | ⏸ sonraya (compose ile ayağa kalkan servis türü) |

## 4. Teknoloji Kararı

**Seçilen:** Tauri 2 + Svelte 5 + TypeScript + Vite, Rust workspace.

| Seçenek | Artı | Eksi |
|---|---|---|
| **Tauri 2 + Svelte 5** ✅ | easy-git ve java-atlas ile aynı yığın; Rust core crate'leri ve Svelte bileşenleri doğrudan alınabilir. Hafif, çapraz platform. Tauri plugin sistemi "araç = modül" fikrine birebir uyuyor. | Gerçek docking/MDI kütüphanesi hazır değil, çalışma alanı düzeni elle yazılacak. |
| Avalonia + Dock | Gerçek MDI/docking hissi, .NET tecrübesi. | Tauri araçlarını gömmek pratikte mümkün değil; hepsi dış süreç kalır. |
| Electron | Gömülü terminal ekosistemi en olgun. | Ağır; Rust çekirdekleri sidecar olarak koşar, tekrar kullanım zayıf. |

**Neden pencere gömme (reparenting) yok?** Başka bir sürecin penceresini kendi penceremize almak Windows'ta `SetParent` ile kırılgan biçimde mümkün, Linux'ta X11'e bağlı, macOS'ta mümkün değil. Çapraz platform hedefle çelişiyor. Bunun yerine **hibrit barındırma** seçildi.

## 5. Barındırma Modeli (hibrit)

Her araç manifestinde `run.host` alanıyla üç moddan birini seçer:

| Mod | Ne olur | Kim |
|---|---|---|
| `external` | Araç ayrı bir süreç ve pencere olarak açılır. Kabuk PID, durum (çalışıyor / çıktı kodu), stdout/stderr loglarını izler; durdurabilir. | DockerCity; Faz 5–6'ya kadar easy-git ve java-atlas |
| `terminal` | Araç kabuğun içindeki bir terminal sekmesinde (pty + xterm.js) çalışır. Manifestteki `run.params` ile küçük bir parametre formu çıkar. | GitDash, Docky |
| `module` | Araç bir **Tauri plugin**'i olarak kabuğa derlenir; arayüzü kabuğun içinde kendi sekmesinde açılır. | easy-git (Faz 5), java-atlas (Faz 6) |

Modül yaklaşımı *(açık karar, bkz. §10)*: aracın `src-tauri` içindeki komutları bir `tauri-plugin-<araç>` crate'ine taşınır (komutlar `plugin:<araç>|<komut>` ad alanına girer, çakışma olmaz). Svelte arayüzü kök bileşeni dışa açan bir pakete dönüşür. Araç kendi başına çalışmaya devam eder: kendi `src-tauri`'si yalnızca bu plugin'i kaydeden ince bir kabuk olur.

### 5.1 Dağıtım: GitHub Releases

Olgunlaşan araçlar ayrı repolarda (Ekim 2026'da `coding-vibes`'tan geçmişleriyle taşındı). Her repoda:

- `ci.yml` — `main`'e push ve PR'larda kontrol + testler.
- `release.yml` — `v*.*.*` tag'i push'lanınca: tag ile proje sürümünü karşılaştırır → taslak release açar → platform matrisinde derler ve dosyaları yükler → `release.json`'u (dosya, SHA256, boyut, çalıştırılacak giriş) üretip yükler → yayınlar. Bir platform düşerse release taslak kalır.
- `scripts/release/` — `check-version.sh`, `fragment.sh`, `compose-manifest.sh` (üç repoda aynı).

| Araç | Platformlar | Taşınabilir paket | Yükleyiciler |
|---|---|---|---|
| easy-git, java-atlas | windows-x64 · linux-x64 · macos-arm64 | `.zip` (exe) · `.AppImage` · `.app.tar.gz` | nsis, msi · deb · dmg |
| DockerCity | windows-x64 · windows-arm64 | self-contained `.zip` | — |

Kabuk yalnızca `release.json`'u ve `portable` dosyasını kullanır: indirir, SHA256'yı doğrular, `<veri-klasörü>/tools/<id>/<sürüm>/` altına açar. Şema: [catalog/README.md](../catalog/README.md). Repo ayarları: `v*` tag'leri ruleset ile korunuyor, release'ler değiştirilemez (immutable).

GitDash ve Docky şimdilik `coding-vibes`'ta; yerel kök + yol ile çalışırlar. Olgunlaşınca aynı kalıpla kendi repolarına taşınırlar.

## 6. Mimari

### 6.1 Klasör yapısı (hedef)

```
my-tool-set/
├─ catalog/                 araç manifestleri (her araç için tool.toml)
├─ crates/
│  ├─ mts-core/             domain: Tool, Catalog, HostMode, Platform, doğrulama (altyapıdan habersiz)
│  ├─ mts-catalog/          manifest okuma, kök çözümleme, release.json okuma, indirme + SHA256 doğrulama
│  └─ mts-runner/           süreç başlatma/izleme, pty oturumları
├─ src-tauri/               Tauri kabuğu: komutlar, state, plugin kayıtları
├─ src/                     Svelte 5 ön yüz
│  ├─ shell/                kenar çubuğu, sekme çubuğu, çalışma alanı, durum çubuğu
│  ├─ views/                araç kartı, README görünümü, log paneli, terminal
│  └─ modules/              modül araçlarının sekme bağlayıcıları
└─ docs/  PLAN.md
```

### 6.2 Manifest ve kökler

Şema taslağı: [catalog/README.md](../catalog/README.md). `[release]` araçları GitHub repo adıyla tanımlanır. Release'i olmayan araçlar mutlak yol içermez; `source.root` adı makineye özgü `settings.toml [roots]` ile diske bağlanır.

### 6.3 Tauri köprüsü (taslak)

```rust
#[tauri::command] fn list_tools(state) -> Result<Vec<ToolSummaryDto>, AppError>;
#[tauri::command] fn get_tool(id: String, state) -> Result<ToolDetailsDto, AppError>;   // README + ekran görüntüsü yolları + platform durumu
#[tauri::command] async fn launch_tool(id: String, args: Vec<String>, on_event: Channel<ProcessEvent>, state) -> Result<RunId, AppError>;
#[tauri::command] fn stop_tool(run: RunId, state) -> Result<(), AppError>;
#[tauri::command] async fn open_terminal(id: Option<String>, params: Map, on_data: Channel<Vec<u8>>, state) -> Result<PtyId, AppError>;
#[tauri::command] fn write_terminal(pty: PtyId, data: Vec<u8>, state) -> Result<(), AppError>;
#[tauri::command] async fn check_updates(state) -> Result<Vec<UpdateDto>, AppError>;              // release.json'lardaki son sürümler
#[tauri::command] async fn install_tool(id: String, version: Option<String>, on_progress: Channel<DownloadProgress>, state) -> Result<InstalledDto, AppError>;
```

### 6.4 Ekran düzeni

```
┌───────────────┬────────────────────────────────────────────────┐
│ ⌕ Ara         │ [Ana sayfa] [easy-git ●] [GitDash ⌨] [README] ×│  ← sekmeler
│               ├────────────────────────────────────────────────┤
│ GIT           │                                                │
│  ● easy-git   │       Aktif sekme: araç kartı / README /        │
│    GitDash    │       log paneli / terminal / modül arayüzü    │
│ JAVA          │                                                │
│    java-atlas │                                                │
│ DOCKER        │                                                │
│    DockerCity │                                                │
│    Docky      │                                                │
├───────────────┴────────────────────────────────────────────────┤
│ 2 araç çalışıyor · coding-vibes: C:\…\coding-vibes · Windows   │  ← durum çubuğu
└────────────────────────────────────────────────────────────────┘
```

MDI hissi sekmelerle başlar; Faz 7'de çalışma alanı yatay/dikey bölünebilir hale gelir.

## 7. Yol Haritası

### Faz 0 — Atölye kurulumu
- Tauri 2 + Svelte 5 iskeleti, Rust workspace (`mts-core`, `mts-catalog`, `mts-runner`, `src-tauri`).
- Boş kabuk düzeni: kenar çubuğu, sekme çubuğu, durum çubuğu.
- Windows notları (WebView2, VS Build Tools — easy-git'te yaşanan sorun) ve Linux notları (webkit2gtk).
- Kabuğun kendi `ci.yml` / `release.yml`'ı (araç repolarındaki kalıp).

### Faz 1 — Katalog ve release'ler
- Manifest şeması → Rust tipleri (`serde` + `toml`), şema sürümü, doğrulama hataları.
- `release.json` okuma (GitHub API, önbellek, 60 istek/saat sınırına dikkat), platform anahtarı seçimi.
- İndirme + SHA256 doğrulama + arşiv açma (`zip`, `tar.gz`, `file`), kurulu sürümlerin takibi.
- `settings.toml` ve kök çözümleme (`[source]` araçları); platform durumu hesaplama (`yes/untested/no`).
- `cargo test` ile fixture manifestleri ve `release.json` örnekleri.

### Faz 2 — Kabuk ve bilgi görünümü
- Kenar çubuğunda kategorili araç listesi, arama.
- Araç kartı: özet, yığın, platform rozetleri, eylem düğmeleri.
- README görünümü: markdown → HTML (release tag'indeki README), göreli görsellerin çözülmesi, ekran görüntüsü galerisi.
- Kur / güncelle düğmeleri, kurulu ve son sürüm bilgisi.
- Sekme yönetimi (aç/kapat/sırala).

### Faz 3 — Süreç yönetimi (`external`)
- Başlat/durdur, PID ve çıktı kodu, stdout/stderr'in channel ile akması, log paneli.
- Kurulu değilse "önce kur" akışı; `[source]` araçlarında "ikili yok → build et".
- `release.json`'da platformu olmayan aracın (ör. DockerCity Linux'ta) devre dışı gösterilmesi.

### Faz 4 — Gömülü terminal (`terminal`)
- `portable-pty` + xterm.js, yeniden boyutlandırma, çoklu terminal sekmesi.
- `run.params`'tan parametre formu (klasör seçici, sayı alanı) → komut satırı.
- GitDash ve Docky'nin kabukta çalışması; GitDash'in ANSI renkli çıktısı doğru görünmeli.

### Faz 5 — İlk modül: easy-git (`module`) *(MVP)*
- easy-git `src-tauri` komutlarının `tauri-plugin-easy-git` crate'ine taşınması (easy-git tek başına çalışmaya devam eder).
- Svelte arayüzünün kök bileşeni dışa açan pakete dönüşmesi; kabukta sekmede açılması.
- Bağımlılık stratejisi: tag'e sabitlenmiş git bağımlılığı (`git = "https://github.com/buraksenyurt/easy-git", tag = "v…"`); yerel klasör bağımlılığı yok.

### Faz 6 — İkinci modül: java-atlas
- Faz 5 kalıbının tekrarı; ortak noktalar kabukta bir "modül sözleşmesi"ne (TS arayüzü + Rust trait) dönüşür.

### Faz 7 — Çalışma alanı ve kalıcılık
- Bölünebilir çalışma alanı (yatay/dikey), sürükle-bırak sekmeler.
- Oturum kalıcılığı: açık sekmeler, son kullanılanlar, favoriler.
- Kabuk içinden manifest ekleme/düzenleme.

### Faz 8 — Paketleme ve genişleme *(opsiyonel)*
- Kabuğun kendini güncellemesi (Tauri updater) ve imzalama.
- Yeni türler: oyunlar ("Eğlence"), compose ile ayağa kalkan web uygulamaları (DoDi, Friendly) — DockerCity'yle bağlantı kurulabilir.

## 8. Doküman Düzeni

Yalnızca bu plan. Faz kararları ve öğrenilenler buraya işlenir; ayrı öğreti dokümanı yok.

## 9. MVP Kabul Kriterleri (Faz 0–5)

- [ ] Kabuk Windows'ta açılıyor, Linux'ta en az katalog ve terminal çalışıyor.
- [ ] Beş aracın tamamı kenar çubuğunda kategorili listeleniyor; README'leri görselleriyle okunabiliyor.
- [ ] easy-git, java-atlas ve DockerCity release'ten indirilip SHA256 ile doğrulanarak kuruluyor; yeni sürüm olduğunda kabuk bunu gösteriyor.
- [ ] DockerCity, easy-git, java-atlas dış süreç olarak başlatılıp durdurulabiliyor; loglar görünüyor.
- [ ] GitDash ve Docky gömülü terminalde parametre formuyla çalışıyor.
- [ ] easy-git kabuğun içinde sekmede açılıyor ve kendi başına da çalışmaya devam ediyor.

## 10. Riskler ve Açık Konular

| Konu | Not |
|---|---|
| **Modül modu (açık karar)** | İkili dağıtım `external` ve `terminal` modlarına uyuyor; `module` modu kodun kabuğa derlenmesini gerektirir. Seçenekler: modülleri ilk kapsamdan çıkarmak ya da plugin crate'lerini tag'e sabitlenmiş git bağımlılığıyla almak. Faz 5'ten önce karar verilecek. |
| İlk release'ler | Workflow'lar `actionlint`/`shellcheck`'ten geçti; easy-git ve java-atlas'ın CI adımları ile easy-git'in Linux derleme + paketleme adımı yerelde denendi. Windows, macOS ve DockerCity bacakları ilk gerçek tag'de doğrulanacak. |
| macOS imzasız | `.app` imzasız; kabuk dosyayı kendisi indirdiği için Gatekeeper karantinası genelde düşmez, ilk denemede doğrulanacak. Yalnızca Apple Silicon derleniyor. |
| Sürüm hizası | Kabuk ve modül araçları aynı Tauri/Svelte sürümlerinde olmalı; Faz 0'da sabitlenecek. |
| easy-git platformu | Release workflow'u platform başına `--bundles` veriyor; Linux derlemesi ve AppImage paketleme sorunsuz. Linux'ta çalışma zamanı henüz denenmedi. |
| DockerCity | Release'te `PublishTrimmed=false` ile self-contained yayınlanıyor (EF Core/WinUI trimming'e uygun değil). ARM64 derlemesi x64 runner'da çapraz derleniyor; ilk tag'de doğrulanacak. |
| GitDash | `build.zig` yok, `zig build-exe` ile derleniyor; Windows'ta ANSI renk desteği terminalde test edilecek. |
| Docky | `dotnet tool install` mı, `dotnet run` mı ile çağrılacak? İlk sürüm `dotnet run`. |
| Güvenlik | Manifestler komut çalıştırıyor; yalnızca yerel `catalog/` klasörü okunur, Tauri capability'leri dar tutulur. |
