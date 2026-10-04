import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { describe, expect, it } from 'vitest';

describe('CoverForge brand assets', () => {
  for (const relative of [
    '../brand/coverforge-logo.svg',
    '../brand/coverforge-mark.svg',
    'static/coverforge-logo.svg',
    'static/favicon.svg'
  ]) {
    it(relative + ' is vector-only and uses the canonical orange gradient', () => {
      const svg = readFileSync(resolve(process.cwd(), relative), 'utf8');
      expect(svg).toContain('#FF7A00');
      expect(svg).toContain('#FF4D00');
      expect(svg).not.toMatch(/<image\b|data:image|(?:href|src)=["']https?:\/\//i);
    });
  }
});
