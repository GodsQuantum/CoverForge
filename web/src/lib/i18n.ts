export type Locale = 'fr' | 'zh-CN';

export const translations = {
  fr: {
    'nav.project':'Projet',
    'nav.templates':'Modèles',
    'nav.library':'Bibliothèque',
    'nav.brand':'Kit de marque',
    'nav.exports':'Exports',
    'nav.api':'API',
    'action.import':'Importer',
    'action.replaceImage':'Remplacer l’image',
    'action.smartCrop':'Recadrage intelligent',
    'action.resetCrop':'Réinitialiser le recadrage',
    'action.layers':'Calques',
    'action.formats':'Formats',
    'action.exportSelected':'Exporter la sélection',
    'action.exportAll':'Exporter tout',
    'action.save':'Sauvegarder',
    'action.render':'Rendre',
    'status.loading':'Chargement…',
    'status.error':'Erreur',
    'label.brandKit':'Kit de marque',
    'label.templates':'Modèles',
    'label.library':'Bibliothèque',
    'label.api':'API',
    'label.sourceImage':'Image source',
    'label.smartReframe':'Recadrage intelligent',
    'label.title':'Titre',
    'label.subtitle':'Sous-titre',
    'label.logo':'Logo'
  },
  'zh-CN': {
    'nav.project':'项目',
    'nav.templates':'模板',
    'nav.library':'素材库',
    'nav.brand':'品牌工具包',
    'nav.exports':'导出',
    'nav.api':'API',
    'action.import':'导入',
    'action.replaceImage':'替换图片',
    'action.smartCrop':'智能裁切',
    'action.resetCrop':'重置裁切',
    'action.layers':'图层',
    'action.formats':'格式',
    'action.exportSelected':'导出所选',
    'action.exportAll':'全部导出',
    'action.save':'保存',
    'action.render':'渲染',
    'status.loading':'加载中…',
    'status.error':'错误',
    'label.brandKit':'品牌工具包',
    'label.templates':'模板',
    'label.library':'素材库',
    'label.api':'API',
    'label.sourceImage':'原始图片',
    'label.smartReframe':'智能裁切',
    'label.title':'标题',
    'label.subtitle':'副标题',
    'label.logo':'Logo'
  }
} as const;

export type TranslationKey = keyof typeof translations.fr;

export function tr(locale:Locale, key:TranslationKey):string {
  return translations[locale][key];
}
