import { describe, expect, it } from 'vitest';
import type { ReframeResult, TemplateData } from './types';
import { applyReframeToTemplate, normalizeSelectedFormats, toggleFormatSelection } from './format-state';

function template():TemplateData {
  const image = (id:string) => ({
    type:'image' as const,
    id,
    frame:{x:0,y:0,width:1,height:1},
    source:'{{background}}',
    fit:'cover',
    focal_x:0.5,
    focal_y:0.5
  });
  return {
    version:1,
    id:'starter',
    canvas:{width:100,height:100,background:'#000'},
    variants:{},
    layers:[],
    brand:{},
    formats:{
      youtube:{canvas:{width:1920,height:1080,background:'#000'},layers:[image('bg-y')]},
      square:{canvas:{width:1080,height:1080,background:'#000'},layers:[image('bg-s')]}
    }
  };
}

describe('format state', () => {
  it('toggles selection while preserving insertion order', () => {
    const selected = new Set(['youtube','square']);
    const removed = toggleFormatSelection(selected,'youtube');
    expect([...removed]).toEqual(['square']);
    expect([...selected]).toEqual(['youtube','square']);
    const restored = toggleFormatSelection(removed,'youtube');
    expect([...restored]).toEqual(['square','youtube']);
  });

  it('applies focal recommendations only to matching background layers and ignores unknown formats', () => {
    const source = template();
    const results:ReframeResult[] = [
      {format:'square',crop:{x:0.2,y:0,width:0.6,height:1},focal:{x:0.8,y:0.2},score:0.9},
      {format:'missing',crop:{x:0,y:0,width:1,height:1},focal:{x:0.1,y:0.1},score:0.5}
    ];
    const next = applyReframeToTemplate(source,results);
    const square = next.formats.square.layers[0];
    const youtube = next.formats.youtube.layers[0];
    expect(square.focal_x).toBe(0.8);
    expect(square.focal_y).toBe(0.2);
    expect(youtube.focal_x).toBe(0.5);
    expect(youtube.focal_y).toBe(0.5);
    expect(source.formats.square.layers[0].focal_x).toBe(0.5);
  });

  it('normalizes export selection to known formats and falls back to active format', () => {
    expect(normalizeSelectedFormats(new Set(),['youtube','square'],'square')).toEqual(['square']);
    expect(normalizeSelectedFormats(new Set(['unknown','youtube']),['youtube','square'],'square')).toEqual(['youtube']);
  });
});
