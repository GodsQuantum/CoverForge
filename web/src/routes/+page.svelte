<script lang="ts">
  type Asset = { variant:string; width:number; height:number; filename:string; url:string };
  type RenderResponse = { ok:boolean; template:string; assets:Asset[]; error?:string };

  let templates = $state<string[]>([]);
  let template = $state('');
  let title = $state('LE STAND-UP EST MORT');
  let eyebrow = $state('DPAFM #48');
  let background = $state('');
  let logo = $state('');
  let availableVariants = $state<string[]>(['default']);
  let variants = $state(new Set(['default']));
  let rendering = $state(false);
  let error = $state('');
  let assets = $state<Asset[]>([]);
  let templateJson = $state('');
  let templateStatus = $state('');
  let savingTemplate = $state(false);

  async function loadTemplates() {
    try {
      const res = await fetch('/v1/templates');
      const data = await res.json();
      templates = Array.isArray(data.templates) ? data.templates : [];
      if (!template || !templates.includes(template)) template = templates[0] || '';
      if (template) await loadTemplate(template);
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  async function loadTemplate(id:string) {
    templateStatus = '';
    const res = await fetch('/v1/templates/' + encodeURIComponent(id));
    const data = await res.json();
    if (!res.ok) throw new Error(data.error || 'Template load failed');
    templateJson = JSON.stringify(data, null, 2);
    availableVariants = Object.keys(data.variants || {});
    if (!availableVariants.length) availableVariants = ['default'];
    variants = new Set(availableVariants);
  }

  async function onTemplateChange() {
    assets = [];
    error = '';
    try { await loadTemplate(template); }
    catch (e) { error = e instanceof Error ? e.message : String(e); }
  }

  function toggleVariant(name:string) {
    const next = new Set(variants);
    next.has(name) ? next.delete(name) : next.add(name);
    variants = next;
  }

  async function saveTemplate() {
    error = '';
    templateStatus = '';
    savingTemplate = true;
    try {
      const parsed = JSON.parse(templateJson);
      if (parsed.id !== template) throw new Error('Template id must stay "' + template + '"');
      const res = await fetch('/v1/templates/' + encodeURIComponent(template), {
        method: 'PUT',
        headers: {'content-type':'application/json'},
        body: JSON.stringify(parsed)
      });
      const data = await res.json();
      if (!res.ok || !data.ok) throw new Error(data.error || 'Template save failed');
      templateJson = JSON.stringify(parsed, null, 2);
      templateStatus = 'Saved atomically';
      await loadTemplate(template);
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      savingTemplate = false;
    }
  }

  async function render() {
    error = '';
    assets = [];
    rendering = true;
    try {
      const selected = [...variants];
      const res = await fetch('/v1/render', {
        method: 'POST',
        headers: {'content-type':'application/json'},
        body: JSON.stringify({
          template,
          variables: { title, eyebrow, background, logo },
          variants: selected.includes('default') ? [] : selected
        })
      });
      const data:RenderResponse = await res.json();
      if (!res.ok || !data.ok) throw new Error(data.error || 'Render failed');
      assets = data.assets || [];
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      rendering = false;
    }
  }

  loadTemplates();
</script>

<svelte:head>
  <title>CoverForge</title>
  <meta name="description" content="Deterministic social media compositing" />
</svelte:head>

<div class="app-shell">
  <aside class="app-rail">
    <div class="brand-lockup">
      <div class="brand-mark">CF</div>
      <div>
        <div class="brand-title">CoverForge</div>
        <div class="brand-sub">Template studio · API compositor</div>
      </div>
    </div>
    <nav class="nav-list">
      <button class="nav-button active"><span class="nav-icon">◫</span><span class="nav-label">Composer</span></button>
      <button class="nav-button"><span class="nav-icon">▦</span><span class="nav-label">Templates</span></button>
      <button class="nav-button"><span class="nav-icon">⚙</span><span class="nav-label">API</span></button>
    </nav>
    <div class="rail-spacer"></div>
    <div class="rail-note">Rust/resvg · editable JSON templates · deterministic output</div>
  </aside>

  <main class="app-main">
    <section class="page">
      <header class="page-head">
        <div>
          <div class="page-kicker">COVERFORGE 0.2</div>
          <h1 class="page-title">Compose each format deliberately.</h1>
          <p class="page-copy">AUTOPUBLISHER generates native key art per aspect ratio. CoverForge owns exact typography, logos and final brand layout.</p>
        </div>
        <div class="status-pill ready"><span class="status-dot"></span>Rust / resvg</div>
      </header>

      <div class="editor-layout">
        <div class="stack">
          <article class="card">
            <div class="card-header">
              <div><strong>Composition</strong><span class="card-sub">Preview the current production template</span></div>
            </div>
            <div class="card-body stack">
              <div class="field">
                <label for="template">Template</label>
                <select id="template" class="select" bind:value={template} onchange={onTemplateChange}>
                  {#each templates as t}<option value={t}>{t}</option>{/each}
                </select>
              </div>
              <div class="grid two">
                <div class="field">
                  <label for="eyebrow">Eyebrow</label>
                  <input id="eyebrow" class="input" bind:value={eyebrow} />
                </div>
                <div class="field">
                  <label for="title">Title</label>
                  <input id="title" class="input" bind:value={title} />
                </div>
              </div>
              <div class="grid two">
                <div class="field">
                  <label for="background">Native-format key art</label>
                  <input id="background" class="input mono" bind:value={background} placeholder="/srv/storage/.../keyart.png" />
                </div>
                <div class="field">
                  <label for="logo">Logo asset</label>
                  <input id="logo" class="input mono" bind:value={logo} placeholder="/srv/storage/.../logo.png" />
                </div>
              </div>
              <div class="field">
                <span class="field-label">Renditions in this template</span>
                <div class="variant-grid">
                  {#each availableVariants as v}
                    <button type="button" class:active={variants.has(v)} class="variant-card" onclick={() => toggleVariant(v)}>
                      <strong>{v}</strong><span>native scene graph</span>
                    </button>
                  {/each}
                </div>
              </div>
              <button class="btn primary render-button" onclick={render} disabled={rendering || !template || variants.size === 0}>
                {rendering ? 'Rendering…' : 'Render preview'}
              </button>
              {#if error}<div class="error-box">{error}</div>{/if}
            </div>
          </article>

          <article class="card">
            <div class="card-header">
              <div><strong>Template JSON</strong><span class="card-sub">Production file · atomic save · immediately used by AUTOPUBLISHER</span></div>
              {#if templateStatus}<span class="status-pill ready">{templateStatus}</span>{/if}
            </div>
            <div class="card-body stack">
              <textarea class="textarea mono template-editor" bind:value={templateJson} spellcheck="false"></textarea>
              <div class="row between">
                <span class="help">Frames are normalized 0–1. Font sizes are relative to canvas height.</span>
                <button class="btn primary" onclick={saveTemplate} disabled={savingTemplate || !template}>
                  {savingTemplate ? 'Saving…' : 'Save template'}
                </button>
              </div>
            </div>
          </article>
        </div>

        <aside class="card output-card">
          <div class="card-header">
            <div><strong>Render output</strong><span class="card-sub">{assets.length ? assets.length + ' rendition(s)' : 'Waiting for preview'}</span></div>
          </div>
          <div class="card-body output-body">
            {#if rendering}
              <div class="empty-state"><div class="pulse-ring"></div><strong>Rendering locally</strong><span>resvg is composing the selected template.</span></div>
            {:else if assets.length}
              <div class="asset-list">
                {#each assets as asset}
                  <a class="asset-card" href={asset.url} target="_blank" rel="noreferrer">
                    <div class="asset-preview"><img src={asset.url} alt={asset.variant} /></div>
                    <div class="row between"><strong>{asset.variant}</strong><span>{asset.width}×{asset.height}</span></div>
                    <code>{asset.filename}</code>
                  </a>
                {/each}
              </div>
            {:else}
              <div class="empty-state">
                <div class="empty-icon">◫</div>
                <strong>Native key art in. Brand-safe cover out.</strong>
                <span>No blurred fake format conversion. Each background should already be generated for its destination ratio.</span>
              </div>
            {/if}
          </div>
        </aside>
      </div>
    </section>
  </main>
</div>
