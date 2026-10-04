import type { TemplateData } from './types';

const TEMPLATE_ID = /^[A-Za-z0-9_-]+$/;

export function cloneTemplateForId(template:TemplateData, id:string):TemplateData {
  if (!id || !TEMPLATE_ID.test(id)) throw new Error('Template id must use A-Z, a-z, 0-9, - or _');
  const clone = structuredClone(template);
  clone.id = id;
  return clone;
}
