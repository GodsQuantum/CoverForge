import { describe, expect, it } from 'vitest';
import type { TemplateData } from './types';
import { buildPackageRequest } from './export-state';

const template:TemplateData = {
  version:1,
  id:'starter-brand',
  canvas:{width:100,height:100,background:'#fff'},
  variants:{},
  layers:[],
  brand:{},
  formats:{
    youtube:{canvas:{width:1920,height:1080,background:'#fff'},layers:[]},
    square:{canvas:{width:1080,height:1080,background:'#fff'},layers:[]}
  }
};

describe('package export request', () => {
  it('uses saved template id and only selected formats when clean', () => {
    const request = buildPackageRequest({
      templateId:'starter-brand',
      template,
      dirty:false,
      variables:{title:'Hello'},
      selectedFormats:['square']
    });
    expect(request.template).toBe('starter-brand');
    expect(request.inline_template).toBeUndefined();
    expect(request.variants).toEqual(['square']);
  });

  it('uses inline template and no saved id when dirty', () => {
    const request = buildPackageRequest({
      templateId:'starter-brand',
      template,
      dirty:true,
      variables:{title:'Changed'},
      selectedFormats:['youtube','square']
    });
    expect(request.template).toBeUndefined();
    expect(request.inline_template).toEqual(template);
    expect(request.variants).toEqual(['youtube','square']);
  });

  it('always sends exactly one template source', () => {
    for (const dirty of [false,true]) {
      const request = buildPackageRequest({
        templateId:'starter-brand',
        template,
        dirty,
        variables:{},
        selectedFormats:['youtube']
      });
      expect(Number(Boolean(request.template)) + Number(Boolean(request.inline_template))).toBe(1);
    }
  });
});
