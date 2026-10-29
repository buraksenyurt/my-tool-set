<script lang="ts">
  // Full-window view of one picture. Esc, a click anywhere or the close button
  // closes it; focus returns to where it was.
  let { src, alt, onclose }: { src: string; alt: string; onclose: () => void } = $props();

  let closeButton: HTMLButtonElement | undefined = $state();

  $effect(() => {
    const previous = document.activeElement as HTMLElement | null;
    closeButton?.focus();
    return () => previous?.focus();
  });

  function onKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      e.preventDefault();
      e.stopPropagation();
      onclose();
    }
  }
</script>

<svelte:window onkeydown={onKeydown} />

<div class="backdrop" role="dialog" aria-modal="true" aria-label={alt || "Görsel"} tabindex="-1" onclick={onclose} onkeydown={() => {}}>
  <img {src} {alt} />
  {#if alt}<p>{alt}</p>{/if}
  <button bind:this={closeButton} aria-label="Kapat" onclick={onclose}>×</button>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    z-index: 100;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    padding: 40px;
    background: rgb(0 0 0 / 0.82);
    cursor: zoom-out;
  }

  img {
    max-width: 100%;
    max-height: calc(100% - 40px);
    object-fit: contain;
    border-radius: 4px;
    box-shadow: 0 8px 40px rgb(0 0 0 / 0.5);
  }

  p {
    margin: 0;
    color: #ddd;
    font-size: 13px;
  }

  button {
    position: absolute;
    top: 14px;
    right: 18px;
    width: 34px;
    height: 34px;
    border: 0;
    border-radius: 50%;
    background: rgb(255 255 255 / 0.12);
    color: #fff;
    font-size: 22px;
    line-height: 1;
    cursor: pointer;
  }

  button:hover {
    background: rgb(255 255 255 / 0.22);
  }
</style>
