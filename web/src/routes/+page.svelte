<script lang="ts">
  import { onMount, tick } from 'svelte';
  import AppShell from '../lib/components/AppShell.svelte';
  import ProjectHeader from '../lib/components/ProjectHeader.svelte';
  import { tr, type Locale } from '../lib/i18n';
  import { api } from '../lib/api';
  import type {
    CanvasDef,
    FontRecord,
    FormatDef,
    Frame,
    Layer,
    RenderResponse,
    RenderedAsset as Asset,
    TemplateData,
    View
  } from '../lib/types';

  let activeView = $state<View>('project');
  let locale = $state<Locale>('fr');
  let templates = $state<string[]>([]);
  let template = $state('');
  let templateData = $state<TemplateData|null>(null);
  let activeFormat = $state('default');
  let selectedLayerId = $state('');
  let variables = $state<Record<string,string>>({
    title:'LE STAND-UP EST MORT',
    eyebrow:'DPAFM #48',
    episode:'',
    subtitle:'',
    background:'',
    logo:''
  });

  let rendering = $state(false);
  let error = $state('');
  let assets = $state<Asset[]>([]);
  let templateJson = $state('');
  let templateStatus = $state('');
  let savingTemplate = $state(false);
  let dirty = $state(false);

  let fonts = $state<FontRecord[]>([]);
  let fontQuery = $state('');
  let uploadingFont = $state(false);
  let fontStatus = $state('');
  let fontPreviewNames = $state<Record<string,string>>({});

  let zoom = $state(1);
  let showGrid = $state(false);
  let showSafeArea = $state(true);
  let displayWidth = $state(720);
  let displayHeight = $state(405);

  let history = $state<string[]>([]);
  let historyIndex = $state(-1);
  let historyMuted = false;

  let canvasEl = $state<HTMLCanvasElement>();
  let workspaceEl = $state<HTMLDivElement>();
  let workspaceWidth = $state(860);
  let workspaceHeight = $state(650);
  let workspaceObserver:ResizeObserver|null = null;
  let fabricLib:any = null;
  let fabricCanvas:any = null;
  let rebuildingCanvas = false;

  const clampRange = (v:number,min:number,max:number) => Math.max(min, Math.min(max, Number.isFinite(v) ? v : min));
  const clamp01 = (v:number) => clampRange(v,0,1);
  const deepClone = <T,>(v:T):T => structuredClone(v);

  function humanizeLayerName(layer:any):string {
    const raw = String(layer?.text || layer?.source || '');
    const variable = raw.match(/^\s*\{\{([a-zA-Z0-9_-]+)\}\}\s*$/)?.[1]?.toLowerCase();
    const friendly:Record<string,string> = {
      title:'Titre',
      subtitle:'Sous-titre',
      eyebrow:'Numéro / eyebrow',
      episode:'Numéro d’épisode',
      episode_number:'Numéro d’épisode',
      number:'Numéro',
      guest:'Invité',
      logo:'Logo',
      background:'Fond',
      keyart:'Key art'
    };
    if (variable && friendly[variable]) return friendly[variable];
    const id = String(layer?.id || layer?.type || 'Calque')
      .replace(/[_-]+/g,' ')
      .replace(/\b\w/g,(m:string)=>m.toUpperCase());
    return id || 'Calque';
  }

  function normalizeLayer(layer:any):Layer {
    if (!String(layer.name || '').trim()) layer.name = humanizeLayerName(layer);
    layer.visible ??= true;
    layer.locked ??= false;
    layer.frame ??= { x:0.1, y:0.1, width:0.3, height:0.2 };
    if (layer.type === 'text') {
      layer.text ??= '{{title}}';
      layer.font_family ??= 'Inter';
      layer.font_weight ??= 700;
      layer.font_style ??= 'normal';
      layer.font_size ??= 0.08;
      layer.auto_fit ??= false;
      layer.min_font_size ??= Math.max(0.008, layer.font_size * 0.35);
      layer.color ??= '#ffffff';
      layer.stroke_color ??= null;
      layer.stroke_width ??= 0;
      layer.align ??= 'left';
      layer.max_lines ??= 4;
      layer.line_height ??= 1;
      layer.uppercase ??= false;
      layer.rotation_deg ??= 0;
      layer.opacity ??= 1;
    } else if (layer.type === 'image') {
      layer.source ??= '{{background}}';
      layer.fit ??= 'cover';
      layer.focal_x ??= 0.5;
      layer.focal_y ??= 0.5;
      layer.opacity ??= 1;
    } else if (layer.type === 'rect') {
      layer.fill ??= '#111111';
      layer.opacity ??= 1;
      layer.radius ??= 0;
    }
    return layer as Layer;
  }

  function normalizeTemplate(raw:any):TemplateData {
    raw.version ??= 1;
    raw.canvas ??= { width:1080, height:1080, background:'#000000' };
    raw.variants ??= {};
    raw.layers ??= [];
    raw.formats ??= {};
    raw.layers = raw.layers.map(normalizeLayer);
    for (const format of Object.values(raw.formats) as any[]) {
      format.canvas ??= deepClone(raw.canvas);
      format.layers ??= [];
      format.layers = format.layers.map(normalizeLayer);
    }
    return raw as TemplateData;
  }

  function formatKeys():string[] {
    if (!templateData) return ['default'];
    const keys = Object.keys(templateData.formats || {});
    if (keys.length) return keys;
    const variants = Object.keys(templateData.variants || {});
    return variants.length ? variants : ['default'];
  }

  function currentCanvasDef():CanvasDef|null {
    if (!templateData) return null;
    if (Object.keys(templateData.formats || {}).length) {
      return templateData.formats[activeFormat]?.canvas || null;
    }
    if (activeFormat === 'default') return templateData.canvas;
    return templateData.variants?.[activeFormat] || templateData.canvas;
  }

  function currentLayers():Layer[] {
    if (!templateData) return [];
    if (Object.keys(templateData.formats || {}).length) {
      return templateData.formats[activeFormat]?.layers || [];
    }
    return templateData.layers || [];
  }

  function selectedLayer():Layer|null {
    return currentLayers().find((l) => l.id === selectedLayerId) || null;
  }

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
      rules.push('@font-face{font-family:"' + name + '";src:url("' + f.url + '");font-weight:' + fontWeight(f) + ';font-style:' + fontStyle(f) + ';font-display:swap;}');
    });
    for (const family of availableFontFamilies()) {
      for (const face of fontFacesForFamily(family)) {
        if (!face.url) continue;
        const cssFamily = family.replaceAll('"','\\\"');
        rules.push('@font-face{font-family:"' + cssFamily + '";src:url("' + face.url + '");font-weight:' + fontWeight(face) + ';font-style:' + fontStyle(face) + ';font-display:swap;}');
      }
    }
    style.textContent = rules.join('\n');
    document.head.appendChild(style);
    fontPreviewNames = map;
  }

  function previewStyle(f:FontRecord) {
    const family = fontPreviewNames[fontKey(f)] || f.family;
    return 'font-family:"' + family.replaceAll('"','') + '",sans-serif;font-weight:' + fontWeight(f) + ';font-style:' + fontStyle(f);
  }

  function matchedFonts() {
    const q = fontQuery.trim().toLowerCase();
    if (!q) return fonts;
    return fonts.filter(f => (f.family + ' ' + f.style + ' ' + f.source).toLowerCase().includes(q));
  }

  function filteredFonts() {
    return matchedFonts().slice(0, 220);
  }

  function fontWeight(f:FontRecord):number {
    if (Number.isFinite(f.weight)) return Number(f.weight);
    const style = String(f.style || '').toLowerCase().replace(/[\s_-]+/g,'');
    if (style.includes('thin')) return 100;
    if (style.includes('extralight') || style.includes('ultralight')) return 200;
    if (style.includes('light')) return 300;
    if (style.includes('extrabold') || style.includes('ultrabold')) return 800;
    if (style.includes('semibold') || style.includes('demibold')) return 600;
    if (style.includes('bold')) return 700;
    if (style.includes('black') || style.includes('heavy')) return 900;
    if (style.includes('medium')) return 500;
    return 400;
  }

  function fontStyle(f:FontRecord):string {
    if (f.font_style) return f.font_style;
    return /italic|oblique/i.test(f.style || '') ? 'italic' : 'normal';
  }

  function fontFaceKey(f:FontRecord) {
    return [f.family, f.style || 'Regular', fontWeight(f), fontStyle(f)].join('|');
  }

  function availableFontFamilies() {
    return [...new Set(fonts.map((f) => f.family).filter(Boolean))].sort((a,b) => a.localeCompare(b));
  }

  function fontFacesForFamily(family:string):FontRecord[] {
    const priority:Record<string,number> = {custom:0,bundled:1,system:2};
    const sorted = fonts
      .filter((f) => f.family === family)
      .sort((a,b) => fontWeight(a)-fontWeight(b) || fontStyle(a).localeCompare(fontStyle(b)) || (priority[a.source] ?? 9)-(priority[b.source] ?? 9) || (a.style || '').localeCompare(b.style || ''));
    const seen = new Set<string>();
    return sorted.filter((f) => {
      const key = [f.style || 'Regular',fontWeight(f),fontStyle(f)].join('|');
      if (seen.has(key)) return false;
      seen.add(key);
      return true;
    });
  }

  function currentFontFaceKey(layer:Layer):string {
    const faces = fontFacesForFamily(layer.font_family || '');
    const exact = faces.find((f) => fontWeight(f) === Number(layer.font_weight || 400) && fontStyle(f) === String(layer.font_style || 'normal'));
    return exact ? fontFaceKey(exact) : (faces[0] ? fontFaceKey(faces[0]) : '');
  }

  async function loadLayerFont(layer:Layer) {
    if (layer.type !== 'text' || typeof document === 'undefined' || !('fonts' in document)) return;
    const family = String(layer.font_family || 'Inter').replaceAll('"','');
    const style = String(layer.font_style || 'normal');
    const weight = Number(layer.font_weight || 400);
    try {
      await document.fonts.load(style + ' ' + weight + ' 32px "' + family + '"');
    } catch {
      // Fabric will use the normal CSS fallback if a browser rejects a font face.
    }
  }

  async function setFontFamily(family:string) {
    const layer = selectedLayer();
    if (!layer || layer.type !== 'text') return;
    layer.font_family = family;
    const faces = fontFacesForFamily(family);
    const preferred = faces.find((f) => /regular|book/i.test(f.style || '') && fontStyle(f) === 'normal') || faces.find((f) => fontStyle(f) === 'normal') || faces[0];
    if (preferred) {
      layer.font_weight = fontWeight(preferred);
      layer.font_style = fontStyle(preferred);
    }
    markChanged(false);
    await loadLayerFont(layer);
    await rebuildCanvas();
  }

  async function setFontFace(key:string) {
    const layer = selectedLayer();
    if (!layer || layer.type !== 'text') return;
    const face = fontFacesForFamily(layer.font_family || '').find((f) => fontFaceKey(f) === key);
    if (!face) return;
    layer.font_weight = fontWeight(face);
    layer.font_style = fontStyle(face);
    markChanged(false);
    await loadLayerFont(layer);
    await rebuildCanvas();
  }

  async function loadFonts() {
    fonts = await api.listFonts();
    installFontPreviews();
    if (typeof document !== 'undefined' && 'fonts' in document) {
      await document.fonts.ready;
    }
    if (activeView === 'composer' || activeView === 'project') await rebuildCanvas();
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

  function collectVariableNames():string[] {
    if (!templateData) return [];
    const names = new Set<string>();
    const scan = (layers:Layer[]) => {
      for (const layer of layers) {
        for (const value of Object.values(layer)) {
          if (typeof value !== 'string') continue;
          for (const match of value.matchAll(/\{\{([a-zA-Z0-9_-]+)\}\}/g)) names.add(match[1]);
        }
      }
    };
    scan(templateData.layers || []);
    for (const f of Object.values(templateData.formats || {})) scan(f.layers || []);
    return [...names].sort();
  }

  function resolveVars(value:string):string {
    return String(value ?? '').replace(/\{\{([a-zA-Z0-9_-]+)\}\}/g, (_m,key) => variables[key] ?? '');
  }

  function assetUrl(source:string):string {
    const resolved = resolveVars(source).trim();
    return resolved ? '/v1/asset?path=' + encodeURIComponent(resolved) : '';
  }

  function snapshot():string {
    return JSON.stringify(templateData);
  }

  function pushHistory() {
    if (!templateData || historyMuted) return;
    const snap = snapshot();
    if (history[historyIndex] === snap) return;
    const next = history.slice(0, historyIndex + 1);
    next.push(snap);
    if (next.length > 80) next.shift();
    history = next;
    historyIndex = next.length - 1;
  }

  function resetHistory() {
    historyMuted = true;
    history = templateData ? [snapshot()] : [];
    historyIndex = history.length - 1;
    historyMuted = false;
  }

  async function restoreHistory(index:number) {
    if (index < 0 || index >= history.length) return;
    historyMuted = true;
    templateData = normalizeTemplate(JSON.parse(history[index]));
    historyIndex = index;
    templateJson = JSON.stringify(templateData, null, 2);
    dirty = true;
    const keys = formatKeys();
    if (!keys.includes(activeFormat)) activeFormat = keys[0] || 'default';
    if (!currentLayers().some((l) => l.id === selectedLayerId)) selectedLayerId = currentLayers()[0]?.id || '';
    await rebuildCanvas();
    historyMuted = false;
  }

  function undo() { void restoreHistory(historyIndex - 1); }
  function redo() { void restoreHistory(historyIndex + 1); }

  function markChanged(rebuild=true) {
    if (!templateData) return;
    templateJson = JSON.stringify(templateData, null, 2);
    dirty = true;
    pushHistory();
    if (rebuild) void rebuildCanvas();
  }

  async function loadTemplates() {
    try {
      templates = await api.listTemplates();
      if (!template || !templates.includes(template)) template = templates[0] || '';
      if (template) await loadTemplate(template);
      await loadFonts();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  async function loadTemplate(id:string) {
    templateStatus = '';
    error = '';
    const data = await api.getTemplate(id);
    templateData = normalizeTemplate(data);
    templateJson = JSON.stringify(templateData, null, 2);
    const keys = formatKeys();
    activeFormat = keys[0] || 'default';
    selectedLayerId = currentLayers().at(-1)?.id || currentLayers()[0]?.id || '';
    for (const key of collectVariableNames()) {
      if (!(key in variables)) variables[key] = '';
    }
    assets = [];
    dirty = false;
    resetHistory();
    await tick();
    await rebuildCanvas();
  }

  async function onTemplateChange() {
    try { await loadTemplate(template); }
    catch (e) { error = e instanceof Error ? e.message : String(e); }
  }

  async function chooseFormat(name:string) {
    activeFormat = name;
    selectedLayerId = currentLayers().at(-1)?.id || currentLayers()[0]?.id || '';
    assets = [];
    await rebuildCanvas();
  }

  async function saveTemplate() {
    if (!templateData) return;
    error = '';
    templateStatus = '';
    savingTemplate = true;
    try {
      const parsed = normalizeTemplate(JSON.parse(templateJson));
      if (parsed.id !== template) throw new Error('Template id must stay "' + template + '"');
      await api.putTemplate(template, parsed);
      templateData = parsed;
      templateJson = JSON.stringify(parsed, null, 2);
      dirty = false;
      templateStatus = 'Sauvegardé';
      resetHistory();
      await rebuildCanvas();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      savingTemplate = false;
    }
  }

  function applyJson() {
    try {
      const parsed = normalizeTemplate(JSON.parse(templateJson));
      if (parsed.id !== template) throw new Error('L’id JSON doit rester "' + template + '"');
      templateData = parsed;
      const keys = formatKeys();
      if (!keys.includes(activeFormat)) activeFormat = keys[0] || 'default';
      selectedLayerId = currentLayers().at(-1)?.id || currentLayers()[0]?.id || '';
      dirty = true;
      pushHistory();
      error = '';
      void rebuildCanvas();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  async function renderExact() {
    if (!templateData) return;
    error = '';
    assets = [];
    rendering = true;
    try {
      const data:RenderResponse = await api.renderPreview({
        template:templateData,
        variables,
        variants:[activeFormat]
      });
      assets = data.assets || [];
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      rendering = false;
    }
  }

  function uniqueLayerId(prefix:string) {
    const existing = new Set(currentLayers().map((l) => l.id));
    let i = 1;
    let id = prefix;
    while (existing.has(id)) id = prefix + '_' + i++;
    return id;
  }

  function addLayer(type:Layer['type']) {
    if (!templateData) return;
    const layers = currentLayers();
    let layer:Layer;
    if (type === 'text') {
      layer = normalizeLayer({
        type:'text', id:uniqueLayerId('text'), name:'Texte',
        frame:{x:0.08,y:0.1,width:0.56,height:0.18},
        text:'{{title}}', font_family:'Inter', font_weight:700, font_size:0.075,
        auto_fit:true, min_font_size:0.02, color:'#ffffff', stroke_color:null,
        stroke_width:0, align:'left', max_lines:3, line_height:0.95,
        uppercase:false, rotation_deg:0, opacity:1
      });
    } else if (type === 'image') {
      layer = normalizeLayer({
        type:'image', id:uniqueLayerId('image'), name:'Image',
        frame:{x:0.1,y:0.1,width:0.4,height:0.4},
        source:'{{background}}', fit:'cover', focal_x:0.5, focal_y:0.5, opacity:1
      });
    } else {
      layer = normalizeLayer({
        type:'rect', id:uniqueLayerId('rect'), name:'Rectangle',
        frame:{x:0.08,y:0.1,width:0.42,height:0.2},
        fill:'#111111', opacity:0.8, radius:0.02
      });
    }
    layers.push(layer);
    selectedLayerId = layer.id;
    markChanged();
  }

  function deleteSelectedLayer() {
    if (!templateData || !selectedLayerId) return;
    const layers = currentLayers();
    const i = layers.findIndex((l) => l.id === selectedLayerId);
    if (i < 0) return;
    layers.splice(i,1);
    selectedLayerId = layers[Math.min(i, layers.length - 1)]?.id || '';
    markChanged();
  }

  function duplicateSelectedLayer() {
    const layer = selectedLayer();
    if (!layer) return;
    const copy = deepClone(layer);
    copy.id = uniqueLayerId(layer.id + '_copy');
    copy.name = (layer.name || layer.id) + ' copie';
    copy.frame.x = clamp01(copy.frame.x + 0.02);
    copy.frame.y = clamp01(copy.frame.y + 0.02);
    const layers = currentLayers();
    const i = layers.findIndex((l) => l.id === layer.id);
    layers.splice(i + 1, 0, copy);
    selectedLayerId = copy.id;
    markChanged();
  }

  function moveLayer(delta:number) {
    const layers = currentLayers();
    const i = layers.findIndex((l) => l.id === selectedLayerId);
    if (i < 0) return;
    const target = Math.max(0, Math.min(layers.length - 1, i + delta));
    if (target === i) return;
    const [layer] = layers.splice(i,1);
    layers.splice(target,0,layer);
    markChanged();
  }

  function selectLayer(id:string) {
    selectedLayerId = id;
    if (!fabricCanvas) return;
    const obj = fabricCanvas.getObjects().find((o:any) => o.cfId === id);
    if (obj && obj.selectable !== false) {
      fabricCanvas.setActiveObject(obj);
      fabricCanvas.requestRenderAll();
    } else {
      fabricCanvas.discardActiveObject();
      fabricCanvas.requestRenderAll();
    }
  }

  function setLayerValue(key:string, value:any, rebuild=true) {
    const layer = selectedLayer();
    if (!layer) return;
    const previousSuggestedName = humanizeLayerName(layer);
    (layer as any)[key] = value;
    if ((key === 'text' || key === 'source') && (!layer.name || layer.name === 'Texte' || layer.name === 'Image' || layer.name === previousSuggestedName)) {
      layer.name = humanizeLayerName(layer);
    }
    markChanged(rebuild);
  }

  function setFrameValue(key:keyof Frame, value:number) {
    const layer = selectedLayer();
    if (!layer) return;
    layer.frame[key] = key === 'x' || key === 'y'
      ? clampRange(value,-1,2)
      : clampRange(value,0.001,2);
    markChanged();
  }

  function framePercent(layer:Layer, key:keyof Frame):number {
    return Math.round(Number(layer.frame[key] || 0) * 1000) / 10;
  }

  function setFramePercent(key:keyof Frame, value:number) {
    setFrameValue(key, value / 100);
  }

  function scalarPercent(value:number):number {
    return Math.round(Number(value || 0) * 1000) / 10;
  }

  function setLayerPercent(key:string, value:number) {
    setLayerValue(key, value / 100);
  }

  function toggleLayerVisible(layer:Layer) {
    layer.visible = layer.visible === false;
    markChanged();
  }

  function toggleLayerLocked(layer:Layer) {
    layer.locked = !layer.locked;
    markChanged();
  }

  function maybeMigrateLegacyFormats() {
    if (!templateData || Object.keys(templateData.formats || {}).length) return;
    const keys = Object.keys(templateData.variants || {});
    if (!keys.length) {
      templateData.formats = {
        default:{ canvas:deepClone(templateData.canvas), layers:deepClone(templateData.layers) }
      };
    } else {
      const formats:Record<string,FormatDef> = {};
      for (const key of keys) {
        formats[key] = { canvas:deepClone(templateData.variants[key]), layers:deepClone(templateData.layers) };
      }
      templateData.formats = formats;
    }
    activeFormat = Object.keys(templateData.formats)[0] || 'default';
    selectedLayerId = currentLayers().at(-1)?.id || '';
    markChanged();
  }

  function fitScaleFor(c:CanvasDef) {
    const maxW = Math.max(240, workspaceWidth - 28);
    const maxH = Math.max(220, workspaceHeight - 28);
    return Math.min(maxW / c.width, maxH / c.height) * zoom;
  }

  function attachWorkspaceObserver() {
    workspaceObserver?.disconnect();
    workspaceObserver = null;
    if (!workspaceEl) return;
    const update = () => {
      if (!workspaceEl) return;
      const nextW = Math.max(240, workspaceEl.clientWidth);
      const nextH = Math.max(220, workspaceEl.clientHeight);
      const changed = Math.abs(nextW-workspaceWidth) > 2 || Math.abs(nextH-workspaceHeight) > 2;
      workspaceWidth = nextW;
      workspaceHeight = nextH;
      if (changed && (activeView === 'composer' || activeView === 'project')) void rebuildCanvas();
    };
    update();
    workspaceObserver = new ResizeObserver(update);
    workspaceObserver.observe(workspaceEl);
  }

  async function switchView(view:View) {
    if (view === activeView) return;
    workspaceObserver?.disconnect();
    workspaceObserver = null;
    if (fabricCanvas) {
      fabricCanvas.dispose();
      fabricCanvas = null;
    }
    activeView = view;
    if (view === 'composer' || view === 'project') {
      await tick();
      initFabric();
      attachWorkspaceObserver();
      await rebuildCanvas();
    }
  }

  async function createFabricObject(layer:Layer, c:CanvasDef, scale:number) {
    const { Rect, Textbox, FabricImage } = fabricLib;
    const left = layer.frame.x * c.width * scale;
    const top = layer.frame.y * c.height * scale;
    const width = Math.max(2, layer.frame.width * c.width * scale);
    const height = Math.max(2, layer.frame.height * c.height * scale);
    const common:any = {
      left, top, angle:layer.rotation_deg || 0,
      opacity:layer.opacity ?? 1,
      selectable:layer.locked !== true,
      evented:layer.locked !== true,
      lockMovementX:layer.locked === true,
      lockMovementY:layer.locked === true,
      transparentCorners:false,
      cornerStyle:'circle',
      cornerColor:'#FF7A00',
      borderColor:'#FF7A00',
      cornerSize:10,
      padding:2
    };

    let obj:any;
    if (layer.type === 'rect') {
      obj = new Rect({
        ...common, width, height,
        fill:layer.fill || '#111111',
        rx:(layer.radius || 0) * Math.min(c.width,c.height) * scale,
        ry:(layer.radius || 0) * Math.min(c.width,c.height) * scale
      });
    } else if (layer.type === 'text') {
      const rawText = resolveVars(layer.text || '');
      const text = layer.uppercase ? rawText.toUpperCase() : rawText;
      const maxFont = Math.max(2, (layer.font_size || 0.06) * c.height * scale);
      const minFont = Math.max(2, (layer.min_font_size || (layer.font_size || 0.06) * 0.35) * c.height * scale);
      obj = new Textbox(text || 'Texte', {
        ...common,
        width,
        fontFamily:layer.font_family || 'Inter',
        fontWeight:String(layer.font_weight || 700),
        fontStyle:layer.font_style || 'normal',
        fontSize:maxFont,
        fill:layer.color || '#ffffff',
        stroke:layer.stroke_color || undefined,
        strokeWidth:(layer.stroke_width || 0) * c.height * scale,
        textAlign:layer.align || 'left',
        lineHeight:layer.line_height || 1,
        editable:layer.locked !== true && !String(layer.text || '').includes('{{'),
        splitByGrapheme:false
      });
      if (layer.auto_fit) {
        let low = minFont;
        let high = maxFont;
        const maxLines = Math.max(1, Number(layer.max_lines || 99));
        for (let i=0;i<14;i++) {
          const mid = (low + high) / 2;
          obj.set({fontSize:mid});
          obj.initDimensions();
          const lineCount = Array.isArray(obj.textLines) ? obj.textLines.length : 1;
          if ((obj.height || 0) <= height && lineCount <= maxLines) low = mid; else high = mid;
        }
        obj.set({fontSize:low});
        obj.initDimensions();
      }
    } else {
      const src = assetUrl(layer.source || '');
      if (src) {
        try {
          const image = await FabricImage.fromURL(src, { crossOrigin:'anonymous' });
          const sw = Math.max(1, image.width || 1);
          const sh = Math.max(1, image.height || 1);
          if ((layer.fit || 'cover') === 'contain') {
            const s = Math.min(width/sw, height/sh);
            image.set({
              ...common,
              left:left + (width - sw*s)/2,
              top:top + (height - sh*s)/2,
              scaleX:s, scaleY:s
            });
          } else {
            const targetAr = width / height;
            const sourceAr = sw / sh;
            let cropX = 0, cropY = 0, cropW = sw, cropH = sh;
            if (sourceAr > targetAr) {
              cropW = sh * targetAr;
              cropX = (sw - cropW) * clamp01(layer.focal_x ?? 0.5);
            } else {
              cropH = sw / targetAr;
              cropY = (sh - cropH) * clamp01(layer.focal_y ?? 0.5);
            }
            image.set({
              ...common, cropX, cropY, width:cropW, height:cropH,
              scaleX:width/cropW, scaleY:height/cropH
            });
          }
          obj = image;
        } catch {
          obj = new Rect({...common,width,height,fill:'#2A1B10',stroke:'#FF7A00',strokeDashArray:[8,6]});
        }
      } else {
        obj = new Rect({...common,width,height,fill:'#2A1B10',stroke:'#FF7A00',strokeDashArray:[8,6]});
      }
    }
    obj.cfId = layer.id;
    obj.cfType = layer.type;
    obj.cfFrameHeight = height;
    return obj;
  }

  async function rebuildCanvas() {
    if (!fabricCanvas || !fabricLib || !templateData) return;
    const c = currentCanvasDef();
    if (!c) return;
    rebuildingCanvas = true;
    const scale = fitScaleFor(c);
    displayWidth = Math.max(220, Math.round(c.width * scale));
    displayHeight = Math.max(180, Math.round(c.height * scale));
    fabricCanvas.clear();
    fabricCanvas.setDimensions({ width:displayWidth, height:displayHeight });
    fabricCanvas.backgroundColor = c.background || '#000000';
    for (const layer of currentLayers()) {
      if (layer.visible === false) continue;
      const obj = await createFabricObject(layer,c,scale);
      fabricCanvas.add(obj);
    }
    const active = fabricCanvas.getObjects().find((o:any) => o.cfId === selectedLayerId);
    if (active && active.selectable !== false) fabricCanvas.setActiveObject(active);
    fabricCanvas.requestRenderAll();
    rebuildingCanvas = false;
  }

  function syncLayerFromFabric(obj:any) {
    if (rebuildingCanvas || !obj || !templateData) return;
    const layer = currentLayers().find((l) => l.id === obj.cfId);
    const c = currentCanvasDef();
    if (!layer || !c) return;
    const scale = fitScaleFor(c);
    const fullW = c.width * scale;
    const fullH = c.height * scale;
    layer.frame.x = clampRange((obj.left || 0) / fullW,-1,2);
    layer.frame.y = clampRange((obj.top || 0) / fullH,-1,2);
    layer.frame.width = clampRange(Math.max(2, obj.getScaledWidth()) / fullW,0.001,2);
    if (layer.type !== 'text') {
      layer.frame.height = clampRange(Math.max(2, obj.getScaledHeight()) / fullH,0.001,2);
    }
    if (layer.type === 'text') {
      layer.rotation_deg = Number(obj.angle || 0);
      if (!String(layer.text || '').includes('{{') && typeof obj.text === 'string') layer.text = obj.text;
    }
    templateJson = JSON.stringify(templateData, null, 2);
    dirty = true;
    pushHistory();
  }

  function initFabric() {
    if (!fabricLib || !canvasEl || fabricCanvas) return;
    fabricCanvas = new fabricLib.Canvas(canvasEl, {
      preserveObjectStacking:true,
      selection:true,
      stopContextMenu:true,
      backgroundColor:'#000000'
    });
    const updateSelection = (e:any) => {
      const target = e?.selected?.[0] || fabricCanvas.getActiveObject();
      if (target?.cfId) selectedLayerId = target.cfId;
    };
    fabricCanvas.on('selection:created', updateSelection);
    fabricCanvas.on('selection:updated', updateSelection);
    fabricCanvas.on('object:modified', (e:any) => syncLayerFromFabric(e.target));
    fabricCanvas.on('text:editing:exited', (e:any) => syncLayerFromFabric(e.target));
  }

  function onVariableInput(key:string, value:string) {
    variables[key] = value;
    void rebuildCanvas();
  }

  function setZoom(next:number) {
    zoom = Math.max(0.35, Math.min(2.2, next));
    void rebuildCanvas();
  }

  function isEditingTarget(target:EventTarget|null) {
    const el = target as HTMLElement|null;
    return !!el && ['INPUT','TEXTAREA','SELECT'].includes(el.tagName);
  }

  function onKeydown(e:KeyboardEvent) {
    const mod = e.ctrlKey || e.metaKey;
    if (mod && e.key.toLowerCase() === 's') {
      e.preventDefault();
      void saveTemplate();
      return;
    }
    if (mod && e.key.toLowerCase() === 'z') {
      e.preventDefault();
      e.shiftKey ? redo() : undo();
      return;
    }
    if (mod && e.key.toLowerCase() === 'y') {
      e.preventDefault();
      redo();
      return;
    }
    if (isEditingTarget(e.target)) return;
    if ((e.key === 'Delete' || e.key === 'Backspace') && selectedLayerId) {
      e.preventDefault();
      deleteSelectedLayer();
    } else if (mod && e.key.toLowerCase() === 'd') {
      e.preventDefault();
      duplicateSelectedLayer();
    }
  }

  function viewMeta(view:View):{title:string;copy:string} {
    switch (view) {
      case 'templates': return { title: tr('fr','nav.templates') + ' / ' + tr('zh-CN','nav.templates'), copy: tr(locale,'copy.templates') };
      case 'library': return { title: tr('fr','nav.library') + ' / ' + tr('zh-CN','nav.library'), copy: tr(locale,'copy.library') };
      case 'brand': return { title: tr('fr','nav.brand') + ' / ' + tr('zh-CN','nav.brand'), copy: tr(locale,'copy.brand') };
      case 'exports': return { title: tr('fr','nav.exports') + ' / ' + tr('zh-CN','nav.exports'), copy: tr(locale,'copy.exports') };
      case 'api': return { title: tr('fr','nav.api') + ' / ' + tr('zh-CN','nav.api'), copy: tr(locale,'copy.api') };
      default: return { title:'CoverForge', copy:tr(locale,'copy.product') };
    }
  }

  onMount(() => {
    let disposed = false;
    void (async () => {
      fabricLib = await import('fabric');
      if (disposed) return;
      await tick();
      initFabric();
      attachWorkspaceObserver();
      window.addEventListener('keydown', onKeydown);
      await loadTemplates();
    })();
    return () => {
      disposed = true;
      window.removeEventListener('keydown', onKeydown);
      workspaceObserver?.disconnect();
      workspaceObserver = null;
      fabricCanvas?.dispose();
      fabricCanvas = null;
    };
  });
</script>

<svelte:head>
  <title>CoverForge Studio</title>
  <meta name="description" content="CoverForge — self-hosted branding automation studio and deterministic multi-format renderer" />
</svelte:head>

<AppShell {activeView} {locale} {dirty} onNavigate={(view) => void switchView(view)} onLocale={(next) => locale = next}>
  <main class="app-main">
    {#if activeView === 'project' || activeView === 'composer'}
      <ProjectHeader {locale} {template} />
    {/if}

    <header class="topbar">
      <div class="topbar-left">
        <select class="select compact-select" bind:value={template} onchange={onTemplateChange}>
          {#each templates as t}<option value={t}>{t}</option>{/each}
        </select>
        {#if templateData}
          <div class="format-tabs">
            {#each formatKeys() as f}
              <button class:active={activeFormat===f} class="format-tab" onclick={() => chooseFormat(f)}>{f}</button>
            {/each}
          </div>
        {/if}
      </div>
      <div class="topbar-actions">
        <button class="btn compact" onclick={undo} disabled={historyIndex<=0}>↶</button>
        <button class="btn compact" onclick={redo} disabled={historyIndex>=history.length-1}>↷</button>
        <button class="btn compact" onclick={() => setZoom(zoom-0.1)}>−</button>
        <span class="zoom-readout">{Math.round(zoom*100)}%</span>
        <button class="btn compact" onclick={() => setZoom(zoom+0.1)}>+</button>
        <button class="btn compact" class:active-tool={showGrid} onclick={() => showGrid=!showGrid}>Grille</button>
        <button class="btn compact" class:active-tool={showSafeArea} onclick={() => showSafeArea=!showSafeArea}>Safe</button>
        <button class="btn compact utility-tab" onclick={() => void switchView('fonts')}>Polices / 字体</button>
        <button class="btn compact utility-tab" onclick={() => void switchView('json')}>JSON</button>
        <button class="btn primary" onclick={saveTemplate} disabled={savingTemplate || !template || !dirty}>
          {savingTemplate ? 'Sauvegarde…' : dirty ? 'Sauvegarder' : 'Sauvegardé'}
        </button>
      </div>
    </header>

    {#if error}<div class="global-error">{error}</div>{/if}

    {#if activeView === 'project' || activeView === 'composer'}
      <section class="studio">
        <aside class="layers-panel panel">
          <div class="panel-head">
            <div><strong>Calques</strong><span>{currentLayers().length}</span></div>
            <div class="mini-actions">
              <button class="icon-btn" title="Ajouter texte" onclick={() => addLayer('text')}>T</button>
              <button class="icon-btn" title="Ajouter image" onclick={() => addLayer('image')}>▧</button>
              <button class="icon-btn" title="Ajouter rectangle" onclick={() => addLayer('rect')}>□</button>
            </div>
          </div>

          {#if templateData && !Object.keys(templateData.formats || {}).length}
            <button class="migration-card" onclick={maybeMigrateLegacyFormats}>
              <strong>Layouts encore liés</strong>
              <span>Convertir vers des calques indépendants par format pour placer chaque élément différemment.</span>
            </button>
          {/if}

          <div class="layer-list">
            {#each [...currentLayers()].reverse() as layer (layer.id)}
              <div role="button" tabindex="0" class:selected={selectedLayerId===layer.id} class:muted={layer.visible===false} class="layer-row" onclick={() => selectLayer(layer.id)} onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') selectLayer(layer.id); }}>
                <span class="layer-type">{layer.type==='text'?'T':layer.type==='image'?'▧':'□'}</span>
                <span class="layer-copy">
                  <strong>{layer.name || layer.id}</strong>
                  <small>{layer.id}</small>
                </span>
                <span class="layer-controls">
                  <button class="tiny" title="Visible" onclick={(e) => {e.stopPropagation();toggleLayerVisible(layer)}}>{layer.visible===false?'○':'●'}</button>
                  <button class="tiny" title="Verrouiller" onclick={(e) => {e.stopPropagation();toggleLayerLocked(layer)}}>{layer.locked?'🔒':'◇'}</button>
                </span>
              </div>
            {/each}
          </div>

          <div class="layer-footer">
            <button class="btn compact" onclick={() => moveLayer(1)} title="Monter">↑</button>
            <button class="btn compact" onclick={() => moveLayer(-1)} title="Descendre">↓</button>
            <button class="btn compact" onclick={duplicateSelectedLayer} disabled={!selectedLayerId}>Dupliquer</button>
            <button class="btn compact danger" onclick={deleteSelectedLayer} disabled={!selectedLayerId}>Supprimer</button>
          </div>
        </aside>

        <section class="workspace-panel">
          <div class="workspace-toolbar">
            <div class="variable-strip">
              {#each collectVariableNames() as key}
                <label>
                  <span>{key}</span>
                  <input class="input variable-input" value={variables[key] ?? ''} oninput={(e) => onVariableInput(key,(e.currentTarget as HTMLInputElement).value)} placeholder={key==='background'||key==='logo'?'/srv/storage/...':'Valeur de test'} />
                </label>
              {/each}
            </div>
          </div>

          <div class="workspace" bind:this={workspaceEl}>
            <div class="canvas-stage" style:width={displayWidth+'px'} style:height={displayHeight+'px'}>
              <canvas bind:this={canvasEl}></canvas>
              {#if showGrid}<div class="grid-overlay"></div>{/if}
              {#if showSafeArea}<div class="safe-overlay"></div>{/if}
            </div>
          </div>

          <div class="workspace-status">
            <span>{currentCanvasDef()?.width || 0} × {currentCanvasDef()?.height || 0}</span>
            <span>Format : {activeFormat}</span>
            <span>Preview interactif Fabric.js · rendu final Rust/resvg</span>
            <button class="btn compact" onclick={renderExact} disabled={rendering || !templateData}>
              {rendering?'Rendu…':'Vérifier en Rust'}
            </button>
          </div>

          {#if assets.length}
            <div class="render-strip">
              {#each assets as asset}
                <a href={asset.url} target="_blank" rel="noreferrer"><img src={asset.url} alt={asset.variant} /><span>{asset.variant} · {asset.width}×{asset.height}</span></a>
              {/each}
            </div>
          {/if}
        </section>

        <aside class="inspector panel">
          <div class="panel-head"><div><strong>Propriétés</strong><span>{selectedLayer()?.type || 'aucun calque'}</span></div></div>

          {#if selectedLayer()}
            {@const layer = selectedLayer()!}
            <div class="inspector-body">
              <div class="field">
                <span class="field-label">Nom du calque</span>
                <input class="input" value={layer.name || ''} oninput={(e) => setLayerValue('name',(e.currentTarget as HTMLInputElement).value,false)} />
              </div>

              <div class="field">
                <span class="field-label">ID / clé API</span>
                <input class="input mono" value={layer.id} readonly />
              </div>

              <div class="subhead">Position & taille</div>
              <div class="range-grid">
                <label class="range-field">
                  <span>X</span>
                  <input type="range" min="-50" max="150" step="0.1" value={framePercent(layer,'x')} oninput={(e)=>setFramePercent('x',+(e.currentTarget as HTMLInputElement).value)} />
                  <input class="range-number" type="number" min="-100" max="200" step="0.1" value={framePercent(layer,'x')} oninput={(e)=>setFramePercent('x',+(e.currentTarget as HTMLInputElement).value)} />
                  <em>%</em>
                </label>
                <label class="range-field">
                  <span>Y</span>
                  <input type="range" min="-50" max="150" step="0.1" value={framePercent(layer,'y')} oninput={(e)=>setFramePercent('y',+(e.currentTarget as HTMLInputElement).value)} />
                  <input class="range-number" type="number" min="-100" max="200" step="0.1" value={framePercent(layer,'y')} oninput={(e)=>setFramePercent('y',+(e.currentTarget as HTMLInputElement).value)} />
                  <em>%</em>
                </label>
                <label class="range-field">
                  <span>L</span>
                  <input type="range" min="1" max="200" step="0.1" value={framePercent(layer,'width')} oninput={(e)=>setFramePercent('width',+(e.currentTarget as HTMLInputElement).value)} />
                  <input class="range-number" type="number" min="0.1" max="200" step="0.1" value={framePercent(layer,'width')} oninput={(e)=>setFramePercent('width',+(e.currentTarget as HTMLInputElement).value)} />
                  <em>%</em>
                </label>
                <label class="range-field">
                  <span>H</span>
                  <input type="range" min="1" max="200" step="0.1" value={framePercent(layer,'height')} oninput={(e)=>setFramePercent('height',+(e.currentTarget as HTMLInputElement).value)} />
                  <input class="range-number" type="number" min="0.1" max="200" step="0.1" value={framePercent(layer,'height')} oninput={(e)=>setFramePercent('height',+(e.currentTarget as HTMLInputElement).value)} />
                  <em>%</em>
                </label>
              </div>

              <div class="toggle-row">
                <label><input type="checkbox" checked={layer.visible!==false} onchange={() => toggleLayerVisible(layer)} /> Visible</label>
                <label><input type="checkbox" checked={layer.locked===true} onchange={() => toggleLayerLocked(layer)} /> Verrouillé</label>
              </div>

              {#if layer.type === 'text'}
                <div class="subhead">Texte</div>
                <div class="field">
                  <span class="field-label">Contenu / variable</span>
                  <textarea class="textarea compact-area" value={layer.text || ''} oninput={(e)=>setLayerValue('text',(e.currentTarget as HTMLTextAreaElement).value)}></textarea>
                  <span class="help">Variables API : <code>{'{{title}}'}</code>, <code>{'{{episode}}'}</code>, <code>{'{{subtitle}}'}</code>…</span>
                </div>
                <div class="grid two font-pickers">
                  <label class="mini-field">
                    <span>Famille</span>
                    <input class="input" list="font-families" value={layer.font_family || ''} onchange={(e)=>void setFontFamily((e.currentTarget as HTMLInputElement).value)} />
                    <datalist id="font-families">{#each availableFontFamilies() as family}<option value={family}></option>{/each}</datalist>
                  </label>
                  <label class="mini-field">
                    <span>Variante</span>
                    <select value={currentFontFaceKey(layer)} onchange={(e)=>void setFontFace((e.currentTarget as HTMLSelectElement).value)}>
                      {#each fontFacesForFamily(layer.font_family || '') as face}
                        <option value={fontFaceKey(face)}>{face.style || 'Regular'} · {fontWeight(face)}{fontStyle(face)==='italic'?' · italic':''}</option>
                      {/each}
                    </select>
                  </label>
                </div>
                <label class="range-field">
                  <span>Taille</span>
                  <input type="range" min="0.5" max="30" step="0.1" value={scalarPercent(layer.font_size || 0.06)} oninput={(e)=>setLayerPercent('font_size',+(e.currentTarget as HTMLInputElement).value)} />
                  <input class="range-number" type="number" min="0.1" max="50" step="0.1" value={scalarPercent(layer.font_size || 0.06)} oninput={(e)=>setLayerPercent('font_size',+(e.currentTarget as HTMLInputElement).value)} />
                  <em>% H</em>
                </label>
                <div class="toggle-row">
                  <label><input type="checkbox" checked={layer.auto_fit===true} onchange={(e)=>setLayerValue('auto_fit',(e.currentTarget as HTMLInputElement).checked)} /> Adapter au bloc</label>
                  <label><input type="checkbox" checked={layer.uppercase===true} onchange={(e)=>setLayerValue('uppercase',(e.currentTarget as HTMLInputElement).checked)} /> Capitales</label>
                </div>
                {#if layer.auto_fit}
                  <div class="range-grid compact-ranges">
                    <label class="range-field">
                      <span>Taille mini</span>
                      <input type="range" min="0.2" max="20" step="0.1" value={scalarPercent(layer.min_font_size || 0.015)} oninput={(e)=>setLayerPercent('min_font_size',+(e.currentTarget as HTMLInputElement).value)} />
                      <input class="range-number" type="number" min="0.1" max="30" step="0.1" value={scalarPercent(layer.min_font_size || 0.015)} oninput={(e)=>setLayerPercent('min_font_size',+(e.currentTarget as HTMLInputElement).value)} />
                      <em>% H</em>
                    </label>
                    <label class="range-field">
                      <span>Lignes</span>
                      <input type="range" min="1" max="12" step="1" value={layer.max_lines || 4} oninput={(e)=>setLayerValue('max_lines',+(e.currentTarget as HTMLInputElement).value)} />
                      <input class="range-number" type="number" min="1" max="20" step="1" value={layer.max_lines || 4} oninput={(e)=>setLayerValue('max_lines',+(e.currentTarget as HTMLInputElement).value)} />
                      <em>max</em>
                    </label>
                  </div>
                {/if}
                <details class="advanced-block">
                  <summary>Typographie avancée</summary>
                  <div class="advanced-content">
                    <div class="grid two">
                      <label class="mini-field"><span>Graisse CSS</span><input type="number" min="100" max="1000" step="10" value={layer.font_weight || 700} onchange={(e)=>setLayerValue('font_weight',+(e.currentTarget as HTMLInputElement).value)} /></label>
                      <label class="mini-field"><span>Interligne</span><input type="number" min="0.5" max="3" step="0.05" value={layer.line_height || 1} onchange={(e)=>setLayerValue('line_height',+(e.currentTarget as HTMLInputElement).value)} /></label>
                    </div>
                    <label class="range-field">
                      <span>Rotation</span>
                      <input type="range" min="-180" max="180" step="1" value={layer.rotation_deg || 0} oninput={(e)=>setLayerValue('rotation_deg',+(e.currentTarget as HTMLInputElement).value)} />
                      <input class="range-number" type="number" min="-360" max="360" step="1" value={layer.rotation_deg || 0} oninput={(e)=>setLayerValue('rotation_deg',+(e.currentTarget as HTMLInputElement).value)} />
                      <em>°</em>
                    </label>
                    <label class="range-field">
                      <span>Opacité</span>
                      <input type="range" min="0" max="100" step="1" value={scalarPercent(layer.opacity ?? 1)} oninput={(e)=>setLayerPercent('opacity',+(e.currentTarget as HTMLInputElement).value)} />
                      <input class="range-number" type="number" min="0" max="100" step="1" value={scalarPercent(layer.opacity ?? 1)} oninput={(e)=>setLayerPercent('opacity',+(e.currentTarget as HTMLInputElement).value)} />
                      <em>%</em>
                    </label>
                    <div class="grid two">
                      <label class="mini-field"><span>Couleur</span><input type="color" value={layer.color || '#ffffff'} oninput={(e)=>setLayerValue('color',(e.currentTarget as HTMLInputElement).value)} /></label>
                      <label class="mini-field"><span>Alignement</span>
                        <select value={layer.align || 'left'} onchange={(e)=>setLayerValue('align',(e.currentTarget as HTMLSelectElement).value)}>
                          <option value="left">Gauche</option><option value="center">Centre</option><option value="right">Droite</option>
                        </select>
                      </label>
                    </div>
                    <div class="grid two">
                      <label class="mini-field"><span>Contour</span><input type="color" value={layer.stroke_color || '#000000'} oninput={(e)=>setLayerValue('stroke_color',(e.currentTarget as HTMLInputElement).value)} /></label>
                      <label class="mini-field"><span>Épaisseur</span><input type="number" min="0" max="0.05" step="0.0005" value={layer.stroke_width || 0} onchange={(e)=>setLayerValue('stroke_width',+(e.currentTarget as HTMLInputElement).value)} /></label>
                    </div>
                  </div>
                </details>
              {:else if layer.type === 'image'}
                <div class="subhead">Image</div>
                <div class="field">
                  <span class="field-label">Source / variable</span>
                  <input class="input mono" value={layer.source || ''} oninput={(e)=>setLayerValue('source',(e.currentTarget as HTMLInputElement).value)} />
                </div>
                <div class="grid two">
                  <label class="mini-field"><span>Ajustement</span>
                    <select value={layer.fit || 'cover'} onchange={(e)=>setLayerValue('fit',(e.currentTarget as HTMLSelectElement).value)}>
                      <option value="cover">Cover</option><option value="contain">Contain</option>
                    </select>
                  </label>
                </div>
                <div class="range-grid compact-ranges">
                  <label class="range-field">
                    <span>Opacité</span>
                    <input type="range" min="0" max="100" step="1" value={scalarPercent(layer.opacity ?? 1)} oninput={(e)=>setLayerPercent('opacity',+(e.currentTarget as HTMLInputElement).value)} />
                    <input class="range-number" type="number" min="0" max="100" step="1" value={scalarPercent(layer.opacity ?? 1)} oninput={(e)=>setLayerPercent('opacity',+(e.currentTarget as HTMLInputElement).value)} />
                    <em>%</em>
                  </label>
                  <label class="range-field">
                    <span>Focal X</span>
                    <input type="range" min="0" max="100" step="1" value={scalarPercent(layer.focal_x ?? 0.5)} oninput={(e)=>setLayerPercent('focal_x',+(e.currentTarget as HTMLInputElement).value)} />
                    <input class="range-number" type="number" min="0" max="100" step="1" value={scalarPercent(layer.focal_x ?? 0.5)} oninput={(e)=>setLayerPercent('focal_x',+(e.currentTarget as HTMLInputElement).value)} />
                    <em>%</em>
                  </label>
                  <label class="range-field">
                    <span>Focal Y</span>
                    <input type="range" min="0" max="100" step="1" value={scalarPercent(layer.focal_y ?? 0.5)} oninput={(e)=>setLayerPercent('focal_y',+(e.currentTarget as HTMLInputElement).value)} />
                    <input class="range-number" type="number" min="0" max="100" step="1" value={scalarPercent(layer.focal_y ?? 0.5)} oninput={(e)=>setLayerPercent('focal_y',+(e.currentTarget as HTMLInputElement).value)} />
                    <em>%</em>
                  </label>
                </div>
              {:else}
                <div class="subhead">Rectangle</div>
                <div class="grid two">
                  <label class="mini-field"><span>Couleur</span><input type="color" value={layer.fill || '#111111'} oninput={(e)=>setLayerValue('fill',(e.currentTarget as HTMLInputElement).value)} /></label>
                </div>
                <div class="range-grid compact-ranges">
                  <label class="range-field">
                    <span>Opacité</span>
                    <input type="range" min="0" max="100" step="1" value={scalarPercent(layer.opacity ?? 1)} oninput={(e)=>setLayerPercent('opacity',+(e.currentTarget as HTMLInputElement).value)} />
                    <input class="range-number" type="number" min="0" max="100" step="1" value={scalarPercent(layer.opacity ?? 1)} oninput={(e)=>setLayerPercent('opacity',+(e.currentTarget as HTMLInputElement).value)} />
                    <em>%</em>
                  </label>
                  <label class="range-field">
                    <span>Rayon</span>
                    <input type="range" min="0" max="50" step="0.5" value={scalarPercent(layer.radius || 0)} oninput={(e)=>setLayerPercent('radius',+(e.currentTarget as HTMLInputElement).value)} />
                    <input class="range-number" type="number" min="0" max="50" step="0.5" value={scalarPercent(layer.radius || 0)} oninput={(e)=>setLayerPercent('radius',+(e.currentTarget as HTMLInputElement).value)} />
                    <em>%</em>
                  </label>
                </div>
              {/if}
            </div>
          {:else}
            <div class="empty-inspector">Sélectionne un calque sur le canvas ou dans la pile.</div>
          {/if}
        </aside>
      </section>

    {:else if activeView === 'fonts'}
      <section class="page-section">
        <header class="section-head">
          <div><span class="eyebrow">TYPOGRAPHIE</span><h1>Bibliothèque de polices</h1><p>{fonts.length} faces disponibles côté renderer.</p></div>
          {#if fontStatus}<span class="status-pill">{fontStatus}</span>{/if}
        </header>
        <div class="fonts-toolbar panel">
          <input class="input" bind:value={fontQuery} placeholder="Rechercher une police…" />
          <label class="btn file-btn">Ajouter une police<input type="file" accept=".ttf,.otf,.ttc,.otc" onchange={uploadFont} disabled={uploadingFont} /></label>
        </div>
        <div class="font-grid">
          {#each filteredFonts() as f}
            <article class="font-card panel">
              <div class="font-sample" style={previewStyle(f)}>Aa Bb Cc 123 — CoverForge</div>
              <div class="row between">
                <div><strong>{f.family}</strong><span>{f.style || 'Regular'} · {f.source}</span></div>
                {#if f.source === 'custom'}<button class="btn compact danger" onclick={() => deleteFont(f)}>Supprimer</button>{/if}
              </div>
            </article>
          {/each}
        </div>
      </section>

    {:else if activeView === 'json'}
      <section class="page-section json-page">
        <header class="section-head">
          <div><span class="eyebrow">SOURCE DE VÉRITÉ / 数据源</span><h1>JSON du modèle / 模板 JSON</h1><p>Lisible, versionnable et pilotable par API ou agents.</p></div>
          <div class="row">
            <button class="btn" onclick={applyJson}>Appliquer au studio</button>
            <button class="btn primary" onclick={saveTemplate} disabled={!dirty || savingTemplate}>Sauvegarder</button>
          </div>
        </header>
        <textarea class="textarea mono json-editor" bind:value={templateJson} spellcheck="false" oninput={() => dirty=true}></textarea>
        <div class="json-help">
          <code>GET /openapi.json</code>
          <code>GET /v1/templates/{'{id}'}</code>
          <code>PUT /v1/templates/{'{id}'}</code>
          <code>GET /v1/templates/{'{id}'}/dataset</code>
          <code>POST /v1/render</code>
          <code>POST /v1/render/preview</code>
          <code>POST /v1/render/batch</code>
          <code>GET /v1/fonts</code>
          <code>GET /v1/asset?path=…</code>
        </div>
      </section>
    {:else}
      {@const meta = viewMeta(activeView)}
      <section class="page-section product-placeholder">
        <div class="placeholder-orbit"><img src="/favicon.svg" alt="" /></div>
        <span class="eyebrow">COVERFORGE</span>
        <h1>{meta.title}</h1>
        <p>{meta.copy}</p>
        <span class="placeholder-note">La vue fonctionnelle arrive dans les étapes suivantes de v0.6 / 此功能将在 v0.6 后续步骤中启用</span>
      </section>
    {/if}
  </main>
</AppShell>
