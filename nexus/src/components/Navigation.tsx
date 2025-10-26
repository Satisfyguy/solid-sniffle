import { Button } from "@/components/ui/button";
import { Search, Shield, User } from "lucide-react";

const Navigation = () => {
  return (
    <nav className="fixed top-0 left-0 right-0 z-50 border-b border-border bg-background/80 backdrop-blur-sm">
      <div className="container mx-auto px-6 py-4">
        <div className="flex items-center justify-between">
          {/* Logo */}
          <div className="flex items-center gap-4">
            <div className="w-12 h-12 bg-primary flex items-center justify-center">
              <Shield className="w-6 h-6 text-primary-foreground" />
            </div>
            <h1 className="text-2xl font-black tracking-tighter">NEXUS</h1>
          </div>

          {/* Main Navigation */}
          <div className="hidden md:flex items-center gap-8">
            <a href="#home" className="text-sm font-bold hover:text-primary transition-colors">
              HOME
            </a>
            <a href="#categories" className="text-sm font-bold hover:text-primary transition-colors">
              CATEGORIES
            </a>
            <a href="#listings" className="text-sm font-bold hover:text-primary transition-colors">
              LISTINGS
            </a>
            <a href="#vendors" className="text-sm font-bold hover:text-primary transition-colors">
              VENDORS
            </a>
          </div>

          {/* Actions */}
          <div className="flex items-center gap-4">
            <Button variant="ghost" size="icon">
              <Search className="w-5 h-5" />
            </Button>
            <Button variant="default" size="sm" className="hidden md:flex">
              <User className="w-4 h-4 mr-2" />
              CONNECT
            </Button>
          </div>
        </div>
      </div>
    </nav>
  );
};

export default Navigation;
