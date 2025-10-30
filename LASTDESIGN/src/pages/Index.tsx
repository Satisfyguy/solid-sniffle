import Header from "@/components/Header";
import Hero from "@/components/Hero";
import CategoryCard from "@/components/CategoryCard";
import TrustBadge from "@/components/TrustBadge";
import ProductCard from "@/components/ProductCard";
import { Headphones, Watch, Laptop, Gamepad2, Shield, Key, Clock, Lock } from "lucide-react";

const Index = () => {
  return (
    <div className="min-h-screen bg-background">
      <Header />
      
      <Hero />
      
      {/* Categories Section */}
      <section id="categories" className="container px-4 md:px-8 py-16 md:py-24">
        <div className="text-center mb-12 animate-slide-up">
          <h2 className="text-4xl md:text-5xl font-bold mb-4">Browse Categories</h2>
          <p className="text-xl text-muted-foreground">Discover products across all categories</p>
        </div>
        
        <div className="grid sm:grid-cols-2 lg:grid-cols-4 gap-6">
          <CategoryCard 
            title="Audio"
            subtitle="Enjoy With"
            colorClass="bg-gradient-to-br from-gray-800 to-gray-900"
            icon={Headphones}
          />
          <CategoryCard 
            title="Wearables"
            subtitle="Life"
            colorClass="bg-gradient-to-br from-category-yellow to-amber-500"
            icon={Watch}
          />
          <CategoryCard 
            title="Devices"
            subtitle="Head"
            colorClass="bg-gradient-to-br from-accent to-red-600"
            icon={Laptop}
          />
          <CategoryCard 
            title="Gaming"
            subtitle="Best"
            colorClass="bg-gradient-to-br from-muted-foreground to-gray-400"
            icon={Gamepad2}
          />
          <CategoryCard 
            title="Game"
            subtitle="Play"
            colorClass="bg-gradient-to-br from-category-green to-emerald-600"
            icon={Gamepad2}
          />
          <CategoryCard 
            title="Amazon"
            subtitle="New"
            colorClass="bg-gradient-to-br from-category-blue to-blue-600"
            icon={Laptop}
          />
        </div>
      </section>
      
      {/* Trust Section */}
      <section className="bg-muted py-16 md:py-20">
        <div className="container px-4 md:px-8">
          <div className="grid sm:grid-cols-2 lg:grid-cols-4 gap-8">
            <TrustBadge 
              icon={Shield}
              title="100% Monero Payment"
              description="Private cryptocurrency transactions"
            />
            <TrustBadge 
              icon={Lock}
              title="2/3 Multisig Escrow"
              description="Your funds secured until delivery"
            />
            <TrustBadge 
              icon={Clock}
              title="24/7 Support"
              description="We're here whenever you need us"
            />
            <TrustBadge 
              icon={Key}
              title="Non-Custodial"
              description="You control your private keys"
            />
          </div>
        </div>
      </section>
      
      {/* Featured Products */}
      <section className="container px-4 md:px-8 py-16 md:py-24">
        <div className="text-center mb-12">
          <h2 className="text-4xl md:text-5xl font-bold mb-4">Featured Products</h2>
          <p className="text-xl text-muted-foreground">Hand-picked items from trusted vendors</p>
        </div>
        
        <div className="grid sm:grid-cols-2 lg:grid-cols-4 gap-6">
          <ProductCard 
            title="Premium Wireless Headphones"
            price="0.24"
            image="https://images.unsplash.com/photo-1505740420928-5e560c06d30e?w=800&q=80"
            category="Audio"
            featured
          />
          <ProductCard 
            title="Smart Fitness Watch"
            price="0.18"
            image="https://images.unsplash.com/photo-1523275335684-37898b6baf30?w=800&q=80"
            category="Wearables"
          />
          <ProductCard 
            title="Ultrabook Laptop Pro"
            price="1.85"
            image="https://images.unsplash.com/photo-1496181133206-80ce9b88a853?w=800&q=80"
            category="Devices"
            featured
          />
          <ProductCard 
            title="Gaming Console Bundle"
            price="0.95"
            image="https://images.unsplash.com/photo-1486401899868-0e435ed85128?w=800&q=80"
            category="Gaming"
          />
        </div>
      </section>
      
      {/* How It Works */}
      <section className="bg-primary text-primary-foreground py-16 md:py-24">
        <div className="container px-4 md:px-8">
          <div className="text-center mb-12">
            <h2 className="text-4xl md:text-5xl font-bold mb-4">How It Works</h2>
            <p className="text-xl opacity-90">Simple, secure, private shopping in three steps</p>
          </div>
          
          <div className="grid md:grid-cols-3 gap-12 max-w-5xl mx-auto">
            <div className="text-center space-y-4">
              <div className="w-16 h-16 bg-accent rounded-2xl flex items-center justify-center text-2xl font-bold mx-auto">
                1
              </div>
              <h3 className="text-2xl font-bold">Browse & Select</h3>
              <p className="opacity-90">
                Explore categories and choose products from verified vendors
              </p>
            </div>
            
            <div className="text-center space-y-4">
              <div className="w-16 h-16 bg-accent rounded-2xl flex items-center justify-center text-2xl font-bold mx-auto">
                2
              </div>
              <h3 className="text-2xl font-bold">Secure Payment</h3>
              <p className="opacity-90">
                Pay with Monero. Funds held in 2/3 multisig escrow for protection
              </p>
            </div>
            
            <div className="text-center space-y-4">
              <div className="w-16 h-16 bg-accent rounded-2xl flex items-center justify-center text-2xl font-bold mx-auto">
                3
              </div>
              <h3 className="text-2xl font-bold">Receive & Confirm</h3>
              <p className="opacity-90">
                Get your items, confirm delivery, and funds are released
              </p>
            </div>
          </div>
        </div>
      </section>
      
      {/* Footer */}
      <footer className="bg-muted py-12">
        <div className="container px-4 md:px-8">
          <div className="text-center space-y-4">
            <div className="text-3xl font-bold text-primary">NEXUS</div>
            <p className="text-muted-foreground">Your Market. Your Keys. Your Privacy.</p>
            <p className="text-sm text-muted-foreground">© 2025 NEXUS. Built for financial sovereignty.</p>
          </div>
        </div>
      </footer>
    </div>
  );
};

export default Index;
