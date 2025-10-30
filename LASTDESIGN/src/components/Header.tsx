import { ShoppingCart, User, Search } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Link } from "react-router-dom";

const Header = () => {
  return (
    <header className="sticky top-0 z-50 w-full border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
      <div className="container flex h-16 items-center justify-between px-4 md:px-8">
        <div className="flex items-center gap-8">
          <Link to="/" className="flex items-center gap-2">
            <div className="text-2xl font-bold tracking-tight text-primary">
              NEXUS
            </div>
          </Link>
          <nav className="hidden md:flex items-center gap-6">
            <Link to="/" className="text-sm font-medium text-foreground hover:text-primary transition-colors">
              Home
            </Link>
            <a href="/#categories" className="text-sm font-medium text-muted-foreground hover:text-primary transition-colors">
              Categories
            </a>
            <Link to="/become-vendor" className="text-sm font-medium text-muted-foreground hover:text-primary transition-colors">
              Become a Vendor
            </Link>
          </nav>
        </div>
        
        <div className="flex items-center gap-4">
          <Link to="/search">
            <Button variant="ghost" size="icon" className="hidden md:flex">
              <Search className="h-5 w-5" />
            </Button>
          </Link>
          <Link to="/cart">
            <Button variant="ghost" size="icon">
              <ShoppingCart className="h-5 w-5" />
            </Button>
          </Link>
          <Link to="/auth">
            <Button variant="default" size="sm" className="gap-2">
              <User className="h-4 w-4" />
              <span className="hidden sm:inline">Login</span>
            </Button>
          </Link>
        </div>
      </div>
    </header>
  );
};

export default Header;
