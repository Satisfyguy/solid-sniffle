import Navigation from "@/components/Navigation";
import Hero from "@/components/Hero";
import CategoryCard from "@/components/CategoryCard";
import ProductCard from "@/components/ProductCard";
import { Package, Cpu, Pill, FileText, Wallet, Shield } from "lucide-react";
import { Button } from "@/components/ui/button";

const Index = () => {
  const categories = [
    { icon: Package, title: "DIGITAL GOODS", count: 342 },
    { icon: Cpu, title: "ELECTRONICS", count: 156 },
    { icon: Pill, title: "PHARMACEUTICALS", count: 203 },
    { icon: FileText, title: "DOCUMENTS", count: 89 },
    { icon: Wallet, title: "FINANCIAL", count: 127 },
    { icon: Shield, title: "SECURITY", count: 94 },
  ];

  const products = [
    {
      title: "Premium VPN Access - Lifetime",
      vendor: "CryptoVendor",
      price: "0.0045",
      rating: 5,
      verified: true,
      image: "",
    },
    {
      title: "Encrypted Communication Suite",
      vendor: "SecureDeals",
      price: "0.0089",
      rating: 5,
      verified: true,
      image: "",
    },
    {
      title: "Digital Identity Package",
      vendor: "PhantomMarket",
      price: "0.0234",
      rating: 4,
      verified: false,
      image: "",
    },
    {
      title: "Anonymous Email Service",
      vendor: "DarkMail",
      price: "0.0056",
      rating: 5,
      verified: true,
      image: "",
    },
    {
      title: "Secure Cloud Storage 1TB",
      vendor: "VaultKeeper",
      price: "0.0123",
      rating: 4,
      verified: true,
      image: "",
    },
    {
      title: "Privacy Tools Bundle",
      vendor: "AnonymousOne",
      price: "0.0167",
      rating: 5,
      verified: false,
      image: "",
    },
  ];

  return (
    <div className="min-h-screen bg-background">
      <Navigation />
      <Hero />

      {/* Categories Section */}
      <section id="categories" className="py-24 bg-background">
        <div className="container mx-auto px-6">
          <div className="mb-16 text-center">
            <h2 className="text-6xl font-black tracking-tighter mb-4">
              BROWSE CATEGORIES
            </h2>
            <p className="text-muted-foreground font-bold tracking-wider">
              SECURE • ANONYMOUS • UNTRACEABLE
            </p>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
            {categories.map((category, index) => (
              <CategoryCard
                key={index}
                icon={category.icon}
                title={category.title}
                count={category.count}
              />
            ))}
          </div>
        </div>
      </section>

      {/* Featured Listings Section */}
      <section id="listings" className="py-24 bg-muted/30">
        <div className="container mx-auto px-6">
          <div className="mb-16 flex items-center justify-between">
            <div>
              <h2 className="text-6xl font-black tracking-tighter mb-4">
                FEATURED LISTINGS
              </h2>
              <p className="text-muted-foreground font-bold tracking-wider">
                VERIFIED VENDORS • ESCROW PROTECTED
              </p>
            </div>
            <Button variant="default" size="lg" className="font-bold hidden md:flex">
              VIEW ALL
            </Button>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
            {products.map((product, index) => (
              <ProductCard key={index} {...product} />
            ))}
          </div>
        </div>
      </section>

      {/* Stats Banner */}
      <section className="py-24 bg-primary">
        <div className="container mx-auto px-6">
          <div className="grid grid-cols-1 md:grid-cols-4 gap-12 text-center text-primary-foreground">
            <div>
              <div className="text-6xl font-black mb-2">24/7</div>
              <div className="text-sm font-bold tracking-wider opacity-60">UPTIME</div>
            </div>
            <div>
              <div className="text-6xl font-black mb-2">0</div>
              <div className="text-sm font-bold tracking-wider opacity-60">LOGS KEPT</div>
            </div>
            <div>
              <div className="text-6xl font-black mb-2">256</div>
              <div className="text-sm font-bold tracking-wider opacity-60">BIT ENCRYPTION</div>
            </div>
            <div>
              <div className="text-6xl font-black mb-2">∞</div>
              <div className="text-sm font-bold tracking-wider opacity-60">ANONYMITY</div>
            </div>
          </div>
        </div>
      </section>

      {/* Footer */}
      <footer className="py-12 bg-background border-t border-border">
        <div className="container mx-auto px-6">
          <div className="flex flex-col md:flex-row items-center justify-between gap-6">
            <div className="flex items-center gap-4">
              <Shield className="w-8 h-8 text-primary" />
              <div>
                <div className="text-xl font-black">NEXUS</div>
                <div className="text-xs text-muted-foreground font-bold">
                  ANONYMOUS MARKETPLACE
                </div>
              </div>
            </div>
            
            <div className="text-center md:text-right">
              <div className="text-sm font-bold text-muted-foreground mb-2">
                ACCESSIBLE VIA TOR ONLY
              </div>
              <div className="text-xs text-muted-foreground font-mono">
                nexusmarket[.]onion
              </div>
            </div>
          </div>
        </div>
      </footer>
    </div>
  );
};

export default Index;
