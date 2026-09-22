<script lang="ts">
  type Asset = { variant:string; width:number; height:number; filename:string; url:string };
  type RenderResponse = { ok:boolean; template:string; assets:Asset[]; error?:string };
  type FontRecord = { family:string; style:string; filename:string; source:string; url:string };

  let templates = $state<string[]>([]);
  let template = $state('');
  let templateData = $state<any>(null);
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

  let fonts = $state<FontRecord[]>([]);
  let fontQuery = $state('');
  let uploadingFont = $state(false);
  let fontStatus = $state('');
  let fontPreviewNames = $state<Record<string,string>>({});

  function fontKey(f:FontRecord) { return f.family + '|' + f.style + '|' + f.filename; }

  function installFontPreviews() {
    const old = document.getElementById('coverforge-font-previews');
    if (old) old.remove();
    const style = document.createElement('style');
    style.id = 'coverforge-font-previews';
    const map:Record<string,string> = {};
    const rules:string[] = [];
    fonts.forEach((f,i) => {
      if (!f.url) return;
      const name = 'CFPreview' + i;
      map[fontKey(f)] = name;
      rules.push('@font-face{font-family:"' + name + '";src:url("' + f.url + '");font-display:swap;}');
    });
    style.textContent = rules.join('\n');
    document.head.appendChild(style);
    fontPreviewNames = map;
  }

  function previewStyle(f:FontRecord) {
    const family = fontPreviewNames[fontKey(f)] || f.family;
    return 'font-family:"' + family.replaceAll('"','') + '",sans-serif';
  }

  function matchedFonts() {
    const q = fontQuery.trim().toLowerCase();
    if (!q) return fonts;
    return fonts.filter(f => (f.family + ' ' + f.style + ' ' + f.source).toLowerCase().includes(q));
  }

  function filteredFonts() {
    return matchedFonts().slice(0, 180);
  }

  async function loadFonts() {
    const res = await fetch('/v1/fonts');
    const data = await res.json();
    if (!res.ok) throw new Error(data.error || 'Font catalog failed');
    fonts = Array.isArray(data.fonts) ? data.fonts : [];
    installFontPreviews();
  }

  async function uploadFont(event:Event) {
    const input = event.currentTarget as HTMLInputElement;
    const file = input.files?.[0];
    if (!file) return;
    uploadingFont = true;
    fontStatus = '';
    error = '';
    try {
      const body = new FormData();
      body.append('font', file);
      const res = await fetch('/v1/fonts', { method:'POST', body });
      const data = await res.json();
      if (!res.ok || !data.ok) throw new Error(data.error || 'Font upload failed');
      fontStatus = 'Ajoutée : ' + data.family + ' · ' + data.style;
      await loadFonts();
      input.value = '';
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      uploadingFont = false;
    }
  }

  async function deleteFont(f:FontRecord) {
    if (f.source !== 'custom') return;
    error = '';
    fontStatus = '';
    try {
      const res = await fetch('/v1/fonts/' + encodeURIComponent(f.filename), { method:'DELETE' });
      const data = await res.json();
      if (!res.ok || !data.ok) throw new Error(data.error || 'Delete failed');
      fontStatus = 'Supprimée : ' + f.family;
      await loadFonts();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  async function loadTemplates() {
    try {
      const res = await fetch('/v1/templates');
      const data = await res.json();
      templates = Array.isArray(data.templates) ? data.templates : [];
      if (!template || !templates.includes(template)) template = templates[0] || '';
      if (template) await loadTemplate(template);
      await loadFonts();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  async function loadTemplate(id:string) {
    templateStatus = '';
    const res = await fetch('/v1/templates/' + encodeURIComponent(id));
    const data = await res.json();
    if (!res.ok) throw new Error(data.error || 'Show style load failed');
    templateData = data;
    templateJson = JSON.stringify(data, null, 2);
    availableVariants = Object.keys(data.formats || data.variants || {});
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
      if (parsed.id !== template) throw new Error('Show style id must stay "' + template + '"');
      const res = await fetch('/v1/templates/' + encodeURIComponent(template), {
        method: 'PUT',
        headers: {'content-type':'application/json'},
        body: JSON.stringify(parsed)
      });
      const data = await res.json();
      if (!res.ok || !data.ok) throw new Error(data.error || 'Show style save failed');
      templateJson = JSON.stringify(parsed, null, 2);
      templateStatus = 'Sauvegardé atomiquement';
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
        <div class="brand-sub">Show styles · fonts · compositor</div>
      </div>
    </div>
    <nav class="nav-list">
      <button class="nav-button active"><span class="nav-icon">◫</span><span class="nav-label">Composer</span></button>
      <button class="nav-button"><span class="nav-icon">Aa</span><span class="nav-label">Fonts</span></button>
      <button class="nav-button"><span class="nav-icon">JS</span><span class="nav-label">JSON</span></button>
    </nav>
    <div class="rail-spacer"></div>
    <div class="rail-note">1 JSON par émission · layouts natifs par format · fonts custom persistantes</div>
  </aside>

  <main class="app-main">
    <section class="page">
      <header class="page-head">
        <div>
          <div class="page-kicker">COVERFORGE 0.3</div>
          <h1 class="page-title">Un style par émission. Tous les formats dedans.</h1>
          <p class="page-copy">Le key art est généré nativement au bon ratio. CoverForge applique ensuite typo, logo, couleurs et hiérarchie sans faux crop/flou.</p>
        </div>
        <div class="status-pill ready"><span class="status-dot"></span>Rust / resvg</div>
      </header>

      <div class="editor-layout">
        <div class="stack">
          <article class="card">
            <div class="card-header">
              <div>
                <strong>Show style</strong>
                <span class="card-sub">{templateData?.brand?.display_name || template || '—'}</span>
              </div>
              <span class="status-pill">{availableVariants.length} formats</span>
            </div>
            <div class="card-body stack">
              <div class="field">
                <label for="template">Émission</label>
                <select id="template" class="select" bind:value={template} onchange={onTemplateChange}>
                  {#each templates as t}<option value={t}>{t}</option>{/each}
                </select>
              </div>
              {#if templateData?.brand?.visual_summary}
                <div class="brand-summary">{templateData.brand.visual_summary}</div>
              {/if}
              <div class="grid two">
                <div class="field">
                  <label for="eyebrow">Eyebrow</label>
                  <input id="eyebrow" class="input" bind:value={eyebrow} />
                </div>
                <div class="field">
                  <label for="title">Titre</label>
                  <input id="title" class="input" bind:value={title} />
                </div>
              </div>
              <div class="grid two">
                <div class="field">
                  <label for="background">Key art natif du format</label>
                  <input id="background" class="input mono" bind:value={background} placeholder="/srv/storage/.../keyart.png" />
                </div>
                <div class="field">
                  <label for="logo">Logo</label>
                  <input id="logo" class="input mono" bind:value={logo} placeholder="/srv/storage/.../logo.png" />
                </div>
              </div>
              <div class="field">
                <span class="field-label">Formats</span>
                <div class="variant-grid">
                  {#each availableVariants as v}
                    <button type="button" class:active={variants.has(v)} class="variant-card" onclick={() => toggleVariant(v)}>
                      <strong>{v}</strong><span>layout natif</span>
                    </button>
                  {/each}
                </div>
              </div>
              <button class="btn primary render-button" onclick={render} disabled={rendering || !template || variants.size === 0}>
                {rendering ? 'Rendu…' : 'Prévisualiser'}
              </button>
              {#if error}<div class="error-box">{error}</div>{/if}
            </div>
          </article>

          <article class="card">
            <div class="card-header">
              <div>
                <strong>Polices</strong>
                <span class="card-sub">{fonts.length} faces disponibles · pack libre + custom</span>
              </div>
              {#if fontStatus}<span class="status-pill ready">{fontStatus}</span>{/if}
            </div>
            <div class="card-body stack">
              <div class="grid two">
                <div class="field">
                  <label for="font-search">Rechercher</label>
                  <input id="font-search" class="input" bind:value={fontQuery} placeholder="Roboto, Ubuntu, custom…" />
                </div>
                <div class="field">
                  <label for="font-upload">Ajouter une police custom</label>
                  <input id="font-upload" class="input file-input" type="file" accept=".ttf,.otf,.ttc,.otc" onchange={uploadFont} disabled={uploadingFont} />
                </div>
              </div>
              <div class="help">Tu peux aussi déposer directement les fichiers dans <code>Assets/CoverForge/Fonts/Custom/</code>. TTF, OTF, TTC, OTC · max 20 MiB.</div>
              <div class="help">Affichage : {Math.min(matchedFonts().length, 180)} / {matchedFonts().length} résultats. Utilise la recherche pour les 2 000+ faces disponibles.</div>
              <div class="font-list">
                {#each filteredFonts() as f}
                  <div class="font-row">
                    <div class="font-sample" style={previewStyle(f)}>Abc 123 — Le stand-up est mort</div>
                    <div class="font-meta">
                      <strong>{f.family}</strong>
                      <span>{f.style || 'Regular'} · {f.source}</span>
                    </div>
                    {#if f.source === 'custom'}
                      <button class="btn danger compact" onclick={() => deleteFont(f)}>Supprimer</button>
                    {/if}
                  </div>
                {/each}
              </div>
            </div>
          </article>

          <article class="card">
            <div class="card-header">
              <div><strong>JSON du show</strong><span class="card-sub">brand + fonts + palette + tous les layouts dans un seul fichier</span></div>
              {#if templateStatus}<span class="status-pill ready">{templateStatus}</span>{/if}
            </div>
            <div class="card-body stack">
              <textarea class="textarea mono template-editor" bind:value={templateJson} spellcheck="false"></textarea>
              <div class="row between">
                <span class="help">Coordonnées normalisées 0–1. Sauvegarde atomique, utilisée immédiatement par AUTOPUBLISHER.</span>
                <button class="btn primary" onclick={saveTemplate} disabled={savingTemplate || !template}>
                  {savingTemplate ? 'Sauvegarde…' : 'Sauvegarder'}
                </button>
              </div>
            </div>
          </article>
        </div>

        <aside class="card output-card">
          <div class="card-header">
            <div><strong>Preview</strong><span class="card-sub">{assets.length ? assets.length + ' rendu(s)' : 'En attente'}</span></div>
          </div>
          <div class="card-body output-body">
            {#if rendering}
              <div class="empty-state"><div class="pulse-ring"></div><strong>Rendu local</strong><span>resvg compose les layouts sélectionnés.</span></div>
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
                <strong>Pas de 16:9 recyclé avec fond flou.</strong>
                <span>Chaque key art doit déjà avoir été généré pour son ratio. CoverForge ne fait que la finition déterministe.</span>
              </div>
            {/if}
          </div>
        </aside>
      </div>
    </section>
  </main>
</div>
