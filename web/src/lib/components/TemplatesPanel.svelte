<script lang="ts">
  import { api } from '../api';
  import { cloneTemplateForId } from '../template-state';
  import type { Locale } from '../i18n';

  let {
    templates = [],
    currentTemplate = '',
    locale = 'fr',
    onOpen,
    onCreated
  }:{
    templates?:string[];
    currentTemplate?:string;
    locale?:Locale;
    onOpen:(id:string)=>void|Promise<void>;
    onCreated:(id:string)=>void|Promise<void>;
  } = $props();

  let newId = $state('');
  let busy = $state(false);
  let error = $state('');
  let status = $state('');

  const copy = () => locale === 'fr'
    ? 'Crée, ouvre ou duplique un modèle sans toucher aux modèles de production existants.'
    : '创建、打开或复制模板，不修改现有生产模板。';

  async function createFrom(sourceId:string) {
    const id = newId.trim();
    if (!id) { error = locale === 'fr' ? 'Renseigne un identifiant.' : '请输入模板 ID。'; return; }
    busy = true; error = ''; status = '';
    try {
      const source = await api.getTemplate(sourceId);
      const clone = cloneTemplateForId(source,id);
      await api.putTemplate(id,clone);
      status = locale === 'fr' ? `Modèle créé : ${id}` : `模板已创建：${id}`;
      newId = '';
      await onCreated(id);
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      busy = false;
    }
  }
</script>

<section class="page-section templates-page">
  <header class="section-head">
    <div>
      <span class="eyebrow">MODÈLES / 模板</span>
      <h1>Bibliothèque de modèles / 模板库</h1>
      <p>{copy()}</p>
    </div>
    <span class="status-pill">{templates.length} modèles / 模板</span>
  </header>

  <section class="panel template-create-panel">
    <div class="template-create-copy">
      <strong>Nouveau modèle / 新建模板</strong>
      <span>Identifiant API : lettres, chiffres, tiret ou underscore.</span>
    </div>
    <input class="input mono" bind:value={newId} placeholder="ma-marque / my_brand" />
    <button class="btn" disabled={busy || !currentTemplate} onclick={()=>void createFrom(currentTemplate)}>Dupliquer l’actuel / 复制当前</button>
    <button class="btn primary" disabled={busy || !templates.includes('starter-brand')} onclick={()=>void createFrom('starter-brand')}>Depuis Starter Brand / 从 Starter Brand</button>
  </section>

  {#if error}<div class="global-error">{error}</div>{/if}
  {#if status}<div class="template-status">{status}</div>{/if}

  <div class="template-grid">
    {#each templates as id (id)}
      <article class="template-card panel" class:starter={id==='starter-brand'} class:current={id===currentTemplate}>
        <div class="template-card-mark">
          {#if id==='starter-brand'}<img src="/favicon.svg" alt="" />{:else}<span>{id.slice(0,2).toUpperCase()}</span>{/if}
        </div>
        <div class="template-card-body">
          <div class="template-title-row">
            <strong>{id}</strong>
            {#if id==='starter-brand'}<span class="template-badge">Starter / 起始</span>{/if}
            {#if id===currentTemplate}<span class="template-badge current-badge">Actuel / 当前</span>{/if}
          </div>
          <p>{id==='starter-brand'
            ? (locale==='fr' ? 'Base générique multi-format pour une marque ou une campagne.' : '适用于品牌或营销活动的通用多格式起始模板。')
            : (locale==='fr' ? 'Modèle JSON réutilisable et pilotable par API.' : '可复用并可通过 API 驱动的 JSON 模板。')}</p>
        </div>
        <button class="btn compact" onclick={()=>void onOpen(id)}>Ouvrir / 打开</button>
      </article>
    {/each}
  </div>
</section>
