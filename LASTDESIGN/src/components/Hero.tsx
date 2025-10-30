import { Button } from "@/components/ui/button";
import { ArrowRight } from "lucide-react";
import heroImage from "@/assets/hero-vault.jpg";

const Hero = () => {
  return (
    <section className="relative bg-muted overflow-hidden">
      <div className="container px-4 md:px-8 py-12 md:py-20">
        <div className="grid lg:grid-cols-2 gap-12 items-center">
          <div className="space-y-6 animate-slide-up">
            <div className="inline-block px-3 py-1 bg-accent/10 text-accent text-sm font-semibold rounded-full">
              Financial Sovereignty
            </div>
            <h1 className="text-5xl md:text-7xl font-bold tracking-tight">
              Your Market.
              <br />
              <span className="text-muted-foreground">Your Keys.</span>
              <br />
              Your Privacy.
            </h1>
            <p className="text-xl text-muted-foreground max-w-lg">
              Experience true freedom in commerce. NEXUS is the marketplace built on privacy, powered by Monero, secured by multisig escrow.
            </p>
            <div className="flex flex-wrap gap-4">
              <Button size="lg" className="gap-2 text-base">
                Explore Marketplace
                <ArrowRight className="h-5 w-5" />
              </Button>
              <Button size="lg" variant="outline" className="text-base">
                Learn More
              </Button>
            </div>
          </div>
          
          <div className="relative animate-fade-in">
            <div className="aspect-video rounded-2xl overflow-hidden shadow-2xl">
              <img 
                src={heroImage} 
                alt="Secure digital vault representing NEXUS marketplace security" 
                className="w-full h-full object-cover"
              />
            </div>
          </div>
        </div>
      </div>
    </section>
  );
};

export default Hero;
