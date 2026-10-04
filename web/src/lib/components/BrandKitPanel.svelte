<script lang="ts">
  import { api } from '../api';
  import { ui, type Locale } from '../i18n';
  import type { BrandStyle, FontRecord, UploadedAsset } from '../types';

  let {
    locale='fr',
    brand = {},
    fonts = [],
    onChange,
    onAssetUploaded
  }: {
    locale?:Locale;
    brand?:BrandStyle;
    fonts?:FontRecord[];
    onChange:(brand:BrandStyle)=>void;
    onAssetUploaded:(asset:UploadedAsset)=>void;
  } = $props();

  let uploading = $state('');
  let error = $state('');

  const clone = ():BrandStyle => ({
    display_name:brand.display_name || '',
    visual_summary:brand.visual_summary || '',
    palette:{...(brand.palette || {})},
    fonts:{...(brand.fonts || {})},
    logos:{...(brand.logos || {})},
    notes:[...(brand.notes || [])]
  });

  function setField(key:'display_name'|'visual_summary',value:string) { const next=clone(); next[key]=value; onChange(next); }
  function setMap(kind:'palette'|'fonts'|'logos',key:string,value:string) { const next=clone(); (next[kind] ||= {})[key]=value; onChange(next); }
  function removeMap(kind:'palette'|'fonts'|'logos',key:string) { const next=clone(); delete (next[kind] ||= {})[key]; onChange(next); }
  function addPalette(){const next=clone(); let i=1,key='accent';while(next.palette?.[key])key='color'+i++;(next.palette||={})[key]='#FF7A00';onChange(next)}
  function addFont(){const next=clone(); let i=1,key='heading';while(next.fonts?.[key])key='font'+i++;(next.fonts||={})[key]=families()[0]||'Inter';onChange(next)}
  function families(){return [...new Set(fonts.map(f=>f.family).filter(Boolean))].sort((a,b)=>a.localeCompare(b))}

  async function uploadLogo(slot:string,event:Event) {
    const input=event.currentTarget as HTMLInputElement;
    const file=input.files?.[0]; if(!file)return;
    uploading=slot; error='';
    try {
      const asset=await api.uploadAsset(file);
      setMap('logos',slot,asset.path);
      onAssetUploaded(asset);
    } catch(e) { error=e instanceof Error?e.message:String(e); }
    finally { uploading=''; input.value=''; }
  }
</script>

<section class="brand-kit-page page-section">
  <header class="section-head">
    <div>
      <span class="eyebrow">{ui(locale,'KIT DE MARQUE','BRAND KIT','品牌工具包')}</span>
      <h1>{ui(locale,'Identité réutilisable','Reusable identity','可复用品牌资产')}</h1>
      <p>{ui(locale,'Logos, palette et typographies suivent chaque format et restent accessibles par API.','Logos, palette and typography follow every format and stay available through the API.','Logo、配色与字体会应用到各格式，并可通过 API 使用。')}</p>
    </div>
  </header>

  <div class="brand-kit-grid">
    <section class="panel brand-section">
      <div class="panel-head"><div><strong>{ui(locale,'Identité','Identity','品牌信息')}</strong></div></div>
      <div class="brand-form">
        <label class="field"><span class="field-label">{ui(locale,'Nom','Name','名称')}</span><input class="input" value={brand.display_name||''} oninput={(e)=>setField('display_name',(e.currentTarget as HTMLInputElement).value)} /></label>
        <label class="field"><span class="field-label">{ui(locale,'Résumé visuel','Visual summary','视觉说明')}</span><textarea class="textarea" value={brand.visual_summary||''} oninput={(e)=>setField('visual_summary',(e.currentTarget as HTMLTextAreaElement).value)}></textarea></label>
      </div>
    </section>

    <section class="panel brand-section">
      <div class="panel-head"><div><strong>Logos</strong></div></div>
      <div class="brand-form">
        {#each ['primary','secondary'] as slot}
          <div class="brand-map-row logo-row">
            <strong>{slot==='primary'?ui(locale,'Principal','Primary','主 Logo'):ui(locale,'Secondaire','Secondary','次 Logo')}</strong>
            <input class="input mono" value={brand.logos?.[slot]||''} oninput={(e)=>setMap('logos',slot,(e.currentTarget as HTMLInputElement).value)} />
            <label class="btn compact file-btn">{uploading===slot?ui(locale,'Import…','Uploading…','上传中…'):ui(locale,'Importer','Import','导入')}<input type="file" accept="image/svg+xml,image/png,image/jpeg,image/webp" onchange={(e)=>uploadLogo(slot,e)} /></label>
          </div>
        {/each}
      </div>
    </section>

    <section class="panel brand-section">
      <div class="panel-head"><div><strong>{ui(locale,'Palette','Palette','调色板')}</strong></div><button class="btn compact" onclick={addPalette}>+ {ui(locale,'Couleur','Color','颜色')}</button></div>
      <div class="brand-form">
        {#each Object.entries(brand.palette||{}) as [key,value] (key)}
          <div class="brand-map-row"><input class="input mono" value={key} readonly /><input type="color" value={value} oninput={(e)=>setMap('palette',key,(e.currentTarget as HTMLInputElement).value)} /><input class="input mono" value={value} oninput={(e)=>setMap('palette',key,(e.currentTarget as HTMLInputElement).value)} /><button class="tiny danger" onclick={()=>removeMap('palette',key)}>×</button></div>
        {/each}
      </div>
    </section>

    <section class="panel brand-section">
      <div class="panel-head"><div><strong>{ui(locale,'Typographies','Typography','字体')}</strong></div><button class="btn compact" onclick={addFont}>+ {ui(locale,'Rôle','Role','角色')}</button></div>
      <div class="brand-form">
        <datalist id="brand-font-families">{#each families() as family}<option value={family}></option>{/each}</datalist>
        {#each Object.entries(brand.fonts||{}) as [key,value] (key)}
          <div class="brand-map-row"><input class="input mono" value={key} readonly /><input class="input" list="brand-font-families" value={value} oninput={(e)=>setMap('fonts',key,(e.currentTarget as HTMLInputElement).value)} /><button class="tiny danger" onclick={()=>removeMap('fonts',key)}>×</button></div>
        {/each}
      </div>
    </section>
  </div>
  {#if error}<div class="global-error">{error}</div>{/if}
</section>
