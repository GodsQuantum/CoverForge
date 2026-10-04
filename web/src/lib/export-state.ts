import type { PackageRenderRequest, TemplateData } from './types';

export function buildPackageRequest(input:{
  templateId:string;
  template:TemplateData;
  dirty:boolean;
  variables:Record<string,string>;
  selectedFormats:string[];
  outputStem?:string;
}):PackageRenderRequest {
  const base = {
    variables:{...input.variables},
    variants:[...input.selectedFormats],
    ...(input.outputStem ? {output_stem:input.outputStem} : {})
  };
  if (input.dirty) {
    return {...base, inline_template:input.template};
  }
  return {...base, template:input.templateId};
}
