<script lang="ts">
  import { api } from '../api';
  import { tr, type Locale } from '../i18n';
  import type { Point, UploadedAsset } from '../types';

  let {
    locale,
    asset = null,
    focal = {x:0.5,y:0.5},
    busy = false,
    onUploaded,
    onSmartReframe,
    onReset,
    onFocalChange
  }: {
    locale:Locale;
    asset?:UploadedAsset|null;
    focal?:Point;
    busy?:boolean;
    onUploaded:(asset:UploadedAsset)=>void|Promise<void>;
    onSmartReframe:()=>void|Promise<void>;
    onReset:()=>void|Promise<void>;
    onFocalChange:(point:Point)=>void|Promise<void>;
  } = $props();

  let uploading = $state(false);
  let error = $state('');
  let fileInput = $state<HTMLInputElement>();

  async function handleFile(file:File) {
    error = '';
    if (!['image/jpeg','image/png','image/webp'].includes(file.type)) {
      error = 'JPEG, PNG ou WebP uniquement / 仅支持 JPEG、PNG 或 WebP';
      return;
    }
    uploading = true;
    try {
      const uploaded = await api.uploadAsset(file);
      await onUploaded(uploaded);
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      uploading = false;
    }
  }

  function choose(event:Event) {
    const input = event.currentTarget as HTMLInputElement;
    const file = input.files?.[0];
    if (file) void handleFile(file);
    input.value = '';
  }

  function drop(event:DragEvent) {
    event.preventDefault();
    const file = event.dataTransfer?.files?.[0];
    if (file) void handleFile(file);
  }

  function setFocal(event:MouseEvent) {
    if (!asset) return;
    const el = event.currentTarget as HTMLElement;
    const rect = el.getBoundingClientRect();
    const x = Math.max(0,Math.min(1,(event.clientX-rect.left)/rect.width));
    const y = Math.max(0,Math.min(1,(event.clientY-rect.top)/rect.height));
    void onFocalChange({x,y});
  }
</script>

<section class="source-panel panel">
  <div class="source-panel-head">
    <div>
      <span class="eyebrow">SOURCE / 原图</span>
      <strong>{tr('fr','label.sourceImage')} / {tr('zh-CN','label.sourceImage')}</strong>
    </div>
    {#if asset}<span class="asset-size">{asset.width}×{asset.height}</span>{/if}
  </div>

  <input bind:this={fileInput} class="sr-only" type="file" accept="image/jpeg,image/png,image/webp" onchange={choose} />

  {#if asset}
    <button class="source-preview" onclick={setFocal} title="Déplacer le point focal / 调整焦点">
      <img src={asset.url} alt="Source" />
      <span class="focal-cross" style:left={(focal.x*100)+'%'} style:top={(focal.y*100)+'%'}></span>
      <span class="source-hint">Clique pour déplacer le point focal / 点击调整焦点</span>
    </button>
  {:else}
    <button class="source-dropzone" ondragover={(e)=>e.preventDefault()} ondrop={drop} onclick={() => fileInput?.click()}>
      <img src="/favicon.svg" alt="" />
      <strong>{uploading ? 'Import… / 上传中…' : 'Glisser une image / 拖入图片'}</strong>
      <span>JPEG · PNG · WebP</span>
    </button>
  {/if}

  <div class="source-actions">
    <button class="btn compact" onclick={() => fileInput?.click()} disabled={uploading}>
      {asset ? tr('fr','action.replaceImage') + ' / ' + tr('zh-CN','action.replaceImage') : tr('fr','action.import') + ' / ' + tr('zh-CN','action.import')}
    </button>
    <button class="btn compact primary" onclick={() => void onSmartReframe()} disabled={!asset || busy}>
      {busy ? 'Analyse… / 分析中…' : tr('fr','action.smartCrop') + ' / ' + tr('zh-CN','action.smartCrop')}
    </button>
    <button class="btn compact" onclick={() => void onReset()} disabled={!asset || busy}>
      Auto
    </button>
  </div>
  {#if error}<div class="inline-error">{error}</div>{/if}
</section>
