import { ShieldCheck, Lock, Key, Headphones } from "lucide-react";

const badges = [
  {
    icon: ShieldCheck,
    title: "100% Monero Payment",
    description: "Anonymous cryptocurrency transactions",
  },
  {
    icon: Lock,
    title: "2/3 Multisig Escrow",
    description: "Secure three-party protection",
  },
  {
    icon: Key,
    title: "Non-Custodial",
    description: "You control your funds",
  },
  {
    icon: Headphones,
    title: "24/7 Support",
    description: "Always here to help",
  },
];

const TrustBadges = () => {
  return (
    <section className="py-16 bg-gradient-to-b from-background to-secondary/20">
      <div className="container mx-auto px-4">
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-8">
          {badges.map((badge, index) => {
            const Icon = badge.icon;
            return (
              <div
                key={index}
                className="flex flex-col items-center text-center p-6 rounded-2xl bg-card hover:bg-secondary/50 transition-all duration-300 hover:scale-105 animate-fade-in"
                style={{ animationDelay: `${index * 100}ms` }}
              >
                <div className="p-4 bg-coral/10 rounded-full mb-4">
                  <Icon className="h-8 w-8 text-coral" />
                </div>
                <h3 className="text-lg font-bold mb-2">{badge.title}</h3>
                <p className="text-sm text-muted-foreground">{badge.description}</p>
              </div>
            );
          })}
        </div>
      </div>
    </section>
  );
};

export default TrustBadges;
