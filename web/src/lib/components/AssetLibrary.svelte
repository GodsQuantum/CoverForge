<script lang="ts">
  import type { BrandStyle, UploadedAsset } from '../types';
  let { assets=[], brand={}, onBackground, onLogo }:{
    assets?:UploadedAsset[];
    brand?:BrandStyle;
    onBackground:(asset:UploadedAsset)=>void|Promise<void>;
    onLogo:(asset:UploadedAsset)=>void;
  }=$props();
</script>

<section class="page-section asset-library-page">
  <header class="section-head"><div><span class="eyebrow">BIBLIOTHÈQUE / 素材库</span><h1>Assets de la session / 本次会话素材</h1><p>Les imports restent disponibles pendant cette session pour être réaffectés sans nouvel upload.</p></div><span class="status-pill">{assets.length} assets</span></header>
  {#if assets.length}
    <div class="asset-grid">
      {#each assets as asset (asset.id)}
        <article class="asset-card panel">
          <div class="asset-thumb">{#if asset.url}<img src={asset.url} alt={asset.filename} />{/if}</div>
          <div class="asset-meta"><strong>{asset.filename}</strong><span>{asset.width}×{asset.height} · {asset.kind}</span></div>
          <div class="asset-actions"><button class="btn compact" disabled={asset.kind==='logo'} onclick={()=>void onBackground(asset)}>Fond / 背景</button><button class="btn compact" onclick={()=>onLogo(asset)}>Logo</button></div>
        </article>
      {/each}
    </div>
  {:else}
    <div class="empty-library panel"><img src="/favicon.svg" alt="" /><strong>Aucun asset importé / 尚无导入素材</strong><span>Importe une image depuis Projet / 从项目页面导入图片</span></div>
  {/if}
  {#if Object.keys(brand.logos||{}).length}
    <div class="library-brand-refs panel"><strong>Logos du Brand Kit / 品牌 Logo</strong>{#each Object.entries(brand.logos||{}) as [slot,path]}<code>{slot}: {path}</code>{/each}</div>
  {/if}
</section>
