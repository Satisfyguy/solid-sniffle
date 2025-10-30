import Header from "@/components/Header";
import Footer from "@/components/Footer";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { Card } from "@/components/ui/card";
import { Label } from "@/components/ui/label";
import { ShieldCheck, TrendingUp, Users, DollarSign, Lock, Zap } from "lucide-react";
import { useState } from "react";
import { toast } from "sonner";

const benefits = [
  {
    icon: ShieldCheck,
    title: "Secure Escrow",
    description: "2/3 Multisig protection for all transactions",
  },
  {
    icon: TrendingUp,
    title: "Global Reach",
    description: "Access to privacy-conscious buyers worldwide",
  },
  {
    icon: Users,
    title: "Growing Community",
    description: "Join thousands of trusted vendors",
  },
  {
    icon: DollarSign,
    title: "Low Fees",
    description: "Competitive rates with transparent pricing",
  },
  {
    icon: Lock,
    title: "Full Privacy",
    description: "Anonymous operations with Monero",
  },
  {
    icon: Zap,
    title: "Fast Payouts",
    description: "Quick settlement after delivery confirmation",
  },
];

const BecomeVendor = () => {
  const [formData, setFormData] = useState({
    businessName: "",
    email: "",
    category: "",
    experience: "",
    description: "",
  });

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    toast.success("Application submitted! We'll review and contact you soon.");
    setFormData({
      businessName: "",
      email: "",
      category: "",
      experience: "",
      description: "",
    });
  };

  return (
    <div className="min-h-screen flex flex-col">
      <Header />
      
      <main className="flex-1">
        {/* Hero Section */}
        <section className="bg-gradient-to-br from-coral via-coral/90 to-coral/80 text-white py-20">
          <div className="container mx-auto px-4">
            <div className="max-w-3xl mx-auto text-center space-y-6">
              <h1 className="text-5xl md:text-6xl font-bold animate-fade-in">
                Become a NEXUS Vendor
              </h1>
              <p className="text-xl opacity-90 animate-fade-in">
                Join the most secure and private marketplace. Sell your products to a global audience while maintaining complete anonymity.
              </p>
              <Button
                variant="outline"
                size="lg"
                className="bg-white text-coral hover:bg-white/90 border-white animate-slide-up"
                onClick={() => document.getElementById("application-form")?.scrollIntoView({ behavior: "smooth" })}
              >
                Apply Now
              </Button>
            </div>
          </div>
        </section>

        {/* Benefits Section */}
        <section className="py-20 bg-secondary/30">
          <div className="container mx-auto px-4">
            <div className="text-center mb-12">
              <h2 className="text-4xl font-bold mb-4">Why Sell on NEXUS?</h2>
              <p className="text-muted-foreground text-lg">
                Everything you need to succeed in private commerce
              </p>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8">
              {benefits.map((benefit, index) => {
                const Icon = benefit.icon;
                return (
                  <Card
                    key={index}
                    className="p-6 hover:shadow-xl transition-all duration-300 hover:scale-105 animate-fade-in"
                    style={{ animationDelay: `${index * 100}ms` }}
                  >
                    <div className="flex flex-col items-center text-center">
                      <div className="p-4 bg-coral/10 rounded-full mb-4">
                        <Icon className="h-8 w-8 text-coral" />
                      </div>
                      <h3 className="text-xl font-bold mb-2">{benefit.title}</h3>
                      <p className="text-muted-foreground">{benefit.description}</p>
                    </div>
                  </Card>
                );
              })}
            </div>
          </div>
        </section>

        {/* Stats Section */}
        <section className="py-16 bg-foreground text-background">
          <div className="container mx-auto px-4">
            <div className="grid grid-cols-1 md:grid-cols-4 gap-8 text-center">
              <div>
                <div className="text-4xl font-bold text-coral mb-2">10K+</div>
                <div className="text-sm opacity-80">Active Vendors</div>
              </div>
              <div>
                <div className="text-4xl font-bold text-coral mb-2">50K+</div>
                <div className="text-sm opacity-80">Products Listed</div>
              </div>
              <div>
                <div className="text-4xl font-bold text-coral mb-2">100K+</div>
                <div className="text-sm opacity-80">Happy Customers</div>
              </div>
              <div>
                <div className="text-4xl font-bold text-coral mb-2">99.9%</div>
                <div className="text-sm opacity-80">Uptime</div>
              </div>
            </div>
          </div>
        </section>

        {/* Application Form */}
        <section id="application-form" className="py-20">
          <div className="container mx-auto px-4">
            <div className="max-w-2xl mx-auto">
              <div className="text-center mb-12">
                <h2 className="text-4xl font-bold mb-4">Vendor Application</h2>
                <p className="text-muted-foreground text-lg">
                  Tell us about your business and we'll get back to you
                </p>
              </div>

              <Card className="p-8 shadow-xl">
                <form onSubmit={handleSubmit} className="space-y-6">
                  <div className="space-y-2">
                    <Label htmlFor="businessName">Business Name *</Label>
                    <Input
                      id="businessName"
                      placeholder="Your business or brand name"
                      value={formData.businessName}
                      onChange={(e) => setFormData({ ...formData, businessName: e.target.value })}
                      required
                      className="border-2 focus-visible:ring-coral"
                    />
                  </div>

                  <div className="space-y-2">
                    <Label htmlFor="email">Contact Email *</Label>
                    <Input
                      id="email"
                      type="email"
                      placeholder="your@email.com"
                      value={formData.email}
                      onChange={(e) => setFormData({ ...formData, email: e.target.value })}
                      required
                      className="border-2 focus-visible:ring-coral"
                    />
                  </div>

                  <div className="space-y-2">
                    <Label htmlFor="category">Primary Category *</Label>
                    <Input
                      id="category"
                      placeholder="e.g., Software, Digital Services, Hardware"
                      value={formData.category}
                      onChange={(e) => setFormData({ ...formData, category: e.target.value })}
                      required
                      className="border-2 focus-visible:ring-coral"
                    />
                  </div>

                  <div className="space-y-2">
                    <Label htmlFor="experience">Years of Experience</Label>
                    <Input
                      id="experience"
                      type="number"
                      placeholder="0"
                      value={formData.experience}
                      onChange={(e) => setFormData({ ...formData, experience: e.target.value })}
                      className="border-2 focus-visible:ring-coral"
                    />
                  </div>

                  <div className="space-y-2">
                    <Label htmlFor="description">Tell Us About Your Business *</Label>
                    <Textarea
                      id="description"
                      placeholder="What products or services do you offer? What makes your business unique?"
                      value={formData.description}
                      onChange={(e) => setFormData({ ...formData, description: e.target.value })}
                      required
                      rows={5}
                      className="border-2 focus-visible:ring-coral resize-none"
                    />
                  </div>

                  <Button
                    type="submit"
                    variant="hero"
                    size="lg"
                    className="w-full"
                  >
                    Submit Application
                  </Button>

                  <p className="text-sm text-muted-foreground text-center">
                    We typically review applications within 48 hours
                  </p>
                </form>
              </Card>
            </div>
          </div>
        </section>
      </main>

      <Footer />
    </div>
  );
};

export default BecomeVendor;
