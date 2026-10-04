<script lang="ts">
  import { api } from '../api';
  import { cloneTemplateForId } from '../template-state';
  import { ui, type Locale } from '../i18n';

  let { templates = [], currentTemplate = '', locale = 'fr', onOpen, onCreated }:{
    templates?:string[]; currentTemplate?:string; locale?:Locale;
    onOpen:(id:string)=>void|Promise<void>; onCreated:(id:string)=>void|Promise<void>;
  } = $props();

  let newId = $state('');
  let busy = $state(false);
  let error = $state('');
  let status = $state('');

  async function createFrom(sourceId:string) {
    const id = newId.trim();
    if (!id) { error = ui(locale,'Renseigne un identifiant.','Enter an ID.','请输入模板 ID。'); return; }
    busy = true; error = ''; status = '';
    try {
      const source = await api.getTemplate(sourceId);
      const clone = cloneTemplateForId(source,id);
      await api.putTemplate(id,clone);
      status = ui(locale,`Modèle créé : ${id}`,`Template created: ${id}`,`模板已创建：${id}`);
      newId = '';
      await onCreated(id);
    } catch (e) { error = e instanceof Error ? e.message : String(e); }
    finally { busy = false; }
  }
</script>

<section class="page-section templates-page">
  <header class="section-head">
    <div><span class="eyebrow">{ui(locale,'MODÈLES','TEMPLATES','模板')}</span><h1>{ui(locale,'Bibliothèque de modèles','Template library','模板库')}</h1><p>{ui(locale,'Crée, ouvre ou duplique un modèle sans toucher aux modèles de production existants.','Create, open or duplicate a template without altering existing production templates.','创建、打开或复制模板，不修改现有生产模板。')}</p></div>
    <span class="status-pill">{templates.length} {ui(locale,'modèles','templates','模板')}</span>
  </header>

  <section class="panel template-create-panel">
    <div class="template-create-copy"><strong>{ui(locale,'Nouveau modèle','New template','新建模板')}</strong><span>{ui(locale,'Identifiant API : lettres, chiffres, tiret ou underscore.','API ID: letters, numbers, hyphen or underscore.','API 标识：字母、数字、连字符或下划线。')}</span></div>
    <input class="input mono" bind:value={newId} placeholder="my-brand" />
    <button class="btn" disabled={busy || !currentTemplate} onclick={()=>void createFrom(currentTemplate)}>{ui(locale,'Dupliquer l’actuel','Duplicate current','复制当前')}</button>
    <button class="btn primary" disabled={busy || !templates.includes('starter-brand')} onclick={()=>void createFrom('starter-brand')}>{ui(locale,'Depuis Starter Brand','From Starter Brand','从 Starter Brand')}</button>
  </section>

  {#if error}<div class="global-error">{error}</div>{/if}
  {#if status}<div class="template-status">{status}</div>{/if}

  <div class="template-grid">
    {#each templates as id (id)}
      <article class="template-card panel" class:starter={id==='starter-brand'} class:current={id===currentTemplate}>
        <div class="template-card-mark">{#if id==='starter-brand'}<img src="/favicon.svg" alt="" />{:else}<span>{id.slice(0,2).toUpperCase()}</span>{/if}</div>
        <div class="template-card-body">
          <div class="template-title-row"><strong>{id}</strong>{#if id==='starter-brand'}<span class="template-badge">{ui(locale,'Démarrage','Starter','起始')}</span>{/if}{#if id===currentTemplate}<span class="template-badge current-badge">{ui(locale,'Actuel','Current','当前')}</span>{/if}</div>
          <p>{id==='starter-brand' ? ui(locale,'Base générique multi-format pour une marque ou une campagne.','Generic multi-format starting point for a brand or campaign.','适用于品牌或营销活动的通用多格式起始模板。') : ui(locale,'Modèle JSON réutilisable et pilotable par API.','Reusable JSON template driven by the API.','可复用并可通过 API 驱动的 JSON 模板。')}</p>
        </div>
        <button class="btn compact" onclick={()=>void onOpen(id)}>{ui(locale,'Ouvrir','Open','打开')}</button>
      </article>
    {/each}
  </div>
</section>
