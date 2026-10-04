import { describe, expect, it } from 'vitest';
import { tr, translations, ui } from './i18n';

describe('i18n', () => {
  it('returns one language at a time for primary labels', () => {
    expect(tr('fr','nav.project')).toBe('Projet');
    expect(tr('en','nav.project')).toBe('Project');
    expect(tr('zh-CN','nav.project')).toBe('项目');
    expect(tr('fr','action.exportAll')).toBe('Exporter tout');
    expect(tr('en','action.exportAll')).toBe('Export all');
    expect(tr('zh-CN','action.exportAll')).toBe('全部导出');
  });

  it('has the exact same key set in all locales', () => {
    const fr = Object.keys(translations.fr).sort();
    expect(Object.keys(translations.en).sort()).toEqual(fr);
    expect(Object.keys(translations['zh-CN']).sort()).toEqual(fr);
  });

  it('selects exactly one branch for ad-hoc UI copy', () => {
    expect(ui('fr','Bonjour','Hello','你好')).toBe('Bonjour');
    expect(ui('en','Bonjour','Hello','你好')).toBe('Hello');
    expect(ui('zh-CN','Bonjour','Hello','你好')).toBe('你好');
  });

  it('covers navigation and create workflow in FR, EN and zh-CN', () => {
    const expected = {
      fr: {
        'nav.project':'Projet',
        'nav.templates':'Modèles',
        'nav.library':'Bibliothèque',
        'nav.brand':'Kit de marque',
        'nav.exports':'Exports',
        'nav.api':'API',
        'action.import':'Importer',
        'action.smartCrop':'Recadrage intelligent',
        'action.exportSelected':'Exporter la sélection',
        'action.exportAll':'Exporter tout',
        'copy.tagline':'Une image, tous les formats.'
      },
      en: {
        'nav.project':'Project',
        'nav.templates':'Templates',
        'nav.library':'Library',
        'nav.brand':'Brand kit',
        'nav.exports':'Exports',
        'nav.api':'API',
        'action.import':'Import',
        'action.smartCrop':'Smart Reframe',
        'action.exportSelected':'Export selected',
        'action.exportAll':'Export all',
        'copy.tagline':'One image, every format.'
      },
      'zh-CN': {
        'nav.project':'项目',
        'nav.templates':'模板',
        'nav.library':'素材库',
        'nav.brand':'品牌工具包',
        'nav.exports':'导出',
        'nav.api':'API',
        'action.import':'导入',
        'action.smartCrop':'智能裁切',
        'action.exportSelected':'导出所选',
        'action.exportAll':'全部导出',
        'copy.tagline':'一张图，多种格式。'
      }
    } as const;

    for (const locale of ['fr','en','zh-CN'] as const) {
      const table = translations[locale] as Record<string,string>;
      for (const [key,value] of Object.entries(expected[locale])) {
        expect(table[key], locale + ':' + key).toBe(value);
      }
    }
  });
});
