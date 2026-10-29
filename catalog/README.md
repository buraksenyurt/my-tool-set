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
- Kabuk `portable` dosyasını indirir, SHA256'yı doğrular, `<veri-klasörü>/tools/<id>/<sürüm>/` altına açar. `installers` yalnızca bilgi amaçlı.
- README ve ekran görüntüleri, release tag'indeki repo içeriğinden okunur.

Release'leri üreten workflow ve betikler her aracın reposunda: `.github/workflows/release.yml`, `scripts/release/`.

## Kök yolları

Release'i olmayan araçlar için manifestler mutlak yol içermez. Her araç bir **kök** adına (`root`) ve o köke göre göreli bir yola (`path`) bağlanır. Kökün diskteki karşılığı makineye özgü ayar dosyasında tutulur:

```toml
# %APPDATA%/my-tool-set/settings.toml  (Linux: ~/.config/my-tool-set, macOS: ~/Library/Application Support/my-tool-set)
[roots]
coding-vibes = "C:/Users/burak/Development/coding-vibes"
```

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
