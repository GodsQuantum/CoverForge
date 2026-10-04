import { describe, expect, it } from 'vitest';
import type { TemplateData } from './types';
import { cloneTemplateForId } from './template-state';

const source:TemplateData = {
  version:1,
  id:'starter-brand',
  canvas:{width:100,height:100,background:'#fff'},
  variants:{},
  layers:[],
  brand:{display_name:'Starter',palette:{accent:'#FF7A00'}},
  formats:{
    square:{canvas:{width:1080,height:1080,background:'#fff'},layers:[]}
  }
};

describe('template clone state', () => {
  it('deep-copies and changes only the id', () => {
    const clone = cloneTemplateForId(source,'my-brand');
    expect(clone).not.toBe(source);
    expect(clone.id).toBe('my-brand');
    expect(clone.brand).not.toBe(source.brand);
    expect(clone.formats).not.toBe(source.formats);
    expect({...clone,id:source.id}).toEqual(source);
  });

  it('rejects invalid ids', () => {
    for (const id of ['', 'bad id', '../escape', 'a/b', '..']) {
      expect(() => cloneTemplateForId(source,id)).toThrow();
    }
  });

  it('does not mutate the source', () => {
    const before = structuredClone(source);
    const clone = cloneTemplateForId(source,'copy_01');
    clone.brand!.palette!.accent = '#000000';
    expect(source).toEqual(before);
  });
});
