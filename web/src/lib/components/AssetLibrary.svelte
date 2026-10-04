<script lang="ts">
  import { ui, type Locale } from '../i18n';
  import type { BrandStyle, UploadedAsset } from '../types';
  let { locale='fr', assets=[], brand={}, onBackground, onLogo }:{
    locale?:Locale;
    assets?:UploadedAsset[];
    brand?:BrandStyle;
    onBackground:(asset:UploadedAsset)=>void|Promise<void>;
    onLogo:(asset:UploadedAsset)=>void;
  }=$props();
</script>

<section class="page-section asset-library-page">
  <header class="section-head"><div><span class="eyebrow">{ui(locale,'BIBLIOTHÈQUE','LIBRARY','素材库')}</span><h1>{ui(locale,'Assets de la session','Session assets','本次会话素材')}</h1><p>{ui(locale,'Les imports restent disponibles pendant cette session pour être réaffectés sans nouvel upload.','Imports stay available during this session so you can reuse them without uploading again.','导入的素材在本次会话中保持可用，无需重新上传即可复用。')}</p></div><span class="status-pill">{assets.length} {ui(locale,'assets','assets','素材')}</span></header>
  {#if assets.length}
    <div class="asset-grid">
      {#each assets as asset (asset.id)}
        <article class="asset-card panel">
          <div class="asset-thumb">{#if asset.url}<img src={asset.url} alt={asset.filename} />{/if}</div>
          <div class="asset-meta"><strong>{asset.filename}</strong><span>{asset.width}×{asset.height} · {asset.kind}</span></div>
          <div class="asset-actions"><button class="btn compact" disabled={asset.kind==='logo'} onclick={()=>void onBackground(asset)}>{ui(locale,'Fond','Background','背景')}</button><button class="btn compact" onclick={()=>onLogo(asset)}>Logo</button></div>
        </article>
      {/each}
    </div>
  {:else}
    <div class="empty-library panel"><img src="/favicon.svg" alt="" /><strong>{ui(locale,'Aucun asset importé','No imported assets','尚无导入素材')}</strong><span>{ui(locale,'Importe une image depuis Projet','Import an image from Project','请从项目页面导入图片')}</span></div>
  {/if}
  {#if Object.keys(brand.logos||{}).length}
    <div class="library-brand-refs panel"><strong>{ui(locale,'Logos du Kit de marque','Brand kit logos','品牌工具包 Logo')}</strong>{#each Object.entries(brand.logos||{}) as [slot,path]}<code>{slot}: {path}</code>{/each}</div>
  {/if}
</section>
