import { Shield } from "lucide-react";

const Footer = () => {
  return (
    <footer className="bg-foreground text-background py-12">
      <div className="container mx-auto px-4">
        <div className="grid grid-cols-1 md:grid-cols-4 gap-8 mb-8">
          <div>
            <div className="flex items-center gap-2 mb-4">
              <Shield className="h-6 w-6" />
              <span className="text-xl font-bold">NEXUS</span>
            </div>
            <p className="text-sm opacity-80">
              Your Market. Your Keys. Your Privacy.
            </p>
          </div>

          <div>
            <h4 className="font-bold mb-4">Marketplace</h4>
            <ul className="space-y-2 text-sm opacity-80">
              <li><a href="#" className="hover:opacity-100 transition-opacity">Browse Products</a></li>
              <li><a href="#" className="hover:opacity-100 transition-opacity">Categories</a></li>
              <li><a href="#" className="hover:opacity-100 transition-opacity">Become a Vendor</a></li>
              <li><a href="#" className="hover:opacity-100 transition-opacity">Seller Guidelines</a></li>
            </ul>
          </div>

          <div>
            <h4 className="font-bold mb-4">Support</h4>
            <ul className="space-y-2 text-sm opacity-80">
              <li><a href="#" className="hover:opacity-100 transition-opacity">Help Center</a></li>
              <li><a href="#" className="hover:opacity-100 transition-opacity">How It Works</a></li>
              <li><a href="#" className="hover:opacity-100 transition-opacity">Security</a></li>
              <li><a href="#" className="hover:opacity-100 transition-opacity">Contact Us</a></li>
            </ul>
          </div>

          <div>
            <h4 className="font-bold mb-4">Legal</h4>
            <ul className="space-y-2 text-sm opacity-80">
              <li><a href="#" className="hover:opacity-100 transition-opacity">Terms of Service</a></li>
              <li><a href="#" className="hover:opacity-100 transition-opacity">Privacy Policy</a></li>
              <li><a href="#" className="hover:opacity-100 transition-opacity">Dispute Resolution</a></li>
              <li><a href="#" className="hover:opacity-100 transition-opacity">Cookie Policy</a></li>
            </ul>
          </div>
        </div>

        <div className="border-t border-background/20 pt-8 text-center text-sm opacity-80">
          <p>&copy; 2025 NEXUS. All rights reserved. Powered by privacy.</p>
        </div>
      </div>
    </footer>
  );
};

export default Footer;
