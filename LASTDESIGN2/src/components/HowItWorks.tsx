import { Wallet, ShieldCheck, CheckCircle } from "lucide-react";
import { Card } from "@/components/ui/card";

const steps = [
  {
    icon: Wallet,
    title: "Pay with XMR",
    description: "Make anonymous payments using Monero cryptocurrency for complete privacy.",
    bgColor: "bg-coral/10",
    iconColor: "text-coral",
  },
  {
    icon: ShieldCheck,
    title: "Funds in Multisig Escrow",
    description: "Your payment is secured in a 2/3 multisig escrow protecting both parties.",
    bgColor: "bg-sky/10",
    iconColor: "text-sky",
  },
  {
    icon: CheckCircle,
    title: "Release on Delivery",
    description: "Confirm delivery and release funds. You stay in control throughout.",
    bgColor: "bg-mint/10",
    iconColor: "text-mint",
  },
];

const HowItWorks = () => {
  return (
    <section className="py-20 bg-background">
      <div className="container mx-auto px-4">
        <div className="text-center mb-16">
          <h2 className="text-4xl font-bold mb-4">How It Works</h2>
          <p className="text-muted-foreground text-lg max-w-2xl mx-auto">
            Three simple steps to secure, private commerce
          </p>
        </div>

        <div className="grid md:grid-cols-3 gap-8 relative">
          {/* Connection lines */}
          <div className="hidden md:block absolute top-1/3 left-0 right-0 h-0.5 bg-gradient-to-r from-coral via-sky to-mint -z-10"></div>

          {steps.map((step, index) => {
            const Icon = step.icon;
            return (
              <Card
                key={index}
                className="relative p-8 border-2 hover:border-coral transition-all duration-300 hover:shadow-xl animate-fade-in"
                style={{ animationDelay: `${index * 150}ms` }}
              >
                <div className="flex flex-col items-center text-center">
                  <div className={`w-16 h-16 rounded-full ${step.bgColor} flex items-center justify-center mb-6 relative`}>
                    <Icon className={`h-8 w-8 ${step.iconColor}`} />
                    <div className="absolute -top-2 -right-2 w-8 h-8 bg-coral rounded-full flex items-center justify-center text-white text-sm font-bold">
                      {index + 1}
                    </div>
                  </div>
                  
                  <h3 className="text-xl font-bold mb-3">{step.title}</h3>
                  <p className="text-muted-foreground">{step.description}</p>
                </div>
              </Card>
            );
          })}
        </div>
      </div>
    </section>
  );
};

export default HowItWorks;
