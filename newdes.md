<!DOCTYPE html>
<html lang="fr">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Nexus - Anonymous Marketplace</title>
    <meta name="description" content="Trade anonymously with complete control. Nexus combines Tor anonymity with advanced multisig escrow.">
    
    <!-- Preconnect to Google Fonts -->
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
    <link href="https://fonts.googleapis.com/css2?family=Inter:wght @reputation/target/debug/deps/wasm_bindgen_shared-e35b7741d4336a4c.wasm_bindgen_shared.c95d82bb200000c-cgu.0.rcgu.dwo;300;400;500&display=swap" rel="stylesheet">
    
    <!-- HTMX -->
    <script src="https://unpkg.com/htmx.org @1.9.10"></script>
    
    <!-- Lucide Icons (pour les icônes) -->
    <script src="https://unpkg.com/lucide @latest"></script>
    
    <link rel="stylesheet" href="/static/css/styles.css">
</head>
<body>
    <!-- Navigation Header -->
    <header class="header">
        <nav class="nav-container">
            <div class="nav-content">
                <!-- Left - Navigation -->
                <div class="nav-left">
                    <a href="/listings" class="nav-link">Listings</a>
                </div>

                <!-- Center - Logo -->
                <div class="nav-center">
                    <h1 class="logo">NEXUS</h1>
                </div>

                <!-- Right - Actions -->
                <div class="nav-right">
                    <button class="icon-button" aria-label="Cart">
                        <i data-lucide="shopping-cart"></i>
                    </button>
                    <button class="icon-button" aria-label="Profile">
                        <i data-lucide="user"></i>
                    </button>
                    <button class="icon-button" aria-label="Menu">
                        <i data-lucide="menu"></i>
                    </button>
                </div>
            </div>
        </nav>
    </header>

    <!-- Hero Section -->
    <section class="hero-section">
        <!-- Guide Lines -->
        <div class="guide-lines">
            <div class="guide-container">
                <div class="guide-line guide-1"></div>
                <div class="guide-line guide-2"></div>
                <div class="guide-line guide-3"></div>
                <div class="guide-line guide-4"></div>
                <div class="guide-line guide-5"></div>
            </div>
        </div>

        <!-- Main Hero Content -->
        <div class="hero-content">
            <div class="hero-grid">
                <!-- Title -->
                <div class="hero-title-col">
                    <h2 class="hero-title">
                        <span class="title-line">Trade</span>
                        <span class="title-line accent">anonymously.</span>
                        <span class="title-line">Keep control.</span>
                    </h2>
                </div>

                <!-- Description -->
                <div class="hero-description-col">
                    <div class="description-content">
                        <p class="description-text">
                            Notre plateforme combine l'anonymat de Tor avec un système d'entiercement (escrow) avancé. 
                            Vous gardez toujours le contrôle total de vos fonds. C'est la sécurité de niveau expert, rendue simple.
                        </p>
                        
                        <button class="btn-ghost" hx-get="/listings" hx-push-url="true">
                            Explorer les listings
                            <i data-lucide="arrow-right"></i>
                        </button>
                    </div>
                </div>
            </div>
        </div>

        <!-- Bottom Section -->
        <div class="hero-bottom">
            <div class="hero-grid">
                <!-- Bottom Text -->
                <div class="bottom-text-col">
                    <p class="bottom-text">
                        Chez NEXUS, nous offrons une plateforme décentralisée, de la transaction à l'entiercement sécurisé.
                    </p>
                </div>

                <!-- CTA Circle -->
                <div class="bottom-cta-col">
                    <button class="cta-circle">
                        <div class="circle-content">
                            <span class="circle-text">VOIR</span>
                        </div>
                        <div class="circle-glow"></div>
                    </button>
                </div>
            </div>
        </div>
    </section>

    <script>
        // Initialize Lucide icons
        lucide.createIcons();
    </script>
</body>
</html>
/* Reset & Base */
* {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
}

:root {
    --color-background: #1A1A1A;
    --color-foreground: #FFFFFF;
    --color-accent: #C9A445;
    --color-border: rgba(255, 255, 255, 0.1);
    --font-inter: 'Inter', -apple-system, BlinkMacSystemFont, sans-serif;
}

body {
    font-family: var(--font-inter);
    background-color: var(--color-background);
    color: var(--color-foreground);
    min-height: 100vh;
    position: relative;
    font-weight: 300;
}

/* Grain Texture Effect */
body::before {
    content: '';
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    opacity: 0.15;
    z-index: 0;
    pointer-events: none;
    background-image: 
        repeating-linear-gradient(0deg, transparent, transparent 2px, rgba(255,255,255,.03) 2px, rgba(255,255,255,.03) 4px),
        repeating-linear-gradient(90deg, transparent, transparent 2px, rgba(255,255,255,.03) 2px, rgba(255,255,255,.03) 4px);
}

/* Header Navigation */
.header {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    z-index: 50;
    border-bottom: 1px solid var(--color-border);
    backdrop-filter: blur(8px);
}

.nav-container {
    max-width: 1280px;
    margin: 0 auto;
    padding: 1rem 1.5rem;
}

.nav-content {
    display: flex;
    align-items: center;
    justify-content: space-between;
    position: relative;
}

.nav-left, .nav-right {
    display: flex;
    align-items: center;
    gap: 2rem;
}

.nav-center {
    position: absolute;
    left: 50%;
    transform: translateX(-50%);
}

.nav-link {
    font-size: 0.875rem;
    font-weight: 300;
    color: var(--color-foreground);
    text-decoration: none;
    transition: color 0.3s;
}

.nav-link:hover {
    color: var(--color-accent);
}

.logo {
    font-size: 1.25rem;
    font-weight: 300;
    letter-spacing: 0.3em;
    color: var(--color-foreground);
}

.icon-button {
    background: transparent;
    border: none;
    color: var(--color-foreground);
    cursor: pointer;
    padding: 0.5rem;
    transition: color 0.3s;
}

.icon-button:hover {
    color: var(--color-accent);
}

.icon-button svg {
    width: 20px;
    height: 20px;
    stroke-width: 1.5;
}

/* Hero Section */
.hero-section {
    min-height: 100vh;
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    padding: 8rem 1.5rem 4rem;
    position: relative;
}

/* Guide Lines */
.guide-lines {
    position: fixed;
    inset: 0;
    pointer-events: none;
}

.guide-container {
    max-width: 1280px;
    margin: 0 auto;
    height: 100%;
    position: relative;
}

.guide-line {
    position: absolute;
    top: 0;
    bottom: 0;
    width: 1px;
    background-color: rgba(255, 255, 255, 0.05);
}

.guide-1 { left: 0; }
.guide-2 { left: 25%; }
.guide-3 { left: 50%; }
.guide-4 { left: 75%; }
.guide-5 { right: 0; }

/* Hero Content */
.hero-content, .hero-bottom {
    max-width: 1280px;
    margin: 0 auto;
    width: 100%;
    position: relative;
    z-index: 10;
}

.hero-grid {
    display: grid;
    grid-template-columns: repeat(12, 1fr);
    gap: 3rem;
    align-items: start;
}

.hero-title-col {
    grid-column: span 7;
}

.hero-description-col {
    grid-column: span 5;
    margin-top: 4rem;
}

.hero-title {
    font-size: clamp(3.5rem, 10vw, 9rem);
    font-weight: 200;
    line-height: 1;
    letter-spacing: -0.02em;
}

.title-line {
    display: block;
    color: var(--color-foreground);
}

.title-line.accent {
    color: var(--color-accent);
}

.description-content {
    max-width: 28rem;
    display: flex;
    flex-direction: column;
    gap: 2rem;
}

.description-text {
    font-size: 0.875rem;
    font-weight: 300;
    color: rgba(255, 255, 255, 0.8);
    line-height: 1.6;
    border-left: 1px solid rgba(255, 255, 255, 0.2);
    padding-left: 1.5rem;
}

/* Buttons */
.btn-ghost {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.625rem 1rem;
    background: transparent;
    border: 1px solid rgba(255, 255, 255, 0.3);
    border-radius: 2px;
    color: var(--color-foreground);
    font-size: 0.875rem;
    font-weight: 300;
    cursor: pointer;
    transition: all 0.3s;
    backdrop-filter: blur(4px);
}

.btn-ghost:hover {
    border-color: var(--color-accent);
    color: var(--color-accent);
}

.btn-ghost svg {
    width: 16px;
    height: 16px;
    stroke-width: 1.5;
    transition: transform 0.3s;
}

.btn-ghost:hover svg {
    transform: translateX(4px);
}

/* Bottom Section */
.hero-bottom {
    margin-top: auto;
}

.bottom-text-col {
    grid-column: span 6;
}

.bottom-cta-col {
    grid-column: span 6;
    display: flex;
    justify-content: center;
    align-items: flex-end;
}

.bottom-text {
    font-size: 0.875rem;
    font-weight: 300;
    color: rgba(255, 255, 255, 0.6);
    line-height: 1.6;
    max-width: 32rem;
}

.cta-circle {
    position: relative;
    background: transparent;
    border: none;
    cursor: pointer;
    padding: 0;
}

.circle-content {
    width: 8rem;
    height: 8rem;
    border-radius: 50%;
    background-color: var(--color-accent);
    display: flex;
    align-items: center;
    justify-content: center;
    transition: transform 0.3s;
    position: relative;
    z-index: 2;
}

.cta-circle:hover .circle-content {
    transform: scale(1.1);
}

.circle-text {
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--color-background);
    letter-spacing: 0.15em;
}

.circle-glow {
    position: absolute;
    inset: 0;
    border-radius: 50%;
    background-color: var(--color-accent);
    opacity: 0;
    filter: blur(2rem);
    transition: opacity 0.3s;
    z-index: 1;
}

.cta-circle:hover .circle-glow {
    opacity: 0.3;
}

/* Responsive */
 @DOX/guides/TACHES-IMMEDIATES-FRONTEND.md (max-width: 1024px) {
    .hero-title-col, .hero-description-col,
    .bottom-text-col, .bottom-cta-col {
        grid-column: span 12;
    }
    
    .hero-description-col {
        margin-top: 2rem;
    }
    
    .bottom-cta-col {
        justify-content: flex-start;
        margin-top: 2rem;
    }
}
// main.rs
use actix_web::{web, App, HttpResponse, HttpServer, Result};
use actix_files as fs;
use tera::Tera;

async fn index(tmpl: web::Data<Tera>) -> Result<HttpResponse> {
    let ctx = tera::Context::new();
    let rendered = tmpl.render("index.html", &ctx)
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::Ok().content_type("text/html").body(rendered))
}

async fn listings(tmpl: web::Data<Tera>) -> Result<HttpResponse> {
    // Votre logique pour la page listings
    let ctx = tera::Context::new();
    let rendered = tmpl.render("listings.html", &ctx)
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::Ok().content_type("text/html").body(rendered))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize Tera templates
    let tera = Tera::new("templates/**/*").expect("Failed to initialize Tera");
    
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(tera.clone()))
            // Static files
            .service(fs::Files::new("/static", "./static"))
            // Routes
            .route("/", web::get().to(index))
            .route("/listings", web::get().to(listings))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
[package]
name = "nexus"
version = "0.1.0"
edition = "2021"

[dependencies]
actix-web = "4"
actix-files = "0.6"
tera = "1"
tokio = { version = "1", features = ["full"] }
Points clés de l'adaptation :
HTMX : Intégré pour la navigation (bouton "Explorer les listings" utilise hx-get)
Tera : Template engine similaire à Jinja2, très utilisé en Rust
Lucide Icons : Remplace lucide-react par la version web CDN
Actix-Web : Framework web Rust performant et simple
CSS pur : Pas de Tailwind, tout en CSS vanilla pour plus de contrôle
Performance : Serveur très léger et rapide, parfait pour Tor