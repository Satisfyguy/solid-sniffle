import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Store, Shield, TrendingUp, Users, CheckCircle2 } from "lucide-react";
import { Link } from "react-router-dom";

const BecomeVendor = () => {
  const [isLoading, setIsLoading] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setIsLoading(true);
    // Vendor application logic will be added with backend
    setTimeout(() => setIsLoading(false), 1500);
  };

  return (
    <div className="min-h-screen bg-background">
      {/* Header */}
      <header className="border-b bg-background/95 backdrop-blur">
        <div className="container flex h-16 items-center px-4 md:px-8">
          <Link to="/" className="flex items-center gap-2">
            <div className="text-2xl font-bold tracking-tight text-primary">NEXUS</div>
          </Link>
        </div>
      </header>

      {/* Hero Section */}
      <section className="bg-primary text-primary-foreground py-16 md:py-24">
        <div className="container px-4 md:px-8">
          <div className="max-w-3xl mx-auto text-center space-y-6 animate-slide-up">
            <div className="inline-flex h-16 w-16 items-center justify-center rounded-2xl bg-accent">
              <Store className="h-8 w-8" />
            </div>
            <h1 className="text-4xl md:text-6xl font-bold">Become a NEXUS Vendor</h1>
            <p className="text-xl opacity-90">
              Join a marketplace built on privacy, security, and fair commerce. Reach customers who value financial sovereignty.
            </p>
          </div>
        </div>
      </section>

      {/* Benefits Section */}
      <section className="container px-4 md:px-8 py-16 md:py-24">
        <div className="text-center mb-12">
          <h2 className="text-3xl md:text-4xl font-bold mb-4">Why Sell on NEXUS?</h2>
          <p className="text-muted-foreground text-lg">Premium benefits for trusted vendors</p>
        </div>

        <div className="grid md:grid-cols-2 lg:grid-cols-4 gap-6 mb-16">
          <Card className="text-center">
            <CardHeader>
              <div className="mx-auto h-12 w-12 rounded-full bg-primary/10 flex items-center justify-center mb-4">
                <Shield className="h-6 w-6 text-primary" />
              </div>
              <CardTitle>Secure Escrow</CardTitle>
            </CardHeader>
            <CardContent>
              <CardDescription>
                2/3 multisig protection ensures both buyer and seller security
              </CardDescription>
            </CardContent>
          </Card>

          <Card className="text-center">
            <CardHeader>
              <div className="mx-auto h-12 w-12 rounded-full bg-primary/10 flex items-center justify-center mb-4">
                <TrendingUp className="h-6 w-6 text-primary" />
              </div>
              <CardTitle>Low Fees</CardTitle>
            </CardHeader>
            <CardContent>
              <CardDescription>
                Competitive commission rates with transparent pricing
              </CardDescription>
            </CardContent>
          </Card>

          <Card className="text-center">
            <CardHeader>
              <div className="mx-auto h-12 w-12 rounded-full bg-primary/10 flex items-center justify-center mb-4">
                <Users className="h-6 w-6 text-primary" />
              </div>
              <CardTitle>Privacy-First</CardTitle>
            </CardHeader>
            <CardContent>
              <CardDescription>
                Reach customers who value privacy and cryptocurrency payments
              </CardDescription>
            </CardContent>
          </Card>

          <Card className="text-center">
            <CardHeader>
              <div className="mx-auto h-12 w-12 rounded-full bg-primary/10 flex items-center justify-center mb-4">
                <CheckCircle2 className="h-6 w-6 text-primary" />
              </div>
              <CardTitle>Easy Setup</CardTitle>
            </CardHeader>
            <CardContent>
              <CardDescription>
                Quick onboarding process with dedicated vendor support
              </CardDescription>
            </CardContent>
          </Card>
        </div>

        {/* Application Form */}
        <div className="max-w-2xl mx-auto">
          <Card>
            <CardHeader>
              <CardTitle className="text-2xl">Vendor Application</CardTitle>
              <CardDescription>
                Fill out the form below to apply. We'll review your application within 48 hours.
              </CardDescription>
            </CardHeader>
            <CardContent>
              <form onSubmit={handleSubmit} className="space-y-6">
                <div className="grid md:grid-cols-2 gap-4">
                  <div className="space-y-2">
                    <Label htmlFor="first-name">First Name</Label>
                    <Input id="first-name" placeholder="John" required />
                  </div>
                  <div className="space-y-2">
                    <Label htmlFor="last-name">Last Name</Label>
                    <Input id="last-name" placeholder="Doe" required />
                  </div>
                </div>

                <div className="space-y-2">
                  <Label htmlFor="email">Email Address</Label>
                  <Input id="email" type="email" placeholder="vendor@example.com" required />
                </div>

                <div className="space-y-2">
                  <Label htmlFor="store-name">Store Name</Label>
                  <Input id="store-name" placeholder="Your Store Name" required />
                </div>

                <div className="space-y-2">
                  <Label htmlFor="category">Primary Product Category</Label>
                  <select
                    id="category"
                    className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                    required
                  >
                    <option value="">Select a category</option>
                    <option value="audio">Audio</option>
                    <option value="wearables">Wearables</option>
                    <option value="devices">Devices</option>
                    <option value="gaming">Gaming</option>
                    <option value="other">Other</option>
                  </select>
                </div>

                <div className="space-y-2">
                  <Label htmlFor="description">Store Description</Label>
                  <Textarea
                    id="description"
                    placeholder="Tell us about your store and the products you plan to sell..."
                    rows={5}
                    required
                  />
                </div>

                <div className="space-y-2">
                  <Label htmlFor="experience">Previous Selling Experience</Label>
                  <Textarea
                    id="experience"
                    placeholder="Share your experience with e-commerce or marketplace selling..."
                    rows={4}
                  />
                </div>

                <div className="space-y-2">
                  <Label htmlFor="website">Website or Social Media (Optional)</Label>
                  <Input id="website" type="url" placeholder="https://..." />
                </div>

                <div className="flex items-start gap-2">
                  <input type="checkbox" className="mt-1 rounded" required />
                  <span className="text-sm text-muted-foreground">
                    I agree to NEXUS vendor terms and conditions, including the use of 2/3 multisig escrow for all transactions
                  </span>
                </div>

                <Button type="submit" className="w-full" size="lg" disabled={isLoading}>
                  {isLoading ? "Submitting Application..." : "Submit Application"}
                </Button>
              </form>
            </CardContent>
          </Card>
        </div>
      </section>
    </div>
  );
};

export default BecomeVendor;
