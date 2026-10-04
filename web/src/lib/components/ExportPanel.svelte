<script lang="ts">
  import { api } from '../api';
  import { buildPackageRequest } from '../export-state';
  import type { PackageRenderResponse, TemplateData } from '../types';

  let {
    templateId,
    template,
    dirty,
    variables,
    selectedFormats,
    allFormats,
    history=[],
    onPackage
  }:{
    templateId:string;
    template:TemplateData;
    dirty:boolean;
    variables:Record<string,string>;
    selectedFormats:string[];
    allFormats:string[];
    history?:PackageRenderResponse[];
    onPackage:(response:PackageRenderResponse)=>void;
  }=$props();

  let busy=$state(false);
  let error=$state('');
  let latest=$state<PackageRenderResponse|null>(null);

  async function generate(formats:string[]) {
    if(!formats.length)return;
    busy=true;error='';
    try {
      const response=await api.renderPackage(buildPackageRequest({
        templateId,template,dirty,variables,selectedFormats:formats
      }));
      latest=response;onPackage(response);
    } catch(e){error=e instanceof Error?e.message:String(e)}
    finally{busy=false}
  }
</script>

<section class="page-section exports-page">
  <header class="section-head"><div><span class="eyebrow">EXPORTS / 导出</span><h1>Package multi-format / 多格式导出包</h1><p>Rends les formats choisis et récupère chaque image ou un ZIP avec manifest JSON.</p></div><div class="row"><button class="btn" disabled={busy||!selectedFormats.length} onclick={()=>generate(selectedFormats)}>Sélection / 所选</button><button class="btn primary" disabled={busy||!allFormats.length} onclick={()=>generate(allFormats)}>{busy?'Rendu…':'Tout exporter / 全部导出'}</button></div></header>
  {#if error}<div class="global-error">{error}</div>{/if}
  {#if latest}
    <section class="panel export-latest">
      <div class="panel-head"><div><strong>Dernier package / 最新导出</strong><span>{latest.manifest.generated_at}</span></div><a class="btn primary compact" href={latest.package_url}>ZIP ↓</a></div>
      <div class="export-assets">{#each latest.assets as asset}<a class="export-asset" href={asset.url} target="_blank" rel="noreferrer"><span>{asset.variant}</span><strong>{asset.width}×{asset.height}</strong><small>{asset.filename}</small></a>{/each}</div>
      <details class="manifest-block"><summary>manifest.json</summary><pre>{JSON.stringify(latest.manifest,null,2)}</pre></details>
    </section>
  {/if}
  <section class="export-history">
    <h2>Historique de session / 会话历史</h2>
    {#if history.length}
      {#each history as item (item.package_filename)}
        <article class="history-row panel"><div><strong>{item.template}</strong><span>{item.assets.length} formats · {item.manifest.generated_at}</span></div><a class="btn compact" href={item.package_url}>ZIP ↓</a></article>
      {/each}
    {:else}<p class="muted-copy">Aucun export dans cette session / 本次会话暂无导出</p>{/if}
  </section>
</section>
