import type { ReframeResult, TemplateData } from './types';

export function toggleFormatSelection(selected:Set<string>, id:string):Set<string> {
  const next = new Set(selected);
  if (next.has(id)) next.delete(id);
  else next.add(id);
  return next;
}

export function normalizeSelectedFormats(
  selected:Set<string>,
  available:string[],
  active:string
):string[] {
  const known = new Set(available);
  const normalized = [...selected].filter((id) => known.has(id));
  if (normalized.length) return normalized;
  if (known.has(active)) return [active];
  return available.length ? [available[0]] : [];
}

export function applyReframeToTemplate(
  template:TemplateData,
  results:ReframeResult[]
):TemplateData {
  const next:TemplateData = JSON.parse(JSON.stringify(template));
  for (const result of results) {
    const format = next.formats?.[result.format];
    if (!format) continue;
    for (const layer of format.layers || []) {
      if (layer.type !== 'image') continue;
      const source = String(layer.source || '');
      if (!source.includes('{{background}}') && layer.id !== 'background') continue;
      layer.focal_x = result.focal.x;
      layer.focal_y = result.focal.y;
    }
  }
  return next;
}
