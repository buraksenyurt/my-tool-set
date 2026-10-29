<script lang="ts">
  import { openAbout } from "../tabs.svelte";
</script>

<div class="home">
  <h1>my-tool-set</h1>
  <p class="lead">
    Kendi araçlarımı tek bir kabuktan görmek, okumak, kurmak ve çalıştırmak için.
  </p>

  <ul class="features">
    <li>
      <strong>Katalog</strong>
      <span>Her araç bir <code>tool.toml</code> manifesti ile tanımlanır; ne işe yaradığı ve hangi platformda çalıştığı burada görünür.</span>
    </li>
    <li>
      <strong>Kur ve güncelle</strong>
      <span>Araçlar kendi repolarının GitHub Releases'ından indirilir, SHA256 ile doğrulanır.</span>
    </li>
    <li>
      <strong>Çalıştır ve izle</strong>
      <span>GUI araçlar ayrı pencerede açılır, logları burada izlenir; CLI araçlar gömülü terminal sekmesinde çalışır.</span>
    </li>
  </ul>

  <h2>Kısayollar</h2>
  <dl class="keys">
    <dt><kbd>Ctrl</kbd> + <kbd>Tab</kbd></dt>
    <dd>Sonraki sekme</dd>
    <dt><kbd>Ctrl</kbd> + <kbd>Shift</kbd> + <kbd>Tab</kbd></dt>
    <dd>Önceki sekme</dd>
    <dt><kbd>Ctrl</kbd> + <kbd>W</kbd></dt>
    <dd>Sekmeyi kapat</dd>
  </dl>

  <p class="more"><button class="link" onclick={openAbout}>Sürüm ve platform bilgisi →</button></p>
</div>

<style>
  .home {
    max-width: 720px;
    padding: 40px 48px;
  }

  h1 {
    margin: 0 0 6px;
    font-size: 26px;
    font-weight: 600;
  }

  h2 {
    margin: 32px 0 12px;
    font-size: 13px;
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--muted);
  }

  .lead {
    margin: 0 0 28px;
    color: var(--text-secondary);
    font-size: 15px;
  }

  .features {
    display: grid;
    gap: 10px;
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .features li {
    display: grid;
    gap: 4px;
    padding: 14px 16px;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    background: var(--surface);
  }

  .features span {
    color: var(--text-secondary);
    line-height: 1.45;
  }

  .keys {
    display: grid;
    grid-template-columns: max-content 1fr;
    gap: 8px 18px;
    margin: 0;
  }

  .keys dd {
    margin: 0;
    color: var(--text-secondary);
  }

  .more {
    margin-top: 32px;
  }

  button.link {
    padding: 0;
    border: 0;
    background: none;
    color: var(--accent);
    cursor: pointer;
  }

  button.link:hover {
    text-decoration: underline;
  }
</style>
