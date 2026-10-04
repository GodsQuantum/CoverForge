<script lang="ts">
  import type { FormatDef, ReframeResult } from '../types';

  let {
    id,
    definition,
    selected,
    active,
    sourceUrl = '',
    result = null,
    mode = 'auto',
    onToggle,
    onOpen,
    onResetAuto
  }: {
    id:string;
    definition:FormatDef;
    selected:boolean;
    active:boolean;
    sourceUrl?:string;
    result?:ReframeResult|null;
    mode?:'auto'|'manual';
    onToggle:(id:string)=>void;
    onOpen:(id:string)=>void;
    onResetAuto:(id:string)=>void|Promise<void>;
  } = $props();

  function titleCase(value:string) {
    return value.replace(/[-_]+/g,' ').replace(/\b\w/g,(m)=>m.toUpperCase());
  }

  let label = $derived(definition.label || titleCase(id));
  let aspect = $derived(definition.canvas.width + ' / ' + definition.canvas.height);
  let focal = $derived(result?.focal || {x:0.5,y:0.5});
</script>

<div
  role="button"
  tabindex="0"
  class:active
  class:selected
  class="format-card"
  onclick={() => onOpen(id)}
  onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); onOpen(id); } }}
>
  <div class="format-card-top">
    <label class="format-check">
      <input type="checkbox" checked={selected} onclick={(e)=>e.stopPropagation()} onchange={() => onToggle(id)} />
      <span>{label}</span>
    </label>
    <span class="format-dims">{definition.canvas.width}×{definition.canvas.height}</span>
  </div>

  <div class="format-thumb" style:aspect-ratio={aspect}>
    {#if sourceUrl}
      <img src={sourceUrl} alt="" style:object-position={(focal.x*100)+'% '+(focal.y*100)+'%'} />
    {:else}
      <div class="format-empty"><img src="/favicon.svg" alt="" /></div>
    {/if}
    <span class="format-focus" style:left={(focal.x*100)+'%'} style:top={(focal.y*100)+'%'}></span>
  </div>

  <div class="format-card-bottom">
    <span>{definition.platform || definition.category || 'Brand'}</span>
    {#if mode === 'manual'}
      <button class="format-mode manual" onclick={(e)=>{e.stopPropagation();void onResetAuto(id)}}>Manuel → Auto</button>
    {:else}
      <span class="format-mode">Auto</span>
    {/if}
  </div>
</div>
