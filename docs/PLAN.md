# my-tool-set — Proje Plan Dokümanı

> **Atölye:** Fikirler Atölyesi
> **Doküman sürümü:** v0.5 — 29 Ekim 2026
> **Kod:** `my-tool-set` (araçlar kendi repolarında, kabuk onları GitHub Releases'tan alır)
> **Durum:** Faz 0–1 tamamlandı — sıradaki: Faz 2 (kabuk ve bilgi görünümü)

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
- Kabuk araçları **değiştirmez**. MVP'de her araç dış süreç ya da terminal olarak çalışır; modül entegrasyonu (Faz 5–6) MVP sonrasına bırakıldı.
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
| `external` | Araç ayrı bir süreç ve pencere olarak açılır. Kabuk PID, durum (çalışıyor / çıktı kodu), stdout/stderr loglarını izler; durdurabilir. | DockerCity, easy-git, java-atlas (MVP) |
| `terminal` | Araç kabuğun içindeki bir terminal sekmesinde (pty + xterm.js) çalışır. Manifestteki `run.params` ile küçük bir parametre formu çıkar. | GitDash, Docky |
| `module` *(MVP sonrası)* | Araç bir **Tauri plugin**'i olarak kabuğa derlenir; arayüzü kabuğun içinde kendi sekmesinde açılır. | Aday: easy-git (Faz 5), java-atlas (Faz 6) |

**Karar (29 Ekim 2026):** MVP'de modül yok (seçenek A). Tauri plugin + tag'e sabitlenmiş git bağımlılığı (seçenek B) MVP sonrasında değerlendirilecek. Pencere konumlandırarak gömme taklidi (seçenek C) kırılgan olduğu için reddedildi.

B'ye geçilirse modül yaklaşımı: aracın `src-tauri` içindeki komutları bir `tauri-plugin-<araç>` crate'ine taşınır (komutlar `plugin:<araç>|<komut>` ad alanına girer, çakışma olmaz). Svelte arayüzü kök bileşeni dışa açan bir pakete dönüşür. Araç kendi başına çalışmaya devam eder: kendi `src-tauri`'si yalnızca bu plugin'i kaydeden ince bir kabuk olur.

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
│  ├─ mts-core/             domain: Tool, Catalog, HostMode, Platform, doğrulama (altyapıdan habersiz)  ✅ Faz 0
│  ├─ mts-catalog/          manifest okuma, kök çözümleme, release.json okuma, indirme + SHA256 doğrulama  ✅ Faz 1
│  └─ mts-runner/           süreç başlatma/izleme, pty oturumları  (Faz 3)
├─ src-tauri/               Tauri kabuğu: komutlar, state, plugin kayıtları
├─ src/                     Svelte 5 ön yüz
│  ├─ shell/                kenar çubuğu, sekme çubuğu, çalışma alanı, durum çubuğu
│  ├─ views/                araç kartı, README görünümü, log paneli, terminal
│  └─ modules/              (MVP sonrası) modül araçlarının sekme bağlayıcıları
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

### Faz 0 — Atölye kurulumu ✅
- Tauri 2.12 + Svelte 5 + Vite 8 iskeleti; Rust workspace: `mts-core` + `src-tauri`. Diğer crate'ler kendi fazlarında eklenir (boş crate tutmamak için).
- `mts-core`: `Platform` (`Os` × `Arch`), derlendiği platformu bulma, `release.json` anahtarlarıyla (`windows-x64` …) çift yönlü dönüşüm.
- `src-tauri`: `app_info` komutu (sürüm, platform, veri klasörü), ortak `AppError`, ts-rs ile üretilen tipler (`src/lib/bindings`), izinler `core:default` + `core:window:allow-set-theme`.
- Ön yüz: `api.ts` tek IPC noktası (tarayıcıda demo veri), kenar çubuğu (boş katalog durumu, tema düğmesi, Hakkında), sekme çubuğu (Ana sayfa sabit; Ctrl+Tab / Ctrl+Shift+Tab / Ctrl+W, orta tıkla kapatma), durum çubuğu, Ana sayfa ve Hakkında görünümleri. Açık/koyu/sistem teması, yerel tercih olarak saklanır; yerel başlık çubuğu da temaya uyar.
- Uygulama ikonu: `src-tauri/icons-source.svg` → `npx tauri icon`.
- `ci.yml` (svelte-check, build, rustfmt, clippy `-D warnings`, testler, bindings güncel mi) ve `release.yml` (araç repolarındaki şablon, `TOOL=my-tool-set`).
- CSP şimdilik `null` (easy-git/java-atlas gibi). README'ler ağdan gelmeye başladığında (Faz 2) sıkılaştırılacak.
- Doğrulama: Linux'ta `tauri build` ve Xvfb altında uygulama açılıp `app_info` verisinin arayüze geldiği görüldü. Windows'ta ilk `npm run tauri dev` ile denenecek.

### Faz 1 — Katalog ve release'ler ✅
- **Katalog kaynağı (karar):** gömülü + yerel ek. `catalog/` derleme anında gömülür (`build.rs`), debug derlemelerinde repodaki `catalog/` doğrudan okunur, ayar klasöründeki `catalog/<id>/tool.toml` aynı id'yi ezer. Bozuk manifest diğerlerini engellemez, "Katalog sorunları" sekmesinde listelenir.
- **mts-core:** `ToolManifest` (serde, `deny_unknown_fields`) + `validate()` kuralları (kebab-case id, `owner/name` repo, release ile yerel alanların karışmaması, güvenli göreli yollar, parametre varsayılanlarının türü, `module` reddi); `ReleaseManifest` + `validate()` (şema, araç/repo eşleşmesi, semver, tag = v+sürüm, dosya adı, SHA256 biçimi); `Version` (semver).
- **mts-catalog:** `catalog`, `settings` (kökler, `LocalStatus`: hazır / derlenmemiş / kaynaktan / kök yok / klasör yok), `releases` (GitHub API'siz: `releases/latest/download/release.json`, akışlı indirme + SHA256, söz verilen boyutu aşan indirmeyi keser), `cache` (son release.json, çevrimdışı açılış), `install` (`.partial-<sürüm>` → doğrulama → açma → `.mts-install.json` işareti → yerine taşıma; zip-slip ve tar dışına yazma reddi; tek dosyaya çalıştırma izni; Windows'ta antivirüs kilidine karşı yeniden denenen taşıma; idempotent; kaldırma).
- **TLS:** reqwest + rustls `ring` sağlayıcısıyla; `aws-lc-rs` yok (Windows'ta ek derleme aracı istemesin). Sertifikalar işletim sisteminin deposundan doğrulanır.
- **Kabuk:** `list_tools`, `reload_catalog`, `check_updates` (paralel; biri düşerse diğerleri sürer), `install_tool` (channel ile ilerleme, aynı araç için çift kurulum engeli), `uninstall_tool`. Açılışta önbellekteki bilgi hemen gösterilir, ardından güncellemeler denetlenir. İlk açılışta `settings.toml` şablonu yazılır.
- **Ön yüz:** kenar çubuğunda kategorili araç listesi + arama (Faz 2'den öne alındı), durum rozetleri, araç sekmesi (sürüm, kur/güncelle + ilerleme, kurulu sürümler ve iki adımlı kaldırma; yerel araçlarda ayar ve derleme yönergesi), katalog sorunları sekmesi, durum çubuğunda kurulum/denetim/güncelleme bilgisi.
- **Doğrulama:** 69 test (zip-slip, kurcalanmış dosya, yarım indirme, 404, boyut aşımı dahil). Linux'ta debug derlemesi `MTS_RELEASE_BASE` ile yerel bir sunucuya bağlanıp gerçek easy-git AppImage'ını (79 MB) indirdi, doğruladı ve kurdu; `settings.toml` + yeniden yükleme ile GitDash "hazır" durumuna geçti.
- **Bekleyen:** easy-git, java-atlas ve DockerCity için henüz yayınlanmış release yok (`latest/download/release.json` 404). Kabuk bunları "Sürüm yok" diye gösteriyor; ilk tag'lerden sonra gerçek GitHub ile denenecek.

### Faz 2 — Kabuk ve bilgi görünümü
- ~~Kenar çubuğunda kategorili araç listesi, arama~~ (Faz 1'de yapıldı).
- Araç sekmesinin zenginleştirilmesi (Faz 1'de temel hali var).
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
- **MVP burada tamamlanır.**

### Faz 5 — İlk modül: easy-git (`module`) *(MVP sonrası, seçenek B)*
- Önce MVP deneyimine bakılarak B'ye geçmeye değip değmeyeceğine karar verilir.
- easy-git `src-tauri` komutlarının `tauri-plugin-easy-git` crate'ine taşınması (easy-git tek başına çalışmaya devam eder).
- Svelte arayüzünün kök bileşeni dışa açan pakete dönüşmesi (büyük olasılıkla GitHub Packages); kabukta sekmede açılması.
- Bağımlılık stratejisi: tag'e sabitlenmiş git bağımlılığı (`git = "https://github.com/buraksenyurt/easy-git", tag = "v…"`); yerel klasör bağımlılığı yok.

### Faz 6 — İkinci modül: java-atlas *(MVP sonrası)*
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

## 9. MVP Kabul Kriterleri (Faz 0–4)

- [ ] Kabuk Windows'ta açılıyor, Linux'ta en az katalog ve terminal çalışıyor.
- [ ] Beş aracın tamamı kenar çubuğunda kategorili listeleniyor; README'leri görselleriyle okunabiliyor.
- [ ] easy-git, java-atlas ve DockerCity release'ten indirilip SHA256 ile doğrulanarak kuruluyor; yeni sürüm olduğunda kabuk bunu gösteriyor.
- [ ] DockerCity, easy-git, java-atlas dış süreç olarak başlatılıp durdurulabiliyor; loglar görünüyor.
- [ ] GitDash ve Docky gömülü terminalde parametre formuyla çalışıyor.

## 10. Riskler ve Açık Konular

| Konu | Not |
|---|---|
| Modül modu (MVP sonrası) | B seçilirse kabuk araç sürümüne bağlanır (araç her güncellendiğinde kabuk da yeniden derlenir), Svelte arayüzü paket olarak yayınlanmalı, Tauri/Svelte sürümleri hizalı kalmalı. |
| İlk release'ler | Üç reponun CI'ı yeşil. Release'lerin tüm platform bacakları ilk tag'lerde doğrulanacak. |
| macOS imzasız | `.app` imzasız; kabuk dosyayı kendisi indirdiği için Gatekeeper karantinası genelde düşmez, ilk denemede doğrulanacak. Yalnızca Apple Silicon derleniyor. |
| Sürüm hizası | MVP'de gerekmiyor; modül modu gelirse kabuk ve araçlar aynı Tauri/Svelte sürümlerinde olmalı. |
| easy-git platformu | Release workflow'u platform başına `--bundles` veriyor; Linux derlemesi ve AppImage paketleme sorunsuz. Linux'ta çalışma zamanı henüz denenmedi. |
| DockerCity | Release'te `PublishTrimmed=false` ile self-contained yayınlanıyor (EF Core/WinUI trimming'e uygun değil). ARM64 derlemesi x64 runner'da çapraz derleniyor; ilk tag'de doğrulanacak. |
| GitDash | `build.zig` yok, `zig build-exe` ile derleniyor; Windows'ta ANSI renk desteği terminalde test edilecek. |
| Docky | `dotnet tool install` mı, `dotnet run` mı ile çağrılacak? İlk sürüm `dotnet run`. |
| Güvenlik | Manifestler komut çalıştırıyor; yalnızca yerel `catalog/` klasörü okunur, Tauri capability'leri dar tutulur. |
