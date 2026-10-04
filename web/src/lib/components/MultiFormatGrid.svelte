<script lang="ts">
  import FormatCard from './FormatCard.svelte';
  import type { ReframeResult, TemplateData } from '../types';

  let {
    template,
    sourceUrl = '',
    selectedFormats,
    activeFormat,
    results = [],
    modes = {},
    onToggle,
    onOpen,
    onResetAuto
  }: {
    template:TemplateData;
    sourceUrl?:string;
    selectedFormats:Set<string>;
    activeFormat:string;
    results?:ReframeResult[];
    modes?:Record<string,'auto'|'manual'>;
    onToggle:(id:string)=>void;
    onOpen:(id:string)=>void;
    onResetAuto:(id:string)=>void|Promise<void>;
  } = $props();

  const priority = ['youtube','square','feed','vertical','landscape'];

  function ids():string[] {
    const available = Object.keys(template.formats || {});
    return [...available].sort((a,b) => {
      const ai = priority.indexOf(a);
      const bi = priority.indexOf(b);
      if (ai === -1 && bi === -1) return a.localeCompare(b);
      if (ai === -1) return 1;
      if (bi === -1) return -1;
      return ai-bi;
    });
  }

  function resultFor(id:string) {
    return results.find((entry) => entry.format === id) || null;
  }
</script>

<section class="multi-format-panel panel">
  <div class="multi-format-head">
    <div><span class="eyebrow">MULTI-FORMAT / 多格式</span><strong>Tous les formats / 全部格式</strong></div>
    <span>{selectedFormats.size}/{ids().length}</span>
  </div>
  <div class="format-grid">
    {#each ids() as id (id)}
      <FormatCard
        {id}
        definition={template.formats[id]}
        selected={selectedFormats.has(id)}
        active={activeFormat===id}
        {sourceUrl}
        result={resultFor(id)}
        mode={modes[id] || 'auto'}
        {onToggle}
        {onOpen}
        {onResetAuto}
      />
    {/each}
  </div>
</section>
