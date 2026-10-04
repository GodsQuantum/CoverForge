import type {
  FontRecord,
  InlineRenderRequest,
  ReframeRequest,
  ReframeResult,
  RenderRequest,
  RenderResponse,
  TemplateData,
  UploadedAsset
} from './types';

async function json<T>(input:RequestInfo | URL, init?:RequestInit):Promise<T> {
  const res = await fetch(input, init);
  const data = await res.json().catch(() => ({}));
  if (!res.ok || (data && typeof data === 'object' && 'ok' in data && data.ok === false)) {
    throw new Error((data as any)?.error || res.statusText || 'CoverForge API error');
  }
  return data as T;
}

export const api = {
  async listTemplates():Promise<string[]> {
    const data = await json<{templates:string[]}>('/v1/templates');
    return Array.isArray(data.templates) ? data.templates : [];
  },
  getTemplate(id:string):Promise<TemplateData> {
    return json<TemplateData>('/v1/templates/' + encodeURIComponent(id));
  },
  async putTemplate(id:string, template:TemplateData):Promise<void> {
    await json('/v1/templates/' + encodeURIComponent(id), {
      method:'PUT',
      headers:{'content-type':'application/json'},
      body:JSON.stringify(template)
    });
  },
  async listFonts():Promise<FontRecord[]> {
    const data = await json<{fonts:FontRecord[]}>('/v1/fonts');
    return Array.isArray(data.fonts) ? data.fonts : [];
  },
  async uploadAsset(file:File):Promise<UploadedAsset> {
    const body = new FormData();
    body.append('asset', file);
    return json<UploadedAsset>('/v1/assets', {method:'POST', body});
  },
  reframe(request:ReframeRequest):Promise<ReframeResult[]> {
    return json<ReframeResult[]>('/v1/reframe', {
      method:'POST',
      headers:{'content-type':'application/json'},
      body:JSON.stringify(request)
    });
  },
  render(request:RenderRequest):Promise<RenderResponse> {
    return json<RenderResponse>('/v1/render', {
      method:'POST',
      headers:{'content-type':'application/json'},
      body:JSON.stringify(request)
    });
  },
  renderPreview(request:InlineRenderRequest):Promise<RenderResponse> {
    return json<RenderResponse>('/v1/render/preview', {
      method:'POST',
      headers:{'content-type':'application/json'},
      body:JSON.stringify(request)
    });
  }
};
