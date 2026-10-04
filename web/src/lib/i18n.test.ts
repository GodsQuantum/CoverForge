import { describe, expect, it } from 'vitest';
import { tr } from './i18n';

describe('i18n', () => {
  it('returns French and Simplified Chinese primary labels', () => {
    expect(tr('fr','nav.project')).toBe('Projet');
    expect(tr('zh-CN','nav.project')).toBe('项目');
    expect(tr('fr','action.exportAll')).toBe('Exporter tout');
    expect(tr('zh-CN','action.exportAll')).toBe('全部导出');
  });
});
