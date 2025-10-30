import { ShoppingBag, Search, User } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Link } from "react-router-dom";

const Header = () => {
  return (
    <header className="sticky top-0 z-50 w-full border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
      <div className="container mx-auto flex h-16 items-center justify-between px-4">
        <Link to="/" className="flex items-center gap-2 group">
          <div className="text-2xl font-bold tracking-tight">
            NEXUS
          </div>
        </Link>

        <nav className="hidden md:flex items-center gap-8">
          <Link to="/" className="text-sm font-medium hover:text-coral transition-colors">
            Home
          </Link>
          <Link to="/categories" className="text-sm font-medium hover:text-coral transition-colors">
            Categories
          </Link>
          <Link to="/vendors" className="text-sm font-medium hover:text-coral transition-colors">
            Become a Vendor
          </Link>
        </nav>

        <div className="flex items-center gap-3">
          <Button
            variant="ghost"
            size="icon"
            className="hover:bg-secondary"
            onClick={() => window.location.href = "/search"}
          >
            <Search className="h-5 w-5" />
          </Button>
          <Button
            variant="ghost"
            size="icon"
            className="hover:bg-secondary"
            onClick={() => window.location.href = "/cart"}
          >
            <ShoppingBag className="h-5 w-5" />
          </Button>
          <Button
            variant="outline"
            size="sm"
            onClick={() => window.location.href = "/auth"}
          >
            <User className="h-4 w-4 mr-2" />
            Login
          </Button>
        </div>
      </div>
    </header>
  );
};

export default Header;
