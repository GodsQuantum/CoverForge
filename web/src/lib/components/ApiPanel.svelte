<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '../api';
  import { ui, type Locale } from '../i18n';
  import type { HealthResponse, TemplateDataset } from '../types';

  let { templateId, locale='fr' }:{ templateId:string; locale?:Locale } = $props();

  let health = $state<HealthResponse|null>(null);
  let dataset = $state<TemplateDataset|null>(null);
  let error = $state('');
  let copied = $state('');

  function examples() {
    const id = templateId || 'starter-brand';
    return [
      [ui(locale,'Upload','Upload','上传'), `curl -F "asset=@image.jpg" http://localhost:3099/v1/assets`],
      [ui(locale,'Recadrage intelligent','Smart Reframe','智能裁切'), `curl -X POST http://localhost:3099/v1/reframe -H 'content-type: application/json' -d '{"asset":"/path/image.jpg","formats":[{"id":"square","width":1080,"height":1080}]}'`],
      [ui(locale,'Rendu','Render','渲染'), `curl -X POST http://localhost:3099/v1/render -H 'content-type: application/json' -d '{"template":"${id}","variables":{"title":"Hello"},"variants":["square"]}'`],
      [ui(locale,'Package ZIP','ZIP package','ZIP 打包'), `curl -X POST http://localhost:3099/v1/render/package -H 'content-type: application/json' -d '{"template":"${id}","variants":["youtube","square"]}'`]
    ] as const;
  }

  async function load() {
    error='';
    try {
      health = await api.health();
      dataset = templateId ? await api.getDataset(templateId) : null;
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  async function copy(label:string,value:string) {
    try {
      await navigator.clipboard.writeText(value);
      copied=label;
      window.setTimeout(()=>{ if(copied===label) copied=''; },1200);
    } catch {
      copied='';
    }
  }

  onMount(()=>{ void load(); });
  $effect(() => { if (templateId) void load(); });
</script>

<section class="page-section api-page">
  <header class="section-head">
    <div>
      <span class="eyebrow">API</span>
      <h1>{ui(locale,'Automatisation API-first','API-first automation','API 优先自动化')}</h1>
      <p>{ui(locale,'Pilote les modèles, variables, recadrages et exports sans passer par l’interface.','Drive templates, variables, reframing and exports without using the UI.','无需界面即可驱动模板、变量、智能裁切与导出。')}</p>
    </div>
    {#if health}<span class="status-pill">v{health.version} · {health.renderer || 'Rust'}</span>{/if}
  </header>

  {#if error}<div class="global-error">{error}</div>{/if}

  <div class="api-grid">
    <section class="panel api-card">
      <div class="panel-head"><div><strong>OpenAPI 3.1</strong><span>{ui(locale,'Découverte','Discovery','API 发现')}</span></div><a class="btn compact" href="/openapi.json" target="_blank" rel="noreferrer">openapi.json ↗</a></div>
      <div class="api-card-body"><p>{ui(locale,'Schéma vivant de l’instance.','Live schema for this instance.','当前实例的实时 API 规范。')}</p><code>GET /openapi.json</code></div>
    </section>

    <section class="panel api-card">
      <div class="panel-head"><div><strong>{ui(locale,'Dataset','Dataset','自动填充字段')}</strong><span>{templateId}</span></div></div>
      <div class="api-card-body">
        {#if dataset}
          <div class="dataset-formats">{#each dataset.formats as format}<span>{format.id} · {format.width}×{format.height}</span>{/each}</div>
          <div class="dataset-fields">{#each dataset.fields as field}<div><strong>{field.key}</strong><span>{field.types.join(' + ')}</span><small>{field.usages.join(', ')}</small></div>{/each}</div>
        {:else}<p>{ui(locale,'Aucun dataset','No dataset','暂无数据集')}</p>{/if}
      </div>
    </section>
  </div>

  <section class="api-examples">
    <h2>{ui(locale,'Exemples','Examples','示例')}</h2>
    {#each examples() as [label,command]}
      <article class="panel api-example">
        <div class="api-example-head"><strong>{label}</strong><button class="btn compact" onclick={()=>void copy(label,command)}>{copied===label?ui(locale,'Copié ✓','Copied ✓','已复制 ✓'):ui(locale,'Copier','Copy','复制')}</button></div>
        <pre>{command}</pre>
      </article>
    {/each}
  </section>
</section>
