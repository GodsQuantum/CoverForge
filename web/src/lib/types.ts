export type LegacyView = 'composer' | 'fonts' | 'json';
export type ProductView = 'project' | 'templates' | 'library' | 'brand' | 'exports' | 'api';
export type View = LegacyView | ProductView;

export type Frame = { x:number; y:number; width:number; height:number };
export type CanvasDef = { width:number; height:number; background:string };

export type Layer = {
  type:'text'|'image'|'rect';
  id:string;
  name?:string;
  visible?:boolean;
  locked?:boolean;
  frame:Frame;
  [key:string]:any;
};

export type FormatDef = {
  canvas:CanvasDef;
  layers:Layer[];
  label?:string;
  category?:string;
  platform?:string;
  suffix?:string;
};

export type BrandStyle = {
  display_name?:string;
  visual_summary?:string;
  palette?:Record<string,string>;
  fonts?:Record<string,string>;
  logos?:Record<string,string>;
  notes?:string[];
};

export type TemplateData = {
  version:number;
  id:string;
  canvas:CanvasDef;
  variants:Record<string,CanvasDef>;
  layers:Layer[];
  brand?:BrandStyle;
  formats:Record<string,FormatDef>;
};

export type RenderedAsset = {
  variant:string;
  width:number;
  height:number;
  filename:string;
  path?:string;
  url:string;
};

export type RenderResponse = {
  ok:boolean;
  template:string;
  assets:RenderedAsset[];
  error?:string;
};

export type FontRecord = {
  family:string;
  style:string;
  weight?:number;
  font_style?:string;
  filename:string;
  source:string;
  url:string;
};

export type RenderRequest = {
  template:string;
  variables?:Record<string,string>;
  variants?:string[];
  output_stem?:string;
};

export type InlineRenderRequest = {
  template:TemplateData;
  variables?:Record<string,string>;
  variants?:string[];
  output_stem?:string;
};

export type UploadedAsset = {
  id:string;
  filename:string;
  path:string;
  url:string;
  mime:string;
  width:number;
  height:number;
  kind:'image'|'logo'|string;
};

export type Point = { x:number; y:number };

export type ReframeTarget = {
  id:string;
  width:number;
  height:number;
};

export type ReframeRequest = {
  asset:string;
  formats:ReframeTarget[];
  focal_override?:Point|null;
};

export type ReframeResult = {
  format:string;
  crop:Frame;
  focal:Point;
  score:number;
};
