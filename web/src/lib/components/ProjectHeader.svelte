<script lang="ts">
  import { tr, type Locale } from '../i18n';

  let {
    locale,
    template = ''
  }: {
    locale:Locale;
    template?:string;
  } = $props();
</script>

<section class="product-hero">
  <div class="product-hero-copy">
    <span class="product-kicker">BRANDING AUTOMATION / 品牌自动化</span>
    <h1>{tr(locale,'copy.tagline')}</h1>
    <p>{tr(locale,'copy.product')}</p>
    <div class="product-flow" aria-label="Workflow">
      <span>Image</span><b>→</b><span>Smart crop</span><b>→</b><span>Brand kit</span><b>→</b><span>Multi-format</span><b>→</b><span>API / ZIP</span>
    </div>
  </div>
  <div class="product-hero-meta">
    <span class="hero-chip">API-first</span>
    <span class="hero-chip">Self-hosted</span>
    <span class="hero-chip">Rust renderer</span>
    {#if template}<span class="hero-template">{template}</span>{/if}
  </div>
</section>
