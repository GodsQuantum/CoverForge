<script lang="ts">
  import { tr, ui, type Locale } from '../i18n';

  let { locale, template = '' }:{ locale:Locale; template?:string } = $props();
</script>

<section class="product-hero">
  <div class="product-hero-copy">
    <span class="product-kicker">{ui(locale,'AUTOMATISATION DE MARQUE','BRANDING AUTOMATION','品牌自动化')}</span>
    <h1>{tr(locale,'copy.tagline')}</h1>
    <p>{tr(locale,'copy.product')}</p>
    <div class="product-flow" aria-label={ui(locale,'Flux de travail','Workflow','工作流')}>
      <span>{ui(locale,'Image','Image','图片')}</span><b>→</b>
      <span>{ui(locale,'Recadrage intelligent','Smart Reframe','智能裁切')}</span><b>→</b>
      <span>{ui(locale,'Kit de marque','Brand kit','品牌工具包')}</span><b>→</b>
      <span>{ui(locale,'Multi-format','Multi-format','多格式')}</span><b>→</b>
      <span>API / ZIP</span>
    </div>
  </div>
  <div class="product-hero-meta">
    <span class="hero-chip">API-first</span>
    <span class="hero-chip">{ui(locale,'Auto-hébergé','Self-hosted','自托管')}</span>
    <span class="hero-chip">{ui(locale,'Rendu Rust','Rust renderer','Rust 渲染')}</span>
    {#if template}<span class="hero-template">{template}</span>{/if}
  </div>
</section>
