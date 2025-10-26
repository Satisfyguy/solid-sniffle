import heroPattern from "@/assets/hero-pattern.png";
import AnimatedLetter from "@/components/AnimatedLetter";

const Hero = () => {
  return (
    <section className="relative min-h-screen flex items-center justify-center overflow-hidden bg-primary">
      {/* Background Pattern with Animation */}
      <div 
        className="absolute inset-0 opacity-30 animate-wave"
        style={{
          backgroundImage: `url(${heroPattern})`,
          backgroundSize: 'cover',
          backgroundPosition: 'center',
        }}
      />
      
      {/* Animated Grid Lines */}
      <div className="absolute inset-0 opacity-10 animate-pulse-slow">
        <div className="h-full w-full" style={{
          backgroundImage: 'repeating-linear-gradient(0deg, transparent, transparent 49px, #000 49px, #000 51px)',
        }} />
      </div>

      {/* Floating Geometric Shapes */}
      <div className="absolute top-20 left-20 w-32 h-32 border-4 border-primary-foreground/10 animate-float" />
      <div className="absolute bottom-40 right-40 w-24 h-24 border-4 border-primary-foreground/10 animate-float" style={{ animationDelay: '2s' }} />
      <div className="absolute top-1/3 right-20 w-16 h-16 border-4 border-primary-foreground/10 animate-float" style={{ animationDelay: '4s' }} />

      {/* Content */}
      <div className="relative z-10 container mx-auto px-6">
        <div className="max-w-6xl mx-auto text-center">
          {/* Top Text */}
          <div className="mb-8 text-primary-foreground/60 text-sm font-bold tracking-widest">
            ANONYMOUS • SECURE • DECENTRALIZED
          </div>
          
          {/* Main Title with Interactive Letters */}
          <h1 className="text-[12rem] leading-none font-black tracking-tighter text-primary-foreground mb-8 select-none flex items-center justify-center gap-4">
            <AnimatedLetter letter="N" />
            <AnimatedLetter letter="E" />
            <AnimatedLetter letter="X" />
            <AnimatedLetter letter="U" />
            <AnimatedLetter letter="S" />
          </h1>
          
          {/* Subtitle */}
          <p className="text-4xl font-black tracking-tight text-primary-foreground mb-12">
            UNDERGROUND • MARKETPLACE
          </p>
          
          {/* Stats */}
          <div className="flex justify-center gap-16 text-primary-foreground">
            <div className="text-center">
              <div className="text-5xl font-black">1337</div>
              <div className="text-sm font-bold tracking-wider opacity-60">LISTINGS</div>
            </div>
            <div className="text-center">
              <div className="text-5xl font-black">420</div>
              <div className="text-sm font-bold tracking-wider opacity-60">VENDORS</div>
            </div>
            <div className="text-center">
              <div className="text-5xl font-black">100%</div>
              <div className="text-sm font-bold tracking-wider opacity-60">ANONYMOUS</div>
            </div>
          </div>
        </div>
      </div>

      {/* Scroll Indicator */}
      <div className="absolute bottom-8 left-1/2 -translate-x-1/2 text-primary-foreground animate-bounce">
        <div className="text-xs font-bold tracking-widest">SCROLL</div>
        <div className="w-px h-12 bg-primary-foreground/40 mx-auto mt-2" />
      </div>
    </section>
  );
};

export default Hero;
