# Katalog

Kabuğun gösterdiği her araç bu klasörde kendi alt klasöründe bir `tool.toml` ile tanımlanır. Manifest aracın kodunu içermez; aracın **nerede durduğunu, ne olduğunu ve nasıl çalıştırılacağını** anlatır.

Araçlar iki yoldan gelir:

- **`[release]`** — aracın kendi GitHub reposundaki release'ler. easy-git, java-atlas ve DockerCity böyle.
- **`[source]` + kök** — henüz release'i olmayan, diskteki bir klasörden çalışan araçlar. GitDash ve Docky şimdilik böyle.

## Release'ler ve `release.json`

Her release, dosyalarının yanında bir `release.json` taşır. Kabuk dosya adlarını tahmin etmez, bu dosyayı okur:

```json
{
  "schema": 1,
  "tool": "easy-git",
  "repo": "buraksenyurt/easy-git",
  "version": "0.1.0",
  "tag": "v0.1.0",
  "commit": "…",
  "created_at": "2026-10-29T14:01:44Z",
  "readme": "README.md",
  "platforms": {
    "linux-x64": {
      "portable": {
        "asset": "easy-git-0.1.0-linux-x64.AppImage",
        "sha256": "…", "size": 82815480,
        "format": "file",
        "entry": "easy-git-0.1.0-linux-x64.AppImage"
      },
      "installers": [
        { "asset": "easy-git-0.1.0-linux-x64.deb", "sha256": "…", "size": 4317198, "format": "deb" }
      ]
    }
  }
}
```

- `platforms` anahtarları: `windows-x64`, `windows-arm64`, `linux-x64`, `macos-arm64`.
- `portable.format`: `zip` | `tar.gz` | `file`. `entry`, arşiv açıldıktan sonra çalıştırılacak dosya.
- Kabuk `release.json`'u `github.com/<repo>/releases/latest/download/release.json` adresinden alır; GitHub API'si (ve onun istek sınırı) kullanılmaz.
- `portable` dosyasını indirir, boyut ve SHA256'yı doğrular, `tools/<id>/<sürüm>/` altına açar. Yarım kalan kurulum kurulu sayılmaz (`.mts-install.json` işareti yalnızca başarılı kurulumda yazılır). `installers` yalnızca bilgi amaçlı.
- README ve ekran görüntüleri (`source.readme`, `source.screenshots`) repodan okunur: kurulu sürümün tag'inden, yoksa son sürümün tag'inden, hiç sürüm yoksa `HEAD`'den. Yerel araçlarda araç klasöründen.

Release'leri üreten workflow ve betikler her aracın reposunda: `.github/workflows/release.yml`, `scripts/release/`.

## Katalog kaynakları

Kabuk manifestleri sırayla üç yerden okur; sonra gelen aynı `id`'yi ezer:

1. **Gömülü** — bu klasör, derleme anında uygulamaya gömülür (`src-tauri/build.rs`). Kurulu bir kabuk repoya ihtiyaç duymaz.
2. **Repo** — yalnızca geliştirme (debug) derlemelerinde, bu klasör çalışma anında doğrudan okunur. Manifest düzenlemek için yeniden derleme gerekmez.
3. **Kullanıcı** — ayar klasöründeki `catalog/<id>/tool.toml` dosyaları. Kendi araçlarını eklemek ya da yerleşik bir aracı geçersiz kılmak için.

Okunamayan ya da kurallara uymayan bir manifest diğerlerini engellemez; kabukta "Katalog sorunları" olarak listelenir.

## Klasörler

| | Windows | Linux | macOS |
|---|---|---|---|
| Ayarlar (`settings.toml`, `catalog/`) | `%APPDATA%\dev.buraksenyurt.mytoolset` | `~/.config/dev.buraksenyurt.mytoolset` | `~/Library/Application Support/dev.buraksenyurt.mytoolset` |
| Kurulu araçlar (`tools/`), önbellek (`cache/`) | `%LOCALAPPDATA%\dev.buraksenyurt.mytoolset` | `~/.local/share/dev.buraksenyurt.mytoolset` | `~/Library/Application Support/dev.buraksenyurt.mytoolset` |

## Kök yolları

Release'i olmayan araçlar için manifestler mutlak yol içermez. Her araç bir **kök** adına (`root`) ve o köke göre göreli bir yola (`path`) bağlanır. Kökün diskteki karşılığı `settings.toml`'da tutulur; dosya ilk açılışta şablon olarak oluşturulur:

```toml
[roots]
coding-vibes = "C:/Users/burak/Development/coding-vibes"
```

Dosyayı düzenledikten sonra kabukta "Kataloğu yeniden yükle" yeterli.

## Şema (v0 taslağı)

```toml
schema = 0

id       = "easy-git"            # benzersiz, kebab-case
name     = "easy-git"            # görünen ad
summary  = "Tek satırlık tanım"
category = "git"                 # git | java | docker | ...
kind     = "gui"                 # gui | cli
stack    = ["Rust", "Tauri 2"]
tags     = ["görselleştirme"]

[release]                        # ya release ...
repo = "buraksenyurt/easy-git"

[source]                         # ... ya da kök + yol (release yoksa)
root   = "coding-vibes"          # settings.toml [roots] içindeki ad
path   = "GitDash"               # köke göre göreli klasör
readme = "README.md"             # klasöre (ya da release tag'ine) göre göreli
screenshots = ["docs/images/overview.png"]

[platforms]                      # destek durumu: "yes" | "untested" | "no"
windows = "yes"
linux   = "untested"
macos   = "untested"

[build]                          # yalnızca [source] araçları: ikili yoksa kabuk bu komutu önerir
command = "npm run tauri build"

[run]
host = "external"                # external | terminal | module
# [source] araçları: platforma göre çalıştırılabilir dosya (klasöre göre göreli) ...
# ([release] araçlarında bu bilgi release.json'daki "entry" alanından gelir)
exe.windows = "target/release/easy-git.exe"
exe.linux   = "target/release/easy-git"
# ... ya da geliştirme komutu
dev = "npm run tauri dev"
args = []                        # cli için varsayılan argümanlar
# cli araçları için kabuğun soracağı parametreler
# [[run.params]]
# name = "repo"; kind = "folder"; label = "Repo klasörü"
```

Şema Faz 1'de `crates/mts-core` içinde Rust tiplerine dönüşecek ve doğrulanacak; alanlar o fazda netleşir.
