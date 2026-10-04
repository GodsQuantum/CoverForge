import { describe, expect, it } from 'vitest';
import { tr, translations } from './i18n';

describe('i18n', () => {
  it('returns French and Simplified Chinese primary labels', () => {
    expect(tr('fr','nav.project')).toBe('Projet');
    expect(tr('zh-CN','nav.project')).toBe('项目');
    expect(tr('fr','action.exportAll')).toBe('Exporter tout');
    expect(tr('zh-CN','action.exportAll')).toBe('全部导出');
  });

  it('covers the complete product navigation and create workflow', () => {
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

    for (const locale of ['fr','zh-CN'] as const) {
      const table = translations[locale] as Record<string,string>;
      for (const [key,value] of Object.entries(expected[locale])) {
        expect(table[key], locale + ':' + key).toBe(value);
      }
    }
  });
});
