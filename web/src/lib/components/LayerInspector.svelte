<script lang="ts">
  import { ui, type Locale } from '../i18n';
  import type { FontRecord, Frame, Layer } from '../types';

  let {
    locale='fr',
    layer = null,
    fonts = [],
    onValue,
    onFramePercent,
    onScalarPercent,
    onToggleVisible,
    onToggleLocked,
    onFontFamily,
    onFontFace
  }: {
    locale?:Locale;
    layer?:Layer|null;
    fonts?:FontRecord[];
    onValue:(key:string,value:any,rebuild?:boolean)=>void;
    onFramePercent:(key:keyof Frame,value:number)=>void;
    onScalarPercent:(key:string,value:number)=>void;
    onToggleVisible:(layer:Layer)=>void;
    onToggleLocked:(layer:Layer)=>void;
    onFontFamily:(family:string)=>void|Promise<void>;
    onFontFace:(key:string)=>void|Promise<void>;
  } = $props();

  const scalarPercent = (value:number) => Math.round(Number(value || 0)*1000)/10;
  const framePercent = (key:keyof Frame) => layer ? Math.round(Number(layer.frame[key] || 0)*1000)/10 : 0;

  function fontWeight(f:FontRecord):number {
    if (Number.isFinite(f.weight)) return Number(f.weight);
    const style=String(f.style||'').toLowerCase().replace(/[\s_-]+/g,'');
    if(style.includes('thin'))return 100;if(style.includes('extralight')||style.includes('ultralight'))return 200;
    if(style.includes('light'))return 300;if(style.includes('medium'))return 500;
    if(style.includes('semibold')||style.includes('demibold'))return 600;if(style.includes('bold'))return style.includes('extra')?800:700;
    if(style.includes('black')||style.includes('heavy'))return 900;return 400;
  }
  function fontStyle(f:FontRecord){return f.font_style || (/italic|oblique/i.test(f.style||'')?'italic':'normal')}
  function fontFaceKey(f:FontRecord){return [f.family,f.style||'Regular',fontWeight(f),fontStyle(f)].join('|')}
  function faces(family:string){
    const seen=new Set<string>();
    return fonts.filter(f=>f.family===family).sort((a,b)=>fontWeight(a)-fontWeight(b)).filter(f=>{
      const key=[f.style,fontWeight(f),fontStyle(f)].join('|'); if(seen.has(key))return false;seen.add(key);return true;
    });
  }
  function families(){return [...new Set(fonts.map(f=>f.family).filter(Boolean))].sort((a,b)=>a.localeCompare(b))}
  function currentFaceKey(){
    if(!layer || layer.type!=='text')return '';
    const list=faces(layer.font_family||'');
    return fontFaceKey(list.find(f=>fontWeight(f)===Number(layer.font_weight||400)&&fontStyle(f)===String(layer.font_style||'normal'))||list[0]||{family:'',style:'',filename:'',source:'',url:''});
  }
</script>

<aside class="inspector panel">
  <div class="panel-head"><div><strong>{ui(locale,'Propriétés','Properties','属性')}</strong><span>{layer?.type || ui(locale,'aucun calque','no layer','未选择图层')}</span></div></div>
  {#if layer}
    <div class="inspector-body">
      <div class="field"><span class="field-label">{ui(locale,'Nom','Name','名称')}</span><input class="input" value={layer.name||''} oninput={(e)=>onValue('name',(e.currentTarget as HTMLInputElement).value,false)} /></div>
      <div class="field"><span class="field-label">ID / API</span><input class="input mono" value={layer.id} readonly /></div>

      <div class="subhead">{ui(locale,'Position et taille','Position & size','位置与尺寸')}</div>
      <div class="range-grid">
        {#each [['x','X',-50,150],['y','Y',-50,150],['width','L',1,200],['height','H',1,200]] as item}
          {@const key=item[0] as keyof Frame}
          <label class="range-field">
            <span>{item[1]}</span>
            <input type="range" min={item[2]} max={item[3]} step="0.1" value={framePercent(key)} oninput={(e)=>onFramePercent(key,+(e.currentTarget as HTMLInputElement).value)} />
            <input class="range-number" type="number" min={key==='x'||key==='y'?-100:.1} max="200" step="0.1" value={framePercent(key)} oninput={(e)=>onFramePercent(key,+(e.currentTarget as HTMLInputElement).value)} />
            <em>%</em>
          </label>
        {/each}
      </div>

      <div class="toggle-row">
        <label><input type="checkbox" checked={layer.visible!==false} onchange={()=>onToggleVisible(layer)} /> {ui(locale,'Visible','Visible','显示')}</label>
        <label><input type="checkbox" checked={layer.locked===true} onchange={()=>onToggleLocked(layer)} /> {ui(locale,'Verrouillé','Locked','锁定')}</label>
      </div>

      {#if layer.type==='text'}
        <div class="subhead">{ui(locale,'Texte','Text','文本')}</div>
        <div class="field"><span class="field-label">{ui(locale,'Contenu','Content','内容')}</span><textarea class="textarea compact-area" value={layer.text||''} oninput={(e)=>onValue('text',(e.currentTarget as HTMLTextAreaElement).value)}></textarea></div>
        <div class="grid two font-pickers">
          <label class="mini-field"><span>{ui(locale,'Famille','Family','字体')}</span><input class="input" list="inspector-font-families" value={layer.font_family||''} onchange={(e)=>void onFontFamily((e.currentTarget as HTMLInputElement).value)} /><datalist id="inspector-font-families">{#each families() as family}<option value={family}></option>{/each}</datalist></label>
          <label class="mini-field"><span>{ui(locale,'Variante','Variant','字重')}</span><select value={currentFaceKey()} onchange={(e)=>void onFontFace((e.currentTarget as HTMLSelectElement).value)}>{#each faces(layer.font_family||'') as face}<option value={fontFaceKey(face)}>{face.style||'Regular'} · {fontWeight(face)}{fontStyle(face)==='italic'?' · italic':''}</option>{/each}</select></label>
        </div>
        <label class="range-field"><span>{ui(locale,'Taille','Size','字号')}</span><input type="range" min=".5" max="30" step=".1" value={scalarPercent(layer.font_size||.06)} oninput={(e)=>onScalarPercent('font_size',+(e.currentTarget as HTMLInputElement).value)} /><input class="range-number" type="number" min=".1" max="50" step=".1" value={scalarPercent(layer.font_size||.06)} oninput={(e)=>onScalarPercent('font_size',+(e.currentTarget as HTMLInputElement).value)} /><em>% H</em></label>
        <div class="toggle-row"><label><input type="checkbox" checked={layer.auto_fit===true} onchange={(e)=>onValue('auto_fit',(e.currentTarget as HTMLInputElement).checked)} /> Auto-fit</label><label><input type="checkbox" checked={layer.uppercase===true} onchange={(e)=>onValue('uppercase',(e.currentTarget as HTMLInputElement).checked)} /> {ui(locale,'Capitales','Uppercase','大写')}</label></div>
        {#if layer.auto_fit}
          <label class="range-field"><span>{ui(locale,'Mini','Min','最小')}</span><input type="range" min=".2" max="20" step=".1" value={scalarPercent(layer.min_font_size||.015)} oninput={(e)=>onScalarPercent('min_font_size',+(e.currentTarget as HTMLInputElement).value)} /><input class="range-number" type="number" min=".1" max="30" step=".1" value={scalarPercent(layer.min_font_size||.015)} oninput={(e)=>onScalarPercent('min_font_size',+(e.currentTarget as HTMLInputElement).value)} /><em>% H</em></label>
          <label class="range-field"><span>{ui(locale,'Lignes','Lines','行数')}</span><input type="range" min="1" max="12" step="1" value={layer.max_lines||4} oninput={(e)=>onValue('max_lines',+(e.currentTarget as HTMLInputElement).value)} /><input class="range-number" type="number" min="1" max="20" step="1" value={layer.max_lines||4} oninput={(e)=>onValue('max_lines',+(e.currentTarget as HTMLInputElement).value)} /><em>max</em></label>
        {/if}
        <div class="grid two">
          <label class="mini-field"><span>{ui(locale,'Alignement','Alignment','对齐')}</span><select value={layer.align||'left'} onchange={(e)=>onValue('align',(e.currentTarget as HTMLSelectElement).value)}><option value="left">{ui(locale,'Gauche','Left','左')}</option><option value="center">{ui(locale,'Centre','Center','中')}</option><option value="right">{ui(locale,'Droite','Right','右')}</option></select></label>
          <label class="mini-field"><span>{ui(locale,'Couleur','Color','颜色')}</span><input type="color" value={layer.color||'#ffffff'} oninput={(e)=>onValue('color',(e.currentTarget as HTMLInputElement).value)} /></label>
        </div>
        <details class="advanced-block"><summary>{ui(locale,'Typographie avancée','Advanced typography','高级排版')}</summary><div class="advanced-content">
          <div class="grid two"><label class="mini-field"><span>{ui(locale,'Graisse CSS','CSS weight','CSS 字重')}</span><input type="number" min="100" max="1000" step="10" value={layer.font_weight||700} onchange={(e)=>onValue('font_weight',+(e.currentTarget as HTMLInputElement).value)} /></label><label class="mini-field"><span>{ui(locale,'Interligne','Line height','行高')}</span><input type="number" min=".5" max="3" step=".05" value={layer.line_height||1} onchange={(e)=>onValue('line_height',+(e.currentTarget as HTMLInputElement).value)} /></label></div>
          <label class="range-field"><span>{ui(locale,'Rotation','Rotation','旋转')}</span><input type="range" min="-180" max="180" step="1" value={layer.rotation_deg||0} oninput={(e)=>onValue('rotation_deg',+(e.currentTarget as HTMLInputElement).value)} /><input class="range-number" type="number" min="-360" max="360" step="1" value={layer.rotation_deg||0} oninput={(e)=>onValue('rotation_deg',+(e.currentTarget as HTMLInputElement).value)} /><em>°</em></label>
          <label class="range-field"><span>{ui(locale,'Opacité','Opacity','不透明度')}</span><input type="range" min="0" max="100" step="1" value={scalarPercent(layer.opacity??1)} oninput={(e)=>onScalarPercent('opacity',+(e.currentTarget as HTMLInputElement).value)} /><input class="range-number" type="number" min="0" max="100" step="1" value={scalarPercent(layer.opacity??1)} oninput={(e)=>onScalarPercent('opacity',+(e.currentTarget as HTMLInputElement).value)} /><em>%</em></label>
          <div class="grid two"><label class="mini-field"><span>{ui(locale,'Contour','Stroke','描边')}</span><input type="color" value={layer.stroke_color||'#000000'} oninput={(e)=>onValue('stroke_color',(e.currentTarget as HTMLInputElement).value)} /></label><label class="mini-field"><span>{ui(locale,'Épaisseur','Width','粗细')}</span><input type="number" min="0" max=".05" step=".0005" value={layer.stroke_width||0} onchange={(e)=>onValue('stroke_width',+(e.currentTarget as HTMLInputElement).value)} /></label></div>
        </div></details>
      {:else if layer.type==='image'}
        <div class="subhead">{ui(locale,'Image','Image','图片')}</div>
        <div class="field"><span class="field-label">{ui(locale,'Source','Source','资源')}</span><input class="input mono" value={layer.source||''} oninput={(e)=>onValue('source',(e.currentTarget as HTMLInputElement).value)} /></div>
        <label class="mini-field"><span>{ui(locale,'Ajustement','Fit','适配')}</span><select value={layer.fit||'cover'} onchange={(e)=>onValue('fit',(e.currentTarget as HTMLSelectElement).value)}><option value="cover">Cover</option><option value="contain">Contain</option></select></label>
        {#each [['opacity',ui(locale,'Opacité','Opacity','不透明度'),layer.opacity??1],['focal_x','Focal X',layer.focal_x??.5],['focal_y','Focal Y',layer.focal_y??.5]] as row}
          <label class="range-field"><span>{row[1]}</span><input type="range" min="0" max="100" step="1" value={scalarPercent(Number(row[2]))} oninput={(e)=>onScalarPercent(String(row[0]),+(e.currentTarget as HTMLInputElement).value)} /><input class="range-number" type="number" min="0" max="100" step="1" value={scalarPercent(Number(row[2]))} oninput={(e)=>onScalarPercent(String(row[0]),+(e.currentTarget as HTMLInputElement).value)} /><em>%</em></label>
        {/each}
      {:else}
        <div class="subhead">{ui(locale,'Rectangle','Rectangle','矩形')}</div>
        <label class="mini-field"><span>{ui(locale,'Couleur','Color','颜色')}</span><input type="color" value={layer.fill||'#111111'} oninput={(e)=>onValue('fill',(e.currentTarget as HTMLInputElement).value)} /></label>
        <label class="range-field"><span>{ui(locale,'Opacité','Opacity','不透明度')}</span><input type="range" min="0" max="100" step="1" value={scalarPercent(layer.opacity??1)} oninput={(e)=>onScalarPercent('opacity',+(e.currentTarget as HTMLInputElement).value)} /><input class="range-number" type="number" min="0" max="100" step="1" value={scalarPercent(layer.opacity??1)} oninput={(e)=>onScalarPercent('opacity',+(e.currentTarget as HTMLInputElement).value)} /><em>%</em></label>
        <label class="range-field"><span>{ui(locale,'Rayon','Radius','圆角')}</span><input type="range" min="0" max="50" step=".5" value={scalarPercent(layer.radius||0)} oninput={(e)=>onScalarPercent('radius',+(e.currentTarget as HTMLInputElement).value)} /><input class="range-number" type="number" min="0" max="50" step=".5" value={scalarPercent(layer.radius||0)} oninput={(e)=>onScalarPercent('radius',+(e.currentTarget as HTMLInputElement).value)} /><em>%</em></label>
      {/if}
    </div>
  {:else}
    <div class="empty-inspector">{ui(locale,'Sélectionne un calque','Select a layer','请选择图层')}</div>
  {/if}
</aside>
