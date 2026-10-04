<script lang="ts">
  import type { Snippet } from 'svelte';
  import { tr, ui, type Locale, type TranslationKey } from '../i18n';
  import type { ProductView, View } from '../types';

  let {
    activeView,
    locale,
    dirty = false,
    onNavigate,
    onLocale,
    children
  }: {
    activeView:View;
    locale:Locale;
    dirty?:boolean;
    onNavigate:(view:ProductView)=>void|Promise<void>;
    onLocale:(locale:Locale)=>void;
    children:Snippet;
  } = $props();

  const items:{view:ProductView; icon:string; key:TranslationKey}[] = [
    {view:'project', icon:'✦', key:'nav.project'},
    {view:'templates', icon:'▦', key:'nav.templates'},
    {view:'library', icon:'◫', key:'nav.library'},
    {view:'brand', icon:'◆', key:'nav.brand'},
    {view:'exports', icon:'⇩', key:'nav.exports'},
    {view:'api', icon:'⌘', key:'nav.api'}
  ];

  function active(view:ProductView) {
    return activeView === view || (view === 'project' && ['composer','fonts','json'].includes(activeView));
  }
</script>

<div class="app-shell">
  <aside class="app-rail">
    <button class="brand-lockup brand-home" onclick={() => onNavigate('project')} aria-label="CoverForge">
      <img class="brand-logo" src="/coverforge-logo.svg" alt="CoverForge" />
      <span class="brand-sub">{ui(locale,'Automatisation de marque','Branding Automation','品牌自动化')}</span>
    </button>

    <nav class="nav-list" aria-label="CoverForge">
      {#each items as item}
        <button class:active={active(item.view)} class="nav-button" onclick={() => onNavigate(item.view)}>
          <span class="nav-icon">{item.icon}</span>
          <span class="nav-label">{tr(locale,item.key)}</span>
        </button>
      {/each}
    </nav>

    <div class="rail-spacer"></div>

    <div class="locale-switch" aria-label={ui(locale,'Langue','Language','语言')}>
      <button class:active={locale==='fr'} onclick={() => onLocale('fr')}>FR</button>
      <button class:active={locale==='en'} onclick={() => onLocale('en')}>EN</button>
      <button class:active={locale==='zh-CN'} onclick={() => onLocale('zh-CN')}>ZH</button>
    </div>

    <div class="rail-note">
      <strong class:unsaved={dirty}>{dirty ? tr(locale,'status.unsaved') : tr(locale,'status.synced')}</strong>
      <span>Ctrl+S · Ctrl+Z · Ctrl+D</span>
    </div>
  </aside>

  <div class="app-main-shell">
    {@render children()}
  </div>
</div>
