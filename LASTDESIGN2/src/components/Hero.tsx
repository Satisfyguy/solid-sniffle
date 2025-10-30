import { Button } from "@/components/ui/button";
import { ShieldCheck, ArrowRight } from "lucide-react";
import heroImage from "@/assets/hero-security.jpg";

const Hero = () => {
  return (
    <section className="relative overflow-hidden bg-gradient-to-br from-coral/5 via-background to-sky/5">
      <div className="container mx-auto px-4 py-20 md:py-28">
        <div className="grid md:grid-cols-2 gap-12 items-center">
          <div className="space-y-8 animate-fade-in">
            <div className="inline-flex items-center gap-2 bg-coral/10 text-coral px-4 py-2 rounded-full text-sm font-medium">
              <ShieldCheck className="h-4 w-4" />
              100% Private & Secure
            </div>
            
            <h1 className="text-5xl md:text-6xl font-bold leading-tight">
              Your Market.
              <br />
              <span className="text-coral">Your Keys.</span>
              <br />
              Your Privacy.
            </h1>
            
            <p className="text-lg text-muted-foreground max-w-lg">
              Welcome to NEXUS. Commerce with complete confidentiality. 
              Pay with Monero, protected by 2/3 Multisig escrow, and keep full control of your funds.
            </p>
            
            <div className="flex flex-wrap gap-4">
              <Button variant="hero" size="lg" className="group">
                Start Shopping
                <ArrowRight className="h-5 w-5 group-hover:translate-x-1 transition-transform" />
              </Button>
              <Button variant="outline" size="lg">
                Learn More
              </Button>
            </div>
          </div>
          
          <div className="relative animate-slide-up">
            <div className="aspect-square rounded-3xl overflow-hidden shadow-2xl">
              <img 
                src={heroImage} 
                alt="Secure digital commerce"
                className="w-full h-full object-cover"
              />
            </div>
            <div className="absolute -bottom-6 -right-6 w-48 h-48 bg-mint/20 rounded-full blur-3xl"></div>
            <div className="absolute -top-6 -left-6 w-48 h-48 bg-sky/20 rounded-full blur-3xl"></div>
          </div>
        </div>
      </div>
    </section>
  );
};

export default Hero;
